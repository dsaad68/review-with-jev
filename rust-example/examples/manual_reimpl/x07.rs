use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug)]
enum ConfigError {
    MissingEquals { line: usize },
    EmptyKey { line: usize },
    UnclosedSection { line: usize },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingEquals { line } => write!(f, "line {line}: expected key = value"),
            ConfigError::EmptyKey { line } => write!(f, "line {line}: empty key"),
            ConfigError::UnclosedSection { line } => write!(f, "line {line}: missing ']'"),
        }
    }
}

impl std::error::Error for ConfigError {}

type Sections = BTreeMap<String, BTreeMap<String, String>>;

fn parse(text: &str) -> Result<Sections, ConfigError> {
    let mut sections = Sections::new();
    let mut current = String::from("default");
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let name = rest.strip_suffix(']').ok_or(ConfigError::UnclosedSection { line: n + 1 })?;
            current = name.trim().to_string();
            continue;
        }
        let (key, value) = line.split_once('=').ok_or(ConfigError::MissingEquals { line: n + 1 })?;
        let key = key.trim();
        if key.is_empty() {
            return Err(ConfigError::EmptyKey { line: n + 1 });
        }
        sections
            .entry(current.clone())
            .or_default()
            .insert(key.to_string(), value.trim().to_string());
    }
    Ok(sections)
}

fn main() {
    let good = "# server\nport = 8080\n[db]\nhost = localhost\npool = 12\n\n[cache]\nttl = 300\n";
    match parse(good) {
        Ok(sections) => {
            for (name, keys) in &sections {
                println!("[{name}] {keys:?}");
            }
        }
        Err(e) => println!("error: {e}"),
    }
    for bad in ["[db\nhost = x", "port 8080", " = 3"] {
        if let Err(e) = parse(bad) {
            println!("rejected: {e}");
        }
    }
}
