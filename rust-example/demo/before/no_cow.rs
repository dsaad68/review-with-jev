use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

trait Encoder: Send + 'static {
    fn encode(&self, total: u32, label: &str) -> String;
}

#[derive(Clone)]
struct Hex;

#[derive(Clone)]
struct Rot13 {
    width: usize,
}

impl Encoder for Hex {
    fn encode(&self, total: u32, label: &str) -> String {
        format!("{total:08x}:{}:{}", label.len(), label.to_lowercase())
    }
}

impl Encoder for Rot13 {
    fn encode(&self, total: u32, label: &str) -> String {
        let rotated: String = label
            .chars()
            .map(|c| match c {
                'A'..='Z' => (((c as u8 - b'A' + 13) % 26) + b'A') as char,
                'a'..='z' => (((c as u8 - b'a' + 13) % 26) + b'a') as char,
                _ => c,
            })
            .collect();
        format!("{rotated:>w$}#{total}", w = self.width)
    }
}

fn load(seed: u32, count: u32) -> HashMap<String, Vec<u32>> {
    let raw: String = (0..count)
        .map(|i| format!("user{}:{}\n", (seed + i) % 5, (seed * 31 + i * 7) % 100))
        .collect();
    let mut groups: HashMap<String, Vec<u32>> = HashMap::new();
    for line in raw.lines() {
        if let Some((name, score)) = line.split_once(':') {
            if let Ok(n) = score.parse() {
                groups.entry(name.to_uppercase()).or_default().push(n);
            }
        }
    }
    groups
}

fn spawn_workers<E: Encoder + Clone>(encoder: E, groups: HashMap<String, Vec<u32>>) -> Vec<String> {
    let (tx, rx) = mpsc::channel();
    let handles: Vec<_> = groups
        .into_iter()
        .map(|(name, scores)| {
            let tx = tx.clone();
            let enc = encoder.clone();
            thread::spawn(move || {
                let total: u32 = scores.iter().sum();
                tx.send(enc.encode(total, &name)).unwrap();
            })
        })
        .collect();
    drop(tx);
    for h in handles {
        h.join().unwrap();
    }
    let mut out: Vec<String> = rx.into_iter().collect();
    out.sort();
    out
}

fn main() {
    let hex = spawn_workers(Hex, load(3, 20));
    let rot = spawn_workers(Rot13 { width: 8 }, load(11, 20));
    for (a, b) in hex.iter().zip(&rot) {
        println!("{a:<20} {b}");
    }
}
