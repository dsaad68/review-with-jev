use std::collections::BTreeMap;
use std::fmt::{self, Write};

trait Sensor {
    fn id(&self) -> &str;
    fn unit(&self) -> &'static str;
    fn sample(&mut self, tick: u32) -> f64;
}

#[derive(Clone, Debug)]
struct Reading {
    tick: u32,
    value: f64,
}

struct Thermometer {
    id: String,
    base: f64,
}

impl Sensor for Thermometer {
    fn id(&self) -> &str {
        &self.id
    }
    fn unit(&self) -> &'static str {
        "C"
    }
    fn sample(&mut self, tick: u32) -> f64 {
        self.base + ((tick % 7) as f64 - 3.0) * 0.4
    }
}

struct Hygrometer {
    id: String,
    drift: f64,
    level: f64,
}

impl Sensor for Hygrometer {
    fn id(&self) -> &str {
        &self.id
    }
    fn unit(&self) -> &'static str {
        "%"
    }
    fn sample(&mut self, tick: u32) -> f64 {
        self.level = (self.level + self.drift * if tick % 2 == 0 { 1.0 } else { -0.5 }).clamp(0.0, 100.0);
        self.level
    }
}

struct Channel<S: Sensor> {
    sensor: S,
    readings: Vec<Reading>,
    limit: f64,
}

impl<S: Sensor> Channel<S> {
    fn new(sensor: S, limit: f64) -> Self {
        Channel { sensor, readings: Vec::new(), limit }
    }

    fn poll(&mut self, tick: u32) {
        let value = self.sensor.sample(tick);
        self.readings.push(Reading { tick, value });
    }

    fn stats(&self) -> Option<Stats> {
        let first = self.readings.first()?;
        let mut stats = Stats { min: first.value, max: first.value, sum: 0.0, count: 0, breaches: 0 };
        for r in &self.readings {
            stats.min = stats.min.min(r.value);
            stats.max = stats.max.max(r.value);
            stats.sum += r.value;
            stats.count += 1;
            if r.value > self.limit {
                stats.breaches += 1;
            }
        }
        Some(stats)
    }

    fn report(&self) -> Result<String, fmt::Error> {
        let mut out = String::new();
        writeln!(out, "[{}] limit {:.1}{}", self.sensor.id(), self.limit, self.sensor.unit())?;
        let history = self.readings.clone();
        for r in history.iter().filter(|r| r.value > self.limit) {
            writeln!(out, "  tick {:>3}: {:.2}{}", r.tick, r.value, self.sensor.unit())?;
        }
        if let Some(s) = self.stats() {
            writeln!(
                out,
                "  min {:.2} max {:.2} mean {:.2} breaches {}/{}",
                s.min,
                s.max,
                s.sum / s.count as f64,
                s.breaches,
                s.count
            )?;
        }
        Ok(out)
    }
}

struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: usize,
    breaches: usize,
}

fn run<S: Sensor>(channel: &mut Channel<S>, ticks: u32) {
    for t in 0..ticks {
        channel.poll(t);
    }
}

fn main() {
    let mut temp = Channel::new(Thermometer { id: String::from("greenhouse-t1"), base: 24.0 }, 25.0);
    let mut humid = Channel::new(
        Hygrometer { id: String::from("greenhouse-h1"), drift: 3.5, level: 60.0 },
        75.0,
    );

    run(&mut temp, 20);
    run(&mut humid, 20);

    let mut summary: BTreeMap<&str, usize> = BTreeMap::new();
    for (id, stats) in [
        (temp.sensor.id(), temp.stats()),
        (humid.sensor.id(), humid.stats()),
    ] {
        summary.insert(id, stats.map(|s| s.breaches).unwrap_or(0));
    }

    for text in [temp.report(), humid.report()] {
        match text {
            Ok(t) => print!("{t}"),
            Err(e) => eprintln!("format error: {e}"),
        }
    }
    for (id, n) in &summary {
        println!("{id}: {n} breaches");
    }
}
