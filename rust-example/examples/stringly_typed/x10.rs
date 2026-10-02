use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use std::sync::{mpsc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricKind {
    Counter,
    Gauge,
}

#[derive(Debug)]
enum LineError {
    MissingField(&'static str),
    UnknownKind(String),
    BadValue(String),
    BadLabel(String),
    KindMismatch { name: String, existing: MetricKind },
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineError::MissingField(field) => write!(f, "missing {field}"),
            LineError::UnknownKind(k) => write!(f, "unknown metric kind `{k}`"),
            LineError::BadValue(v) => write!(f, "value `{v}` is not an integer"),
            LineError::BadLabel(l) => write!(f, "label `{l}` is not key=value"),
            LineError::KindMismatch { name, existing } => {
                write!(f, "`{name}` is already registered as {existing:?}")
            }
        }
    }
}

impl FromStr for MetricKind {
    type Err = LineError;

    fn from_str(s: &str) -> Result<Self, LineError> {
        match s {
            "counter" => Ok(MetricKind::Counter),
            "gauge" => Ok(MetricKind::Gauge),
            other => Err(LineError::UnknownKind(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SeriesKey {
    name: String,
    labels: Vec<(String, String)>,
}

impl SeriesKey {
    fn parse(spec: &str) -> Result<SeriesKey, LineError> {
        let (name, rest) = match spec.split_once('{') {
            Some((n, r)) => (n, r.trim_end_matches('}')),
            None => (spec, ""),
        };
        let mut labels = rest
            .split(',')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                pair.split_once('=')
                    .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                    .ok_or_else(|| LineError::BadLabel(pair.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        labels.sort();
        Ok(SeriesKey { name: name.trim().to_string(), labels })
    }
}

impl fmt::Display for SeriesKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.labels.is_empty() {
            let pairs: Vec<String> =
                self.labels.iter().map(|(k, v)| format!("{k}=\"{v}\"")).collect();
            write!(f, "{{{}}}", pairs.join(","))?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct Registry {
    series: Mutex<BTreeMap<SeriesKey, (MetricKind, i64)>>,
}

impl Registry {
    fn apply(&self, line: &str) -> Result<(), LineError> {
        let mut parts = line.split_whitespace();
        let spec = parts.next().ok_or(LineError::MissingField("series"))?;
        let kind: MetricKind = parts.next().ok_or(LineError::MissingField("kind"))?.parse()?;
        let raw = parts.next().ok_or(LineError::MissingField("value"))?;
        let value: i64 = raw.parse().map_err(|_| LineError::BadValue(raw.to_string()))?;
        let key = SeriesKey::parse(spec)?;

        let mut series = self.series.lock().unwrap();
        let slot = series.entry(key).or_insert((kind, 0));
        if slot.0 != kind {
            let name = spec.split('{').next().unwrap_or(spec).to_string();
            return Err(LineError::KindMismatch { name, existing: slot.0 });
        }
        match kind {
            MetricKind::Counter => slot.1 += value,
            MetricKind::Gauge => slot.1 = value,
        }
        Ok(())
    }

    fn snapshot(&self) -> Vec<(SeriesKey, MetricKind, i64)> {
        let series = self.series.lock().unwrap();
        series.iter().map(|(k, (kind, v))| (k.clone(), *kind, *v)).collect()
    }
}

fn main() {
    let registry = Registry::default();
    let batches: [&[&str]; 3] = [
        &[
            "http_requests{route=/login,code=200} counter 3",
            "http_requests{route=/login,code=500} counter 1",
            "queue_depth{queue=emails} gauge 17",
        ],
        &[
            "http_requests{code=200,route=/login} counter 2",
            "queue_depth{queue=emails} gauge 9",
            "cache_hits counter lots",
        ],
        &[
            "queue_depth{queue=emails} counter 1",
            "disk_free{mount=/var} gauge 52000",
            "temperature{room} gauge 21",
            "uploads histogram 4",
        ],
    ];

    let (tx, rx) = mpsc::channel();
    thread::scope(|scope| {
        for (worker, batch) in batches.iter().enumerate() {
            let tx = tx.clone();
            let registry = &registry;
            scope.spawn(move || {
                for line in batch.iter() {
                    if let Err(e) = registry.apply(line) {
                        let _ = tx.send(format!("worker {worker}: {line:?}: {e}"));
                    }
                }
            });
        }
    });
    drop(tx);
    for failure in rx {
        eprintln!("{failure}");
    }

    for (key, kind, value) in registry.snapshot() {
        println!("{key} {kind:?} {value}");
    }
}
