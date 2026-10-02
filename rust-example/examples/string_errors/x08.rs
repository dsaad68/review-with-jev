use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug)]
enum StoreError {
    NotFound(String),
    Conflict { expected: u64, actual: u64 },
    TooLarge { size: usize, max: usize },
    Corrupt(String),
    Io(io::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::NotFound(key) => write!(f, "no object named {key:?}"),
            StoreError::Conflict { expected, actual } => {
                write!(f, "version conflict: expected {expected}, found {actual}")
            }
            StoreError::TooLarge { size, max } => write!(f, "object of {size} bytes exceeds {max}"),
            StoreError::Corrupt(key) => write!(f, "object {key:?} has an unexpected layout"),
            StoreError::Io(e) => write!(f, "storage I/O failed: {e}"),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StoreError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        StoreError::Io(e)
    }
}

trait Backend {
    fn read(&self, key: &str) -> Result<(u64, Vec<u8>), StoreError>;
    fn write(&mut self, key: &str, expected: u64, body: Vec<u8>) -> Result<u64, StoreError>;
}

struct Memory {
    objects: HashMap<String, (u64, Vec<u8>)>,
    max_size: usize,
}

impl Backend for Memory {
    fn read(&self, key: &str) -> Result<(u64, Vec<u8>), StoreError> {
        self.objects
            .get(key)
            .cloned()
            .ok_or_else(|| StoreError::NotFound(key.to_string()))
    }

    fn write(&mut self, key: &str, expected: u64, body: Vec<u8>) -> Result<u64, StoreError> {
        if body.len() > self.max_size {
            return Err(StoreError::TooLarge { size: body.len(), max: self.max_size });
        }
        let slot = self.objects.entry(key.to_string()).or_insert((0, Vec::new()));
        if slot.0 != expected {
            return Err(StoreError::Conflict { expected, actual: slot.0 });
        }
        *slot = (expected + 1, body);
        Ok(slot.0)
    }
}

struct Disk {
    root: PathBuf,
}

impl Backend for Disk {
    fn read(&self, key: &str) -> Result<(u64, Vec<u8>), StoreError> {
        match fs::read(self.root.join(key)) {
            Ok(body) => Ok((1, body)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(StoreError::NotFound(key.to_string())),
            Err(e) => Err(StoreError::Io(e)),
        }
    }

    fn write(&mut self, key: &str, _expected: u64, body: Vec<u8>) -> Result<u64, StoreError> {
        fs::create_dir_all(&self.root)?;
        fs::write(self.root.join(key), body)?;
        Ok(1)
    }
}

fn increment<B: Backend>(store: &Mutex<B>, key: &str) -> Result<u32, StoreError> {
    loop {
        let fetched = store.lock().expect("store poisoned").read(key);
        let (version, body) = match fetched {
            Ok(found) => found,
            Err(StoreError::NotFound(_)) => (0, Vec::new()),
            Err(e) => return Err(e),
        };
        let current = match <[u8; 4]>::try_from(body.as_slice()) {
            Ok(bytes) => u32::from_le_bytes(bytes),
            Err(_) if body.is_empty() => 0,
            Err(_) => return Err(StoreError::Corrupt(key.to_string())),
        };
        let next = current + 1;
        let outcome = store.lock().expect("store poisoned").write(key, version, next.to_le_bytes().to_vec());
        match outcome {
            Ok(_) => return Ok(next),
            Err(StoreError::Conflict { .. }) => thread::yield_now(),
            Err(e) => return Err(e),
        }
    }
}

fn main() -> Result<(), StoreError> {
    let store = Arc::new(Mutex::new(Memory { objects: HashMap::new(), max_size: 16 }));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let store = Arc::clone(&store);
            thread::spawn(move || -> Result<(), StoreError> {
                for _ in 0..250 {
                    increment(&store, "page-views")?;
                }
                Ok(())
            })
        })
        .collect();
    for worker in workers {
        worker.join().expect("worker panicked")?;
    }
    let (version, body) = store.lock().expect("store poisoned").read("page-views")?;
    println!("page-views at version {version}, {} bytes", body.len());

    let banner = store.lock().expect("store poisoned").write("banner", 0, vec![0; 64]);
    match banner {
        Err(StoreError::TooLarge { size, max }) => println!("banner rejected: {size} > {max}"),
        other => println!("unexpected: {other:?}"),
    }

    let mut disk = Disk { root: std::env::temp_dir().join("typed_object_store") };
    disk.write("motd", 0, b"maintenance at noon".to_vec())?;
    let (_, motd) = disk.read("motd")?;
    println!("motd: {}", String::from_utf8_lossy(&motd));
    match disk.read("absent") {
        Err(StoreError::NotFound(key)) => println!("{key} not stored yet"),
        other => println!("unexpected: {other:?}"),
    }
    fs::remove_dir_all(&disk.root)?;
    Ok(())
}
