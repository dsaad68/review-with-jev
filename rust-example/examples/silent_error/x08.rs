use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

trait Metered {
    fn remaining(&self) -> u32;
    fn exhausted(&self) -> bool {
        self.remaining() == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Quota {
    used: u32,
    limit: u32,
}

#[derive(Debug)]
enum QuotaParseError {
    Shape,
    Number(std::num::ParseIntError),
}

impl FromStr for Quota {
    type Err = QuotaParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (used, limit) = s.split_once('/').ok_or(QuotaParseError::Shape)?;
        Ok(Quota {
            used: used.trim().parse().map_err(QuotaParseError::Number)?,
            limit: limit.trim().parse().map_err(QuotaParseError::Number)?,
        })
    }
}

impl fmt::Display for Quota {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.used, self.limit)
    }
}

impl Metered for Quota {
    fn remaining(&self) -> u32 {
        self.limit.saturating_sub(self.used)
    }
}

struct Store<T> {
    entries: BTreeMap<String, T>,
    dir: PathBuf,
    dirty: bool,
}

impl<T: FromStr + fmt::Display> Store<T> {
    fn open(dir: &Path) -> Self {
        let mut entries = BTreeMap::new();
        if let Ok(text) = fs::read_to_string(dir.join("snapshot.db")) {
            for line in text.lines() {
                if let Some((key, raw)) = line.split_once('\t') {
                    if let Ok(value) = raw.parse::<T>() {
                        entries.insert(key.to_string(), value);
                    }
                }
            }
        }
        Store { entries, dir: dir.to_path_buf(), dirty: false }
    }

    fn put(&mut self, key: &str, value: T) {
        self.entries.insert(key.to_string(), value);
        self.dirty = true;
    }

    fn get(&self, key: &str) -> Option<&T> {
        self.entries.get(key)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn flush(&mut self) {
        if !self.dirty {
            return;
        }
        let _ = fs::create_dir_all(&self.dir);
        let staging = self.dir.join("snapshot.tmp");
        if let Ok(mut file) = fs::File::create(&staging) {
            for (key, value) in &self.entries {
                let _ = writeln!(file, "{key}\t{value}");
            }
        }
        let _ = fs::rename(&staging, self.dir.join("snapshot.db"));
        self.dirty = false;
    }
}

impl<T: Metered> Store<T> {
    fn exhausted_keys(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|(_, v)| v.exhausted())
            .map(|(k, _)| k.as_str())
            .collect()
    }
}

fn main() -> io::Result<()> {
    let dir = std::env::temp_dir().join("tenant_quota_store");
    let _ = fs::remove_dir_all(&dir);

    let mut store: Store<Quota> = Store::open(&dir);
    store.put("acme", Quota { used: 120, limit: 500 });
    store.put("globex", Quota { used: 500, limit: 500 });
    store.put("initech", Quota { used: 37, limit: 100 });
    store.flush();

    let mut extra = fs::OpenOptions::new()
        .append(true)
        .open(dir.join("snapshot.db"))?;
    writeln!(extra, "umbrella\t90 of 100")?;
    writeln!(extra, "hooli\t-5/100")?;
    writeln!(extra, "stark 10/20")?;
    drop(extra);

    let reopened: Store<Quota> = Store::open(&dir);
    println!("tenants loaded: {}", reopened.len());
    for name in ["acme", "globex", "umbrella", "hooli"] {
        match reopened.get(name) {
            Some(q) => println!("{name}: {q} ({} left)", q.remaining()),
            None => println!("{name}: no quota on record"),
        }
    }
    println!("exhausted: {:?}", reopened.exhausted_keys());

    let _ = fs::remove_dir_all(&dir);
    Ok(())
}
