use std::collections::HashMap;
use std::str::FromStr;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Level {
    Info,
    Warn,
    Error,
}

impl FromStr for Level {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INFO" => Ok(Level::Info),
            "WARN" => Ok(Level::Warn),
            "ERROR" => Ok(Level::Error),
            other => Err(format!("unknown level {}", other)),
        }
    }
}

struct Entry {
    ts: u64,
    level: Level,
    user: String,
    message: String,
}

fn parse_line(line: &str) -> Result<Entry, String> {
    let mut parts = line.splitn(4, ' ');
    let ts = parts
        .next()
        .ok_or("missing timestamp")?
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    let level = parts.next().ok_or("missing level")?.parse::<Level>()?;
    let user = parts.next().ok_or("missing user")?;
    let message = parts.next().unwrap_or("");
    Ok(Entry {
        ts,
        level,
        user: user.to_string(),
        message: message.to_string(),
    })
}

trait Sink {
    fn accept(&mut self, entry: &Entry);
    fn summary(&self) -> String;
}

#[derive(Default)]
struct LevelCounts {
    counts: HashMap<Level, usize>,
    longest: usize,
}

impl Sink for LevelCounts {
    fn accept(&mut self, entry: &Entry) {
        *self.counts.entry(entry.level).or_insert(0) += 1;
        self.longest = self.longest.max(entry.message.len());
    }
    fn summary(&self) -> String {
        let get = |l: Level| self.counts.get(&l).copied().unwrap_or(0);
        format!(
            "info={} warn={} error={} longest_message={}",
            get(Level::Info),
            get(Level::Warn),
            get(Level::Error),
            self.longest
        )
    }
}

#[derive(Default)]
struct ActiveUsers {
    users: Vec<String>,
    first_seen: Vec<u64>,
}

impl Sink for ActiveUsers {
    fn accept(&mut self, entry: &Entry) {
        if !self.users.iter().any(|u| *u == entry.user) {
            self.users.push(entry.user.clone());
            self.first_seen.push(entry.ts);
        }
    }
    fn summary(&self) -> String {
        let latest = self.first_seen.iter().max().copied().unwrap_or(0);
        format!("distinct users={} last newcomer at ts={}", self.users.len(), latest)
    }
}

fn parse_parallel(lines: Vec<String>, workers: usize) -> (Vec<Entry>, Vec<String>) {
    let chunk_size = lines.len().div_ceil(workers.max(1)).max(1);
    let mut handles = Vec::new();
    let mut source = lines.into_iter().peekable();
    while source.peek().is_some() {
        let chunk: Vec<String> = source.by_ref().take(chunk_size).collect();
        handles.push(thread::spawn(move || {
            let mut ok = Vec::with_capacity(chunk.len());
            let mut bad = Vec::new();
            for line in &chunk {
                match parse_line(line) {
                    Ok(e) => ok.push(e),
                    Err(err) => bad.push(format!("{:?}: {}", line, err)),
                }
            }
            (ok, bad)
        }));
    }
    let mut entries = Vec::new();
    let mut errors = Vec::new();
    for h in handles {
        let (ok, bad) = h.join().expect("parser thread panicked");
        entries.extend(ok);
        errors.extend(bad);
    }
    entries.sort_by_key(|e| e.ts);
    (entries, errors)
}

fn main() {
    let levels = ["INFO", "INFO", "WARN", "INFO", "ERROR"];
    let mut lines: Vec<String> = (0..3000u64)
        .map(|i| {
            format!(
                "{} {} user{} request {} took {}ms",
                1_700_000_000 + i * 3,
                levels[(i % 5) as usize],
                (i * 7919) % 450,
                i,
                (i * 37) % 900
            )
        })
        .collect();
    lines.push("garbage line".to_string());
    lines.push("1700009999 TRACE user1 noisy".to_string());

    let (entries, errors) = parse_parallel(lines, 4);
    let mut sinks: Vec<Box<dyn Sink>> = vec![Box::new(LevelCounts::default()), Box::new(ActiveUsers::default())];
    for e in &entries {
        for s in sinks.iter_mut() {
            s.accept(e);
        }
    }
    println!("parsed {} entries, {} errors", entries.len(), errors.len());
    for err in &errors {
        println!("  {}", err);
    }
    for s in &sinks {
        println!("{}", s.summary());
    }
}
