use std::sync::mpsc;
use std::thread;

trait Filter: Send + Sync {
    fn apply(&self, sample: f32) -> f32;
    fn name(&self) -> &str;
}

struct Gain {
    factor: f32,
}

impl Filter for Gain {
    fn apply(&self, sample: f32) -> f32 {
        sample * self.factor
    }
    fn name(&self) -> &str {
        "gain"
    }
}

struct Clip {
    limit: f32,
}

impl Filter for Clip {
    fn apply(&self, sample: f32) -> f32 {
        sample.clamp(-self.limit, self.limit)
    }
    fn name(&self) -> &str {
        "clip"
    }
}

struct Channel {
    id: usize,
    samples: Vec<f32>,
}

impl Channel {
    fn synth(id: usize, len: usize) -> Self {
        let samples = (0..len)
            .map(|n| ((n as f32) * 0.37 + id as f32).sin() * (1.0 + id as f32 * 0.5))
            .collect();
        Channel { id, samples }
    }

    fn run_chain(&mut self, chain: &[Box<dyn Filter>]) {
        for filter in chain {
            for i in 0..self.samples.len() {
                self.samples[i] = filter.apply(self.samples[i]);
            }
        }
    }

    fn peak(&self) -> f32 {
        self.samples.iter().fold(0.0f32, |acc, s| acc.max(s.abs()))
    }
}

fn mix(left: &[f32], right: &[f32], balance: f32) -> Vec<f32> {
    let n = left.len().min(right.len());
    let mut out = vec![0.0; n];
    for i in 0..n {
        out[i] = left[i] * (1.0 - balance) + right[i] * balance;
    }
    out
}

fn build_chain() -> Vec<Box<dyn Filter>> {
    vec![Box::new(Gain { factor: 1.8 }), Box::new(Clip { limit: 1.5 })]
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();
    for id in 0..4 {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let chain = build_chain();
            let mut channel = Channel::synth(id, 256);
            channel.run_chain(&chain);
            let names: Vec<&str> = chain.iter().map(|f| f.name()).collect();
            println!("channel {} through {}", channel.id, names.join(" > "));
            if tx.send(channel).is_err() {
                eprintln!("collector gone, dropping channel {id}");
            }
        }));
    }
    drop(tx);
    for h in handles {
        h.join().expect("worker panicked");
    }
    let mut channels: Vec<Channel> = rx.iter().collect();
    channels.sort_by_key(|c| c.id);
    for c in &channels {
        println!("channel {} peak {:.3}", c.id, c.peak());
    }
    if let [a, b, ..] = channels.as_slice() {
        let mixed = mix(&a.samples, &b.samples, 0.25);
        let peak = mixed.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
        println!("mix of {} and {} peak {:.3} over {} samples", a.id, b.id, peak, mixed.len());
    }
}
