use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

const ENDPOINTS: [&str; 4] = ["/api/orders", "/api/users", "/static/app.js", "/health"];

struct Request {
    endpoint: &'static str,
    bytes: u64,
    latency_ms: u32,
}

#[derive(Default, Clone, Copy)]
struct Totals {
    bytes: u64,
    latency_ms: u64,
    count: u64,
}

impl Totals {
    fn add(&mut self, r: &Request) {
        self.bytes += r.bytes;
        self.latency_ms += u64::from(r.latency_ms);
        self.count += 1;
    }

    fn merge(&mut self, other: &Totals) {
        self.bytes += other.bytes;
        self.latency_ms += other.latency_ms;
        self.count += other.count;
    }
}

#[derive(Debug)]
struct Summary {
    endpoint: String,
    requests: u32,
    total_kb: u32,
    avg_latency_ms: u16,
}

trait Report {
    fn summarize(&self) -> Vec<Summary>;
}

impl Report for HashMap<String, Totals> {
    fn summarize(&self) -> Vec<Summary> {
        let mut out: Vec<Summary> = self
            .iter()
            .map(|(endpoint, t)| Summary {
                endpoint: endpoint.clone(),
                requests: t.count as u32,
                total_kb: (t.bytes / 1024) as u32,
                avg_latency_ms: if t.count == 0 {
                    0
                } else {
                    (t.latency_ms / t.count) as u16
                },
            })
            .collect();
        out.sort_by(|a, b| {
            b.total_kb
                .cmp(&a.total_kb)
                .then_with(|| a.endpoint.cmp(&b.endpoint))
        });
        out
    }
}

fn synthetic_shard(shard: u64, size: u64) -> Vec<Request> {
    (0..size)
        .map(|i| {
            let n = shard * 1_000 + i;
            Request {
                endpoint: ENDPOINTS[(n % 4) as usize],
                bytes: (n * 7_919) % 48_000 + 200,
                latency_ms: ((n * 31) % 900 + 3) as u32,
            }
        })
        .collect()
}

fn aggregate(shards: Vec<Vec<Request>>) -> HashMap<String, Totals> {
    let merged: Arc<Mutex<HashMap<String, Totals>>> = Arc::new(Mutex::new(HashMap::new()));
    let handles: Vec<_> = shards
        .into_iter()
        .map(|shard| {
            let merged = Arc::clone(&merged);
            thread::spawn(move || {
                let mut local: HashMap<&'static str, Totals> = HashMap::new();
                for r in &shard {
                    local.entry(r.endpoint).or_default().add(r);
                }
                let mut guard = merged.lock().unwrap();
                for (endpoint, t) in local {
                    guard.entry(endpoint.to_string()).or_default().merge(&t);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    Arc::try_unwrap(merged)
        .map(|m| m.into_inner().unwrap())
        .unwrap_or_default()
}

fn main() {
    let shards: Vec<Vec<Request>> = (0..4).map(|s| synthetic_shard(s, 250)).collect();
    let totals = aggregate(shards);
    println!("{:<16} {:>8} {:>10} {:>8}", "endpoint", "requests", "kb", "avg ms");
    for s in totals.summarize() {
        println!(
            "{:<16} {:>8} {:>10} {:>8}",
            s.endpoint, s.requests, s.total_kb, s.avg_latency_ms
        );
    }
}
