use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Default)]
struct Metrics {
    counters: RwLock<BTreeMap<&'static str, u64>>,
}

impl Metrics {
    fn bump(&self, name: &'static str, by: u64) {
        *self.counters.write().unwrap().entry(name).or_insert(0) += by;
    }

    fn export(&self, path: &Path) -> io::Result<usize> {
        let guard = self.counters.read().unwrap();
        let snapshot: Vec<(&'static str, u64)> = guard.iter().map(|(k, v)| (*k, *v)).collect();
        drop(guard);
        let mut body = String::new();
        for (name, value) in &snapshot {
            let _ = writeln!(body, "{name} {value}");
        }
        fs::write(path, body)?;
        Ok(snapshot.len())
    }
}

fn main() -> io::Result<()> {
    let metrics = Arc::new(Metrics::default());
    let mut writers = Vec::new();
    for worker in 0..4u64 {
        let metrics = Arc::clone(&metrics);
        writers.push(thread::spawn(move || {
            for _ in 0..100 {
                metrics.bump("requests", 1);
                metrics.bump("bytes_in", 512 + worker);
            }
        }));
    }
    let path = std::env::temp_dir().join("metrics_x07.txt");
    let early = metrics.export(&path)?;
    for writer in writers {
        writer.join().unwrap();
    }
    let late = metrics.export(&path)?;
    println!("exported {early} then {late} series");
    print!("{}", fs::read_to_string(&path)?);
    fs::remove_file(path)
}
