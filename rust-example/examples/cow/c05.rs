use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Origin {
    Defaults,
    File,
    Env,
    Cli,
}

#[derive(Debug)]
struct Layer {
    origin: Origin,
    values: BTreeMap<String, String>,
}

#[derive(Debug)]
enum ConfigError {
    Missing(String),
    Syntax { line: usize, text: String },
    Cycle(String),
    Invalid { key: String, value: String, reason: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(k) => write!(f, "missing key {}", k),
            ConfigError::Syntax { line, text } => write!(f, "syntax error on line {}: {}", line, text),
            ConfigError::Cycle(k) => write!(f, "reference cycle through {}", k),
            ConfigError::Invalid { key, value, reason } => write!(f, "{}={:?}: {}", key, value, reason),
        }
    }
}

impl Layer {
    fn from_pairs<I, K, V>(origin: Origin, pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values = pairs.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Layer { origin, values }
    }

    fn parse_ini(origin: Origin, text: &str) -> Result<Self, ConfigError> {
        let mut values = BTreeMap::new();
        let mut section = String::new();
        for (idx, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = name.trim().to_string();
                continue;
            }
            let (k, v) = line.split_once('=').ok_or_else(|| ConfigError::Syntax {
                line: idx + 1,
                text: line.to_string(),
            })?;
            let key = if section.is_empty() {
                k.trim().to_string()
            } else {
                format!("{}.{}", section, k.trim())
            };
            values.insert(key, v.trim().trim_matches('"').to_string());
        }
        Ok(Layer { origin, values })
    }
}

struct Config {
    layers: Vec<Layer>,
}

impl Config {
    fn new(mut layers: Vec<Layer>) -> Self {
        layers.sort_by_key(|l| l.origin);
        Config { layers }
    }

    fn raw(&self, key: &str) -> Option<(&str, Origin)> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.values.get(key).map(|v| (v.as_str(), l.origin)))
    }

    fn expand(&self, value: &str, depth: usize) -> Result<String, ConfigError> {
        if !value.contains("${") {
            return Ok(value.to_string());
        }
        if depth > 8 {
            return Err(ConfigError::Cycle(value.to_string()));
        }
        let mut out = String::with_capacity(value.len() + 16);
        let mut rest = value;
        while let Some(start) = rest.find("${") {
            out.push_str(&rest[..start]);
            let tail = &rest[start + 2..];
            let end = tail.find('}').ok_or_else(|| ConfigError::Syntax { line: 0, text: value.to_string() })?;
            let name = &tail[..end];
            let (inner, _) = self.raw(name).ok_or_else(|| ConfigError::Missing(name.to_string()))?;
            out.push_str(&self.expand(inner, depth + 1)?);
            rest = &tail[end + 1..];
        }
        out.push_str(rest);
        Ok(out)
    }

    fn get<T>(&self, key: &str) -> Result<T, ConfigError>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        let (raw, _) = self.raw(key).ok_or_else(|| ConfigError::Missing(key.to_string()))?;
        let expanded = self.expand(raw, 0)?;
        expanded.parse::<T>().map_err(|e| ConfigError::Invalid {
            key: key.to_string(),
            value: expanded.clone(),
            reason: e.to_string(),
        })
    }

    fn get_or<T>(&self, key: &str, fallback: T) -> T
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        self.get(key).unwrap_or(fallback)
    }

    fn report(&self) -> Vec<(String, Origin, Result<String, ConfigError>)> {
        let mut keys: Vec<&String> = self.layers.iter().flat_map(|l| l.values.keys()).collect();
        keys.sort();
        keys.dedup();
        keys.into_iter()
            .filter_map(|k| self.raw(k).map(|(v, o)| (k.clone(), o, self.expand(v, 0))))
            .collect()
    }
}

fn main() {
    let defaults = Layer::from_pairs(
        Origin::Defaults,
        [("server.host", "127.0.0.1"), ("server.port", "8080"), ("server.workers", "4"), ("debug", "false")],
    );
    let file = Layer::parse_ini(
        Origin::File,
        "; app config\n[server]\nhost = \"0.0.0.0\"\nport = 9000\n[db]\nuser = app\nurl = postgres://${db.user}@${server.host}:5432/main\n",
    )
    .expect("ini");
    let env = Layer::from_pairs(Origin::Env, vec![("db.user".to_string(), "svc".to_string())]);
    let cli = Layer::from_pairs(Origin::Cli, [("debug", "true"), ("server.workers", "lots")]);
    let config = Config::new(vec![cli, file, env, defaults]);

    let port: u16 = config.get_or("server.port", 80);
    let debug: bool = config.get_or("debug", false);
    println!("port={} debug={}", port, debug);
    match config.get::<usize>("server.workers") {
        Ok(n) => println!("workers={}", n),
        Err(e) => println!("error: {}", e),
    }
    match config.get::<String>("db.url") {
        Ok(url) => println!("db.url={}", url),
        Err(e) => println!("error: {}", e),
    }
    for (key, origin, value) in config.report() {
        match value {
            Ok(v) => println!("{:<16} {:<9} {}", key, format!("{:?}", origin), v),
            Err(e) => println!("{:<16} {:<9} !{}", key, format!("{:?}", origin), e),
        }
    }
}
