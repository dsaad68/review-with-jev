struct Histogram {
    min: f64,
    max: f64,
    counts: Vec<u32>,
    rejected: u32,
}

impl Histogram {
    fn new(min: f64, max: f64, buckets: usize) -> Option<Self> {
        if buckets == 0 || !(min < max) {
            return None;
        }
        Some(Histogram {
            min,
            max,
            counts: vec![0; buckets],
            rejected: 0,
        })
    }

    fn record(&mut self, value: f64) {
        if !(value >= self.min && value < self.max) {
            self.rejected += 1;
            return;
        }
        let frac = (value - self.min) / (self.max - self.min);
        let last = self.counts.len().saturating_sub(1);
        let idx = ((frac * self.counts.len() as f64) as usize).min(last);
        self.counts[idx] += 1;
    }

    fn render(&self) -> String {
        let total: u32 = self.counts.iter().sum();
        let width = (self.max - self.min) / self.counts.len() as f64;
        let mut out = String::new();
        for (i, &c) in self.counts.iter().enumerate() {
            let lo = self.min + width * i as f64;
            let share = if total > 0 {
                f64::from(c) * 100.0 / f64::from(total)
            } else {
                0.0
            };
            out.push_str(&format!(
                "{:>7.1}ms {:<12} {:>5.1}%\n",
                lo,
                "#".repeat(c as usize),
                share
            ));
        }
        out.push_str(&format!("out of range: {}\n", self.rejected));
        out
    }
}

fn main() {
    let latencies = [
        12.5, 48.0, 3.2, 99.9, 150.0, 22.1, 67.3, 71.0, 5.5, 18.8, 44.4, -1.0, 81.2, 33.3,
    ];
    let Some(mut hist) = Histogram::new(0.0, 100.0, 5) else {
        println!("invalid histogram bounds");
        return;
    };
    for v in latencies {
        hist.record(v);
    }
    print!("{}", hist.render());
}
