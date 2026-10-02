use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Severity {
    Debug,
    Info,
    Warn,
    Error,
}

impl Severity {
    fn parse(tag: &str) -> Option<Self> {
        match tag {
            "D" | "DEBUG" => Some(Severity::Debug),
            "I" | "INFO" => Some(Severity::Info),
            "W" | "WARN" => Some(Severity::Warn),
            "E" | "ERROR" => Some(Severity::Error),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct LogRecord {
    service: String,
    severity: Severity,
    message: String,
    latency_ms: Option<u32>,
}

trait RecordSource {
    type Item;
    fn next_batch(&mut self, max: usize) -> Vec<Self::Item>;
}

struct LineSource<'a> {
    lines: std::slice::Iter<'a, &'a str>,
    rejected: usize,
}

impl<'a> RecordSource for LineSource<'a> {
    type Item = LogRecord;

    fn next_batch(&mut self, max: usize) -> Vec<LogRecord> {
        let mut out = Vec::with_capacity(max);
        while out.len() < max {
            match self.lines.next() {
                Some(line) => match parse_line(line) {
                    Some(record) => out.push(record),
                    None => self.rejected += 1,
                },
                None => break,
            }
        }
        out
    }
}

fn service_key(raw: &str) -> String {
    raw.trim_matches(|c| c == '[' || c == ']').to_ascii_lowercase()
}

fn parse_line(line: &str) -> Option<LogRecord> {
    let mut parts = line.splitn(4, ' ');
    let _timestamp = parts.next()?;
    let service = service_key(parts.next()?);
    let severity = Severity::parse(parts.next()?)?;
    let rest = parts.next().unwrap_or("");
    let (latency_ms, message) = match rest.strip_prefix("latency=") {
        Some(tail) => {
            let (num, msg) = tail.split_once(' ').unwrap_or((tail, ""));
            (num.parse().ok(), msg.to_string())
        }
        None => (None, rest.to_string()),
    };
    Some(LogRecord { service, severity, message, latency_ms })
}

#[derive(Default, Debug)]
struct ServiceStats {
    counts: HashMap<Severity, usize>,
    total_latency: u64,
    samples: u64,
    last_error: Option<String>,
}

type SharedStats = Arc<Mutex<HashMap<String, ServiceStats>>>;

fn shard_for(service: &str, shards: usize) -> usize {
    service.bytes().fold(0usize, |acc, b| acc.wrapping_mul(31).wrapping_add(b as usize)) % shards
}

fn spawn_workers(n: usize, stats: SharedStats) -> (Vec<Sender<LogRecord>>, Vec<thread::JoinHandle<usize>>) {
    let mut senders = Vec::with_capacity(n);
    let mut handles = Vec::with_capacity(n);
    for _ in 0..n {
        let (tx, rx) = mpsc::channel::<LogRecord>();
        let stats = Arc::clone(&stats);
        handles.push(thread::spawn(move || {
            let mut processed = 0;
            for record in rx {
                let mut guard = stats.lock().unwrap();
                let entry = guard.entry(record.service).or_default();
                *entry.counts.entry(record.severity).or_insert(0) += 1;
                if let Some(ms) = record.latency_ms {
                    entry.total_latency += ms as u64;
                    entry.samples += 1;
                }
                if record.severity == Severity::Error {
                    entry.last_error = Some(record.message);
                }
                processed += 1;
            }
            processed
        }));
        senders.push(tx);
    }
    (senders, handles)
}

fn main() {
    let raw: Vec<&str> = vec![
        "2024-05-01T10:00:00 [API] INFO latency=120 GET /users",
        "2024-05-01T10:00:01 [Billing] WARN latency=480 slow charge",
        "2024-05-01T10:00:02 [api] ERROR upstream timeout",
        "2024-05-01T10:00:03 [auth] DEBUG token refreshed",
        "garbage line",
        "2024-05-01T10:00:04 [billing] ERROR latency=900 card declined",
        "2024-05-01T10:00:05 [Auth] INFO latency=15 login ok",
        "2024-05-01T10:00:06 [api] INFO latency=95 GET /orders",
    ];
    let mut source = LineSource { lines: raw.iter(), rejected: 0 };
    let stats: SharedStats = Arc::new(Mutex::new(HashMap::new()));
    let (senders, handles) = spawn_workers(3, Arc::clone(&stats));

    loop {
        let batch = source.next_batch(3);
        if batch.is_empty() {
            break;
        }
        for record in batch {
            let shard = shard_for(&record.service, senders.len());
            senders[shard].send(record).expect("worker hung up");
        }
    }
    drop(senders);

    let processed: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    println!("processed={} rejected={}", processed, source.rejected);

    let guard = stats.lock().unwrap();
    let mut services: Vec<_> = guard.iter().collect();
    services.sort_by(|a, b| a.0.cmp(b.0));
    for (name, s) in services {
        let mut counts: Vec<_> = s.counts.iter().collect();
        counts.sort();
        let avg = if s.samples > 0 { s.total_latency / s.samples } else { 0 };
        println!(
            "{:<8} avg_latency={:>4}ms counts={:?} last_error={:?}",
            name, avg, counts, s.last_error
        );
    }
}
