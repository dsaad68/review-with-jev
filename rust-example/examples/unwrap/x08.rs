use std::collections::HashMap;
use std::fmt;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Debug)]
enum TelemetryError {
    Malformed(String),
    UnknownSensor(String),
    OutOfRange { sensor: String, value: f64 },
}

impl fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelemetryError::Malformed(l) => write!(f, "malformed line: {l}"),
            TelemetryError::UnknownSensor(s) => write!(f, "unknown sensor: {s}"),
            TelemetryError::OutOfRange { sensor, value } => {
                write!(f, "{sensor} reading {value} out of range")
            }
        }
    }
}

trait Calibrate {
    fn id(&self) -> &str;
    fn bounds(&self) -> (f64, f64);
    fn apply(&self, raw: f64) -> f64;
}

struct Linear {
    id: String,
    gain: f64,
    offset: f64,
    bounds: (f64, f64),
}

impl Calibrate for Linear {
    fn id(&self) -> &str {
        &self.id
    }
    fn bounds(&self) -> (f64, f64) {
        self.bounds
    }
    fn apply(&self, raw: f64) -> f64 {
        raw * self.gain + self.offset
    }
}

struct Registry<C: Calibrate> {
    sensors: HashMap<String, C>,
}

impl<C: Calibrate> Registry<C> {
    fn new(items: Vec<C>) -> Self {
        let sensors = items.into_iter().map(|c| (c.id().to_string(), c)).collect();
        Registry { sensors }
    }

    fn lookup<'a>(&'a self, id: &str) -> Result<&'a C, TelemetryError> {
        self.sensors
            .get(id)
            .ok_or_else(|| TelemetryError::UnknownSensor(id.to_string()))
    }
}

struct Reading<'a> {
    sensor: &'a str,
    value: f64,
}

fn parse_line(line: &str) -> Result<Reading<'_>, TelemetryError> {
    let (sensor, raw) = line
        .split_once('=')
        .ok_or_else(|| TelemetryError::Malformed(line.to_string()))?;
    let value = raw
        .trim()
        .parse::<f64>()
        .map_err(|_| TelemetryError::Malformed(line.to_string()))?;
    Ok(Reading { sensor: sensor.trim(), value })
}

fn calibrate<C: Calibrate>(reg: &Registry<C>, line: &str) -> Result<(String, f64), TelemetryError> {
    let reading = parse_line(line)?;
    let sensor = reg.lookup(reading.sensor)?;
    let value = sensor.apply(reading.value);
    let (lo, hi) = sensor.bounds();
    if value < lo || value > hi {
        return Err(TelemetryError::OutOfRange { sensor: reading.sensor.to_string(), value });
    }
    Ok((reading.sensor.to_string(), value))
}

#[derive(Default)]
struct Summary {
    count: u32,
    sum: f64,
    max: f64,
}

fn main() {
    let registry = Arc::new(Registry::new(vec![
        Linear { id: "temp".into(), gain: 0.1, offset: -40.0, bounds: (-40.0, 85.0) },
        Linear { id: "humidity".into(), gain: 0.5, offset: 0.0, bounds: (0.0, 100.0) },
        Linear { id: "pressure".into(), gain: 1.0, offset: 900.0, bounds: (900.0, 1100.0) },
    ]));

    let feeds = vec![
        "temp=612\nhumidity=90\npressure=113\ntemp=abc",
        "humidity=250\nwind=4\ntemp=655\npressure 12",
        "pressure=101\ntemp=598\nhumidity=120",
    ];

    let summaries: Arc<Mutex<HashMap<String, Summary>>> = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = mpsc::channel::<String>();

    let handles: Vec<_> = feeds
        .into_iter()
        .enumerate()
        .map(|(n, feed)| {
            let reg = Arc::clone(&registry);
            let out = Arc::clone(&summaries);
            let tx = tx.clone();
            thread::spawn(move || {
                for line in feed.lines() {
                    match calibrate(&reg, line) {
                        Ok((sensor, value)) => {
                            let mut map = out.lock().unwrap();
                            let s = map.entry(sensor).or_default();
                            s.count += 1;
                            s.sum += value;
                            s.max = if s.count == 1 { value } else { s.max.max(value) };
                        }
                        Err(e) => tx.send(format!("feed {n}: {e}")).unwrap(),
                    }
                }
            })
        })
        .collect();
    drop(tx);

    for h in handles {
        h.join().unwrap();
    }
    for problem in rx {
        println!("{problem}");
    }

    let map = summaries.lock().unwrap();
    let mut names: Vec<&String> = map.keys().collect();
    names.sort();
    for name in names {
        let s = &map[name];
        println!("{name:<9} n={} mean={:.2} max={:.2}", s.count, s.sum / s.count as f64, s.max);
    }
}
