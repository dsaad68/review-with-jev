use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Low,
    Normal,
    Urgent,
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Checksum,
    WordCount,
    Reverse,
}

#[derive(Debug)]
enum SpecError {
    MissingField(&'static str),
    UnknownKind(String),
    UnknownPriority(String),
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecError::MissingField(field) => write!(f, "missing field `{field}`"),
            SpecError::UnknownKind(k) => write!(f, "unknown job kind `{k}`"),
            SpecError::UnknownPriority(p) => write!(f, "unknown priority `{p}`"),
        }
    }
}

impl FromStr for Priority {
    type Err = SpecError;

    fn from_str(s: &str) -> Result<Self, SpecError> {
        match s {
            "low" => Ok(Priority::Low),
            "normal" => Ok(Priority::Normal),
            "urgent" => Ok(Priority::Urgent),
            other => Err(SpecError::UnknownPriority(other.to_string())),
        }
    }
}

impl FromStr for Kind {
    type Err = SpecError;

    fn from_str(s: &str) -> Result<Self, SpecError> {
        match s {
            "checksum" => Ok(Kind::Checksum),
            "wordcount" => Ok(Kind::WordCount),
            "reverse" => Ok(Kind::Reverse),
            other => Err(SpecError::UnknownKind(other.to_string())),
        }
    }
}

struct Job {
    name: String,
    kind: Kind,
    priority: Priority,
    payload: String,
}

fn parse_job(line: &str) -> Result<Job, SpecError> {
    let mut parts = line.splitn(4, '|').map(str::trim);
    let name = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or(SpecError::MissingField("name"))?;
    let kind = parts.next().ok_or(SpecError::MissingField("kind"))?.parse()?;
    let priority = parts.next().ok_or(SpecError::MissingField("priority"))?.parse()?;
    let payload = parts.next().ok_or(SpecError::MissingField("payload"))?;
    Ok(Job { name: name.to_string(), kind, priority, payload: payload.to_string() })
}

trait Executor {
    fn execute(&self, kind: Kind, payload: &str) -> String;
}

struct Local;

impl Executor for Local {
    fn execute(&self, kind: Kind, payload: &str) -> String {
        match kind {
            Kind::Checksum => {
                let sum = payload
                    .bytes()
                    .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(u32::from(b)));
                format!("{sum:08x}")
            }
            Kind::WordCount => payload.split_whitespace().count().to_string(),
            Kind::Reverse => payload.chars().rev().collect(),
        }
    }
}

fn dispatch<E: Executor + Sync>(
    jobs: Vec<Job>,
    executor: &E,
    workers: usize,
) -> BTreeMap<(Priority, String), String> {
    let workers = workers.max(1);
    let mut buckets: Vec<Vec<Job>> = (0..workers).map(|_| Vec::new()).collect();
    for (i, job) in jobs.into_iter().enumerate() {
        buckets[i % workers].push(job);
    }

    let (tx, rx) = mpsc::channel();
    thread::scope(|scope| {
        for bucket in buckets {
            let tx = tx.clone();
            scope.spawn(move || {
                for job in bucket {
                    let output = executor.execute(job.kind, &job.payload);
                    if tx.send((job.priority, job.name, output)).is_err() {
                        break;
                    }
                }
            });
        }
    });
    drop(tx);

    rx.into_iter().map(|(p, name, out)| ((p, name), out)).collect()
}

fn main() {
    let spec = "\
nightly-sum | checksum | normal | the quick brown fox
greeting | reverse | urgent | hello world
essay | wordcount | low | four score and seven years ago
broken | compress | normal | data
| reverse | low | nameless
audit | checksum | critical | ledger
tagline | reverse | normal | rust is fun";

    let mut jobs = Vec::new();
    for (lineno, line) in spec.lines().enumerate() {
        match parse_job(line) {
            Ok(job) => jobs.push(job),
            Err(e) => eprintln!("line {}: {}", lineno + 1, e),
        }
    }

    let results = dispatch(jobs, &Local, 3);
    for ((priority, name), output) in results.iter().rev() {
        println!("[{priority:?}] {name}: {output}");
    }
}
