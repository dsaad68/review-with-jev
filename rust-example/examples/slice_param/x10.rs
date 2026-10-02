use std::fmt::Write;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn parse(token: &str) -> Option<Level> {
        match token {
            "DEBUG" => Some(Level::Debug),
            "INFO" => Some(Level::Info),
            "WARN" => Some(Level::Warn),
            "ERROR" => Some(Level::Error),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Entry {
    source: String,
    millis: u64,
    level: Level,
    message: String,
}

trait Filter: Send {
    fn keep(&self, entry: &Entry) -> bool;
    fn name(&self) -> &str;
}

struct MinLevel(Level);

struct Mentions {
    word: String,
}

impl Filter for MinLevel {
    fn keep(&self, entry: &Entry) -> bool {
        entry.level >= self.0
    }
    fn name(&self) -> &str {
        "min-level"
    }
}

impl Filter for Mentions {
    fn keep(&self, entry: &Entry) -> bool {
        entry.message.contains(self.word.as_str())
    }
    fn name(&self) -> &str {
        "mentions"
    }
}

fn parse_line(source: &Path, line: &str) -> Option<Entry> {
    let mut parts = line.splitn(3, ' ');
    let millis = parts.next()?.parse().ok()?;
    let level = Level::parse(parts.next()?)?;
    let message = parts.next()?.trim();
    Some(Entry {
        source: source.display().to_string(),
        millis,
        level,
        message: message.to_string(),
    })
}

fn ingest<'a, I>(buffer: &mut Vec<Entry>, source: &Path, lines: I) -> usize
where
    I: IntoIterator<Item = &'a str>,
{
    let before = buffer.len();
    for line in lines {
        if let Some(entry) = parse_line(source, line) {
            buffer.push(entry);
        }
    }
    buffer.len() - before
}

fn passes(filters: &[Box<dyn Filter>], entry: &Entry) -> bool {
    filters.iter().all(|f| f.keep(entry))
}

fn describe(filter: &dyn Filter) -> String {
    format!("<{}>", filter.name())
}

fn render(out: &mut String, entry: &Entry) {
    let _ = writeln!(
        out,
        "{:>6}ms {:<5?} {:<18} {}",
        entry.millis, entry.level, entry.source, entry.message
    );
}

fn main() {
    let sources: [(&str, &str); 2] = [
        (
            "/var/log/api.log",
            "100 INFO request accepted\n140 WARN slow upstream db\nbroken line\n180 ERROR db timeout",
        ),
        (
            "/var/log/worker.log",
            "90 DEBUG polling queue\n150 ERROR db connection reset\n210 INFO job done",
        ),
    ];

    let (tx, rx) = mpsc::channel::<Vec<Entry>>();
    let mut handles = Vec::new();
    for (path, text) in sources {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let mut buffer = Vec::new();
            let parsed = ingest(&mut buffer, Path::new(path), text.lines());
            println!("{path}: parsed {parsed} lines");
            let _ = tx.send(buffer);
        }));
    }
    drop(tx);

    let mut all = Vec::new();
    for batch in rx {
        all.extend(batch);
    }
    for handle in handles {
        handle.join().expect("parser thread panicked");
    }
    all.sort_by_key(|e| e.millis);

    let filters: Vec<Box<dyn Filter>> = vec![
        Box::new(MinLevel(Level::Warn)),
        Box::new(Mentions { word: String::from("db") }),
    ];

    let mut out = String::new();
    out.push_str("filters:");
    for filter in &filters {
        out.push(' ');
        out.push_str(&describe(filter.as_ref()));
    }
    out.push('\n');
    for entry in &all {
        if passes(&filters, entry) {
            render(&mut out, entry);
        }
    }
    print!("{out}");
}
