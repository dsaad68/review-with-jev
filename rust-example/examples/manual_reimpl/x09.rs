use std::collections::HashMap;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Region {
    North,
    South,
    East,
}

#[derive(Debug)]
enum Message {
    Reading { region: Region, kwh: u32 },
    Fault { region: Region },
    Done,
}

#[derive(Debug)]
struct Tally {
    readings: u32,
    total_kwh: u64,
    peak_kwh: u32,
    faults: u32,
}

impl Tally {
    fn new() -> Self {
        Tally { readings: 0, total_kwh: 0, peak_kwh: 0, faults: 0 }
    }
}

fn meter(region: Region, seed: u32, tx: mpsc::Sender<Message>) {
    let mut x = seed;
    for _ in 0..25 {
        x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        let kwh = (x >> 16) % 500;
        let msg = if kwh > 480 {
            Message::Fault { region }
        } else {
            Message::Reading { region, kwh }
        };
        if tx.send(msg).is_err() {
            return;
        }
    }
    let _ = tx.send(Message::Done);
}

fn collect(rx: mpsc::Receiver<Message>, producers: usize) -> HashMap<Region, Tally> {
    let mut tallies: HashMap<Region, Tally> = HashMap::new();
    let mut finished = 0;
    while finished < producers {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Message::Reading { region, kwh }) => {
                let t = tallies.entry(region).or_insert_with(Tally::new);
                t.readings += 1;
                t.total_kwh += u64::from(kwh);
                t.peak_kwh = t.peak_kwh.max(kwh);
            }
            Ok(Message::Fault { region }) => {
                tallies.entry(region).or_insert_with(Tally::new).faults += 1;
            }
            Ok(Message::Done) => finished += 1,
            Err(RecvTimeoutError::Timeout) => {
                eprintln!("meters silent, stopping with {finished}/{producers} finished");
                break;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    tallies
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let regions = [(Region::North, 7), (Region::South, 91), (Region::East, 4242)];
    let handles: Vec<_> = regions
        .iter()
        .map(|&(region, seed)| {
            let tx = tx.clone();
            thread::spawn(move || meter(region, seed, tx))
        })
        .collect();
    drop(tx);
    let tallies = collect(rx, handles.len());
    for h in handles {
        h.join().expect("meter thread panicked");
    }
    let mut regions: Vec<_> = tallies.into_iter().collect();
    regions.sort_by_key(|(r, _)| *r);
    for (region, t) in &regions {
        let mean = if t.readings == 0 { 0 } else { t.total_kwh / u64::from(t.readings) };
        println!(
            "{:?}: {} readings, mean {} kWh, peak {} kWh, {} faults",
            region, t.readings, mean, t.peak_kwh, t.faults
        );
    }
}
