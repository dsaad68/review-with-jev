use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::num::ParseIntError;
use std::path::Path;

fn latency_of(line: &str) -> Result<u32, ParseIntError> {
    line.rsplit(' ')
        .next()
        .unwrap_or("")
        .trim_end_matches("ms")
        .parse()
}

fn load_latencies(path: &Path) -> io::Result<Vec<u32>> {
    let file = fs::File::open(path)?;
    let latencies = BufReader::new(file)
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| latency_of(&line).ok())
        .collect();
    Ok(latencies)
}

fn percentile(sorted: &[u32], pct: usize) -> Option<u32> {
    if sorted.is_empty() {
        return None;
    }
    let rank = (sorted.len() - 1) * pct / 100;
    sorted.get(rank).copied()
}

fn main() -> io::Result<()> {
    let path = std::env::temp_dir().join("gateway_access.log");
    {
        let mut out = fs::File::create(&path)?;
        writeln!(out, "GET /index 12ms")?;
        writeln!(out, "GET /search 48ms")?;
        writeln!(out, "POST /checkout timeout")?;
        writeln!(out, "GET /search 51ms")?;
        writeln!(out, "GET /profile 9O0ms")?;
        writeln!(out)?;
        writeln!(out, "GET /index 15ms")?;
        writeln!(out, "DELETE /cart 230ms")?;
    }

    let mut latencies = load_latencies(&path)?;
    latencies.sort_unstable();
    println!("samples: {}", latencies.len());
    for pct in [50, 90, 99] {
        match percentile(&latencies, pct) {
            Some(ms) => println!("p{pct}: {ms}ms"),
            None => println!("p{pct}: n/a"),
        }
    }
    let total: u64 = latencies.iter().map(|&ms| u64::from(ms)).sum();
    if !latencies.is_empty() {
        println!("mean: {}ms", total / latencies.len() as u64);
    }
    let _ = fs::remove_file(&path);
    Ok(())
}
