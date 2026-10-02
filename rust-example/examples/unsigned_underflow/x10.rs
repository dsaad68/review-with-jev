use std::fmt;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

mod range {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct ByteRange {
        start: usize,
        end: usize,
    }

    #[derive(Debug, PartialEq)]
    pub struct Inverted {
        pub start: usize,
        pub end: usize,
    }

    impl ByteRange {
        pub fn new(start: usize, end: usize) -> Result<Self, Inverted> {
            if end < start {
                return Err(Inverted { start, end });
            }
            Ok(ByteRange { start, end })
        }

        pub fn start(&self) -> usize {
            self.start
        }

        pub fn end(&self) -> usize {
            self.end
        }

        pub fn len(&self) -> usize {
            self.end - self.start
        }
    }
}

use range::{ByteRange, Inverted};

impl fmt::Display for Inverted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "range end {} is before start {}", self.end, self.start)
    }
}

trait Source: Send + Sync {
    fn total_len(&self) -> usize;
    fn read_range(&self, range: ByteRange) -> Option<Vec<u8>>;
}

struct MemorySource {
    data: Vec<u8>,
}

impl Source for MemorySource {
    fn total_len(&self) -> usize {
        self.data.len()
    }

    fn read_range(&self, range: ByteRange) -> Option<Vec<u8>> {
        self.data.get(range.start()..range.end()).map(<[u8]>::to_vec)
    }
}

fn split(total: usize, already: usize, parts: usize) -> Vec<ByteRange> {
    if already >= total || parts == 0 {
        return Vec::new();
    }
    let pending = total - already;
    let chunk = pending.div_ceil(parts);
    (0..parts)
        .map(|i| already + i * chunk)
        .take_while(|&s| s < total)
        .filter_map(|s| ByteRange::new(s, (s + chunk).min(total)).ok())
        .collect()
}

enum Event {
    Done(ByteRange, Vec<u8>),
    Failed(ByteRange),
}

fn download<S: Source + 'static>(source: Arc<S>, already: &[u8], parts: usize) -> Result<Vec<u8>, ByteRange> {
    let total = source.total_len();
    let ranges = split(total, already.len(), parts);
    let (tx, rx) = mpsc::channel();
    for range in ranges.iter().copied() {
        let tx = tx.clone();
        let source = Arc::clone(&source);
        thread::spawn(move || {
            let event = match source.read_range(range) {
                Some(bytes) => Event::Done(range, bytes),
                None => Event::Failed(range),
            };
            tx.send(event).ok();
        });
    }
    drop(tx);
    let mut buffer = vec![0u8; total];
    let prefix = already.len().min(total);
    buffer[..prefix].copy_from_slice(&already[..prefix]);
    let mut received = prefix;
    for event in rx {
        match event {
            Event::Done(range, bytes) => {
                buffer[range.start()..range.end()].copy_from_slice(&bytes);
                received += range.len();
                println!(
                    "got {:>4}..{:<4} remaining {}",
                    range.start(),
                    range.end(),
                    total.saturating_sub(received)
                );
            }
            Event::Failed(range) => return Err(range),
        }
    }
    Ok(buffer)
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |acc, &b| acc.rotate_left(5) ^ u32::from(b))
}

fn main() {
    let data: Vec<u8> = (0..=255u8).cycle().take(1_000).collect();
    let expected = checksum(&data);
    let source = Arc::new(MemorySource { data });
    let resumed = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    match download(Arc::clone(&source), &resumed, 4) {
        Ok(bytes) => println!("checksum ok: {}", checksum(&bytes) == expected),
        Err(r) => println!("failed on {r:?}"),
    }
    println!("{:?}", split(100, 100, 4));
    println!("{:?}", split(10, 0, 4));
    match ByteRange::new(40, 10) {
        Ok(r) => println!("unexpected range of len {}", r.len()),
        Err(e) => println!("rejected: {e}"),
    }
}
