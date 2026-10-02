use std::collections::BTreeMap;
use std::sync::mpsc;
use std::thread;

trait Sample: Copy + Send + 'static {
    fn value(self) -> f64;
}

#[derive(Debug, Clone, Copy)]
struct Reading {
    sensor: u16,
    millivolts: f64,
}

impl Sample for Reading {
    fn value(self) -> f64 {
        self.millivolts
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Stats {
    n: u64,
    mean: f64,
    m2: f64,
    min: f64,
    max: f64,
}

impl Stats {
    fn push(&mut self, x: f64) {
        if self.n == 0 {
            self.min = x;
            self.max = x;
        } else {
            self.min = self.min.min(x);
            self.max = self.max.max(x);
        }
        self.n += 1;
        let delta = x - self.mean;
        self.mean += delta / self.n as f64;
        self.m2 += delta * (x - self.mean);
    }

    fn merge(self, other: Stats) -> Stats {
        if self.n == 0 {
            return other;
        }
        if other.n == 0 {
            return self;
        }
        let n = self.n + other.n;
        let delta = other.mean - self.mean;
        let mean = self.mean + delta * other.n as f64 / n as f64;
        let m2 = self.m2 + other.m2 + delta * delta * self.n as f64 * other.n as f64 / n as f64;
        Stats { n, mean, m2, min: self.min.min(other.min), max: self.max.max(other.max) }
    }

    fn std_dev(&self) -> Option<f64> {
        (self.n > 1).then(|| (self.m2 / (self.n - 1) as f64).sqrt())
    }
}

fn approx_eq(a: f64, b: f64, rel: f64) -> bool {
    let scale = a.abs().max(b.abs()).max(1.0);
    (a - b).abs() <= rel * scale
}

fn summarise<S: Sample>(batch: &[S]) -> Stats {
    let mut s = Stats::default();
    for x in batch {
        s.push(x.value());
    }
    s
}

fn synthetic(sensor: u16, count: u32) -> Vec<Reading> {
    (0..count)
        .map(|i| {
            let phase = f64::from(i) * 0.37 + f64::from(sensor);
            Reading { sensor, millivolts: 1200.0 + 15.0 * phase.sin() + f64::from(sensor) * 40.0 }
        })
        .collect()
}

fn main() {
    let sensors: [u16; 3] = [1, 2, 3];
    let (tx, rx) = mpsc::channel::<(u16, Stats)>();

    let workers: Vec<_> = sensors
        .iter()
        .map(|&sensor| {
            let tx = tx.clone();
            thread::spawn(move || {
                let data = synthetic(sensor, 5_000);
                for chunk in data.chunks(1_000) {
                    if tx.send((chunk[0].sensor, summarise(chunk))).is_err() {
                        return;
                    }
                }
            })
        })
        .collect();
    drop(tx);

    let mut per_sensor: BTreeMap<u16, Stats> = BTreeMap::new();
    for (sensor, partial) in rx {
        let slot = per_sensor.entry(sensor).or_default();
        *slot = slot.merge(partial);
    }
    for w in workers {
        w.join().expect("worker panicked");
    }

    for (sensor, stats) in &per_sensor {
        let direct = summarise(&synthetic(*sensor, 5_000));
        let consistent = approx_eq(stats.mean, direct.mean, 1e-9)
            && approx_eq(stats.std_dev().unwrap_or(0.0), direct.std_dev().unwrap_or(0.0), 1e-6);
        println!(
            "sensor {sensor}: n={} mean={:.3} sd={:.3} range=[{:.2}, {:.2}] merged-matches-direct={consistent}",
            stats.n,
            stats.mean,
            stats.std_dev().unwrap_or(0.0),
            stats.min,
            stats.max
        );
        if stats.max - stats.min > 25.0 {
            println!("  sensor {sensor} swing exceeds 25 mV");
        }
    }
}
