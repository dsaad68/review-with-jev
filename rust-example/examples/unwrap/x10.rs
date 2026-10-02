use std::collections::HashMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug)]
enum StoreError {
    Io(io::Error),
    Corrupt { line: usize, text: String },
}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "io: {e}"),
            StoreError::Corrupt { line, text } => write!(f, "corrupt record at line {line}: {text}"),
        }
    }
}

trait Codec: Sized {
    fn encode(&self) -> String;
    fn decode(s: &str) -> Option<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Counter {
    hits: u64,
    misses: u64,
}

impl Codec for Counter {
    fn encode(&self) -> String {
        format!("{}/{}", self.hits, self.misses)
    }
    fn decode(s: &str) -> Option<Self> {
        let (h, m) = s.split_once('/')?;
        Some(Counter { hits: h.parse().ok()?, misses: m.parse().ok()? })
    }
}

enum Op<V> {
    Put(String, V),
    Delete(String),
}

struct LogStore<V: Codec> {
    path: PathBuf,
    data: HashMap<String, V>,
}

impl<V: Codec> LogStore<V> {
    fn open(path: &Path) -> Result<Self, StoreError> {
        let mut data = HashMap::new();
        if path.exists() {
            let reader = BufReader::new(File::open(path)?);
            for (idx, line) in reader.lines().enumerate() {
                let line = line?;
                let mut parts = line.splitn(3, '\t');
                let ok = match (parts.next(), parts.next(), parts.next()) {
                    (Some("P"), Some(key), Some(value)) => match V::decode(value) {
                        Some(v) => {
                            data.insert(key.to_string(), v);
                            true
                        }
                        None => false,
                    },
                    (Some("D"), Some(key), None) => {
                        data.remove(key);
                        true
                    }
                    _ => false,
                };
                if !ok {
                    return Err(StoreError::Corrupt { line: idx + 1, text: line });
                }
            }
        }
        Ok(LogStore { path: path.to_path_buf(), data })
    }

    fn apply(&mut self, ops: Vec<Op<V>>) -> Result<(), StoreError> {
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        for op in ops {
            match op {
                Op::Put(k, v) => {
                    writeln!(file, "P\t{k}\t{}", v.encode())?;
                    self.data.insert(k, v);
                }
                Op::Delete(k) => {
                    writeln!(file, "D\t{k}")?;
                    self.data.remove(&k);
                }
            }
        }
        file.flush()?;
        Ok(())
    }

    fn get(&self, key: &str) -> Option<&V> {
        self.data.get(key)
    }
}

fn record_batch(store: &Mutex<LogStore<Counter>>, worker: u64) -> Result<(), StoreError> {
    let mut guard = store.lock().unwrap();
    let key = format!("route{}", worker % 3);
    let current = guard.get(&key).copied().unwrap_or(Counter { hits: 0, misses: 0 });
    let next = Counter { hits: current.hits + worker, misses: current.misses + 1 };
    let mut ops = vec![Op::Put(key, next)];
    if worker == 4 {
        ops.push(Op::Delete("route0".to_string()));
    }
    guard.apply(ops)
}

fn run(path: &Path) -> Result<(), StoreError> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    let store = Arc::new(Mutex::new(LogStore::<Counter>::open(path)?));
    let handles: Vec<_> = (1..=6)
        .map(|w| {
            let s = Arc::clone(&store);
            thread::spawn(move || record_batch(&s, w))
        })
        .collect();
    for h in handles {
        h.join().unwrap()?;
    }

    let reopened = LogStore::<Counter>::open(path)?;
    let mut keys: Vec<&String> = reopened.data.keys().collect();
    keys.sort();
    for k in keys {
        println!("{k}: {:?}", reopened.data[k]);
    }

    let mut f = OpenOptions::new().append(true).open(path)?;
    writeln!(f, "P\troute9\tnot-a-counter")?;
    match LogStore::<Counter>::open(path) {
        Ok(_) => println!("unexpectedly clean"),
        Err(e) => println!("reopen failed as expected: {e}"),
    }
    Ok(())
}

fn main() {
    let path = std::env::temp_dir().join("x10_store.log");
    if let Err(e) = run(&path) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
