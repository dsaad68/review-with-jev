#[derive(Debug)]
struct Stats {
    mean: f64,
    stddev: f64,
    outliers: usize,
}

fn parse_latencies(log: &str) -> Vec<f64> {
    log.lines()
        .filter_map(|line| line.rsplit_once("latency="))
        .filter_map(|(_, value)| value.trim_end_matches("ms").parse::<f64>().ok())
        .filter(|ms| *ms >= 0.0)
        .collect()
}

fn analyze(log: &str) -> Option<Stats> {
    let samples = parse_latencies(log);
    if samples.is_empty() {
        return None;
    }

    let n = samples.len() as f64;
    let mean = samples.iter().sum::<f64>() / n;
    let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let stddev = variance.sqrt();
    let outliers = samples
        .iter()
        .filter(|x| (*x - mean).abs() > 2.0 * stddev)
        .count();

    Some(Stats { mean, stddev, outliers })
}

fn main() {
    let log = "\
GET /api/users status=200 latency=12ms
GET /api/users status=200 latency=15ms
POST /api/login status=401 latency=9ms
GET /api/feed status=200 latency=14ms
GET /api/feed status=200 latency=11ms
GET /api/feed status=200 latency=13ms
GET /api/report status=200 latency=480ms
GET /health status=200
GET /api/users status=200 latency=10ms
GET /api/users status=200 latency=16ms";

    match analyze(log) {
        Some(stats) => println!(
            "mean {:.1}ms, stddev {:.1}ms, {} outliers",
            stats.mean, stats.stddev, stats.outliers
        ),
        None => println!("no samples"),
    }
}
