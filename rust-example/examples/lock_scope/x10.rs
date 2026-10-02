use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

#[derive(Debug, Clone, PartialEq)]
struct RateConfig {
    burst: u32,
    per_window: u32,
    banned: Vec<String>,
}

#[derive(Debug)]
enum ConfigError {
    Io(io::Error),
    MissingField(&'static str),
    Malformed(String),
    BadNumber { field: &'static str, value: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "io error: {err}"),
            ConfigError::MissingField(field) => write!(f, "missing field {field}"),
            ConfigError::Malformed(line) => write!(f, "malformed line {line:?}"),
            ConfigError::BadNumber { field, value } => write!(f, "bad number {value:?} for {field}"),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::Io(err)
    }
}

fn parse_config(text: &str) -> Result<RateConfig, ConfigError> {
    let mut fields: HashMap<&str, &str> = HashMap::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (key, value) = line.split_once('=').ok_or_else(|| ConfigError::Malformed(line.to_string()))?;
        fields.insert(key.trim(), value.trim());
    }
    let number = |field: &'static str| -> Result<u32, ConfigError> {
        let raw = fields.get(field).ok_or(ConfigError::MissingField(field))?;
        raw.parse().map_err(|_| ConfigError::BadNumber { field, value: raw.to_string() })
    };
    let banned = fields
        .get("banned")
        .map(|list| list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    Ok(RateConfig { burst: number("burst")?, per_window: number("per_window")?, banned })
}

struct Limiter {
    config: RwLock<Arc<RateConfig>>,
    usage: Mutex<HashMap<String, u32>>,
    source: PathBuf,
}

impl Limiter {
    fn load(source: &Path) -> Result<Self, ConfigError> {
        let config = parse_config(&fs::read_to_string(source)?)?;
        Ok(Limiter {
            config: RwLock::new(Arc::new(config)),
            usage: Mutex::new(HashMap::new()),
            source: source.to_path_buf(),
        })
    }

    fn reload(&self) -> Result<bool, ConfigError> {
        let fresh = Arc::new(parse_config(&fs::read_to_string(&self.source)?)?);
        let mut current = self.config.write().unwrap();
        let changed = **current != *fresh;
        *current = fresh;
        Ok(changed)
    }

    fn admit(&self, client: &str) -> bool {
        let config = Arc::clone(&self.config.read().unwrap());
        if config.banned.iter().any(|b| b == client) {
            return false;
        }
        let limit = config.burst.saturating_add(config.per_window);
        let mut usage = self.usage.lock().unwrap();
        let used = usage.entry(client.to_string()).or_insert(0);
        if *used < limit {
            *used += 1;
            true
        } else {
            false
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("limiter failed: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ConfigError> {
    let path = std::env::temp_dir().join("limiter_x10.conf");
    fs::write(&path, "burst = 3\nper_window = 2\nbanned = mallory\n")?;
    let limiter = Arc::new(Limiter::load(&path)?);

    let (tx, rx) = mpsc::channel();
    let mut clients = Vec::new();
    for name in ["alice", "bob", "mallory"] {
        let limiter = Arc::clone(&limiter);
        let tx = tx.clone();
        clients.push(thread::spawn(move || {
            for attempt in 0..8 {
                let admitted = limiter.admit(name);
                if tx.send((name, attempt, admitted)).is_err() {
                    return;
                }
            }
        }));
    }
    drop(tx);
    for client in clients {
        client.join().unwrap();
    }

    let mut admitted: HashMap<&str, u32> = HashMap::new();
    for (name, _, ok) in rx {
        if ok {
            *admitted.entry(name).or_insert(0) += 1;
        }
    }
    let mut summary: Vec<_> = admitted.into_iter().collect();
    summary.sort();
    println!("admitted: {summary:?}");

    fs::write(&path, "burst = 10\nper_window = 5\nbanned =\n")?;
    println!("config changed: {}", limiter.reload()?);
    println!("mallory now admitted: {}", limiter.admit("mallory"));
    fs::remove_file(&path)?;
    Ok(())
}
