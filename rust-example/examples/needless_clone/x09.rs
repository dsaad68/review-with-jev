use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Int(i64),
    Bool(bool),
    Text(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Text(s) => write!(f, "\"{s}\""),
        }
    }
}

#[derive(Debug)]
enum ParseError {
    MissingEquals(usize),
    EmptyKey(usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingEquals(line) => write!(f, "line {line}: expected key = value"),
            ParseError::EmptyKey(line) => write!(f, "line {line}: empty key"),
        }
    }
}

struct Lexer<'a> {
    src: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer { src }
    }

    fn pairs(&self) -> Result<Vec<(&'a str, &'a str)>, ParseError> {
        let mut out = Vec::new();
        for (i, raw) in self.src.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (k, v) = line.split_once('=').ok_or(ParseError::MissingEquals(i + 1))?;
            let key = k.trim();
            if key.is_empty() {
                return Err(ParseError::EmptyKey(i + 1));
            }
            out.push((key, v.trim()));
        }
        Ok(out)
    }
}

fn interpret(raw: &str) -> Value {
    if let Ok(n) = raw.parse::<i64>() {
        return Value::Int(n);
    }
    match raw {
        "true" | "yes" | "on" => Value::Bool(true),
        "false" | "no" | "off" => Value::Bool(false),
        _ => Value::Text(raw.trim_matches('"').to_string()),
    }
}

#[derive(Clone, Default)]
struct Config {
    entries: HashMap<String, Value>,
}

impl Config {
    fn load(text: &str) -> Result<Config, ParseError> {
        let mut entries = HashMap::new();
        for (k, v) in Lexer::new(text).pairs()? {
            entries.insert(k.to_string(), interpret(v));
        }
        Ok(Config { entries })
    }

    fn overlay(&mut self, other: Config) {
        self.entries.extend(other.entries);
    }

    fn diff<'c>(&'c self, other: &'c Config) -> Vec<(&'c str, Option<&'c Value>, &'c Value)> {
        let mut changes: Vec<_> = other
            .entries
            .iter()
            .filter(|(k, v)| self.entries.get(*k) != Some(*v))
            .map(|(k, v)| (k.as_str(), self.entries.get(k), v))
            .collect();
        changes.sort_by(|a, b| a.0.cmp(b.0));
        changes
    }
}

fn read_layer(name: &str) -> &'static str {
    match name {
        "defaults" => "# shipped defaults\nport = 8080\nworkers = 4\ndebug = off\nname = \"edge\"\n",
        "site" => "workers = 16\nregion = eu-west\n",
        "local" => "debug = yes\nport = 9090\n",
        _ => "",
    }
}

fn main() {
    let base = {
        let text = read_layer("defaults");
        match Config::load(text) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("defaults: {e}");
                return;
            }
        }
    };

    let mut effective = base.clone();
    for layer in ["site", "local"] {
        let text = read_layer(layer);
        match Config::load(text) {
            Ok(c) => effective.overlay(c),
            Err(e) => eprintln!("{layer}: {e}"),
        }
    }

    for (key, old, new) in base.diff(&effective) {
        match old {
            Some(o) => println!("{key}: {o} -> {new}"),
            None => println!("{key}: (unset) -> {new}"),
        }
    }
    println!("{} defaults, {} effective", base.entries.len(), effective.entries.len());

    match Config::load("broken line\n") {
        Ok(_) => println!("unexpected success"),
        Err(e) => println!("rejected: {e}"),
    }
}
