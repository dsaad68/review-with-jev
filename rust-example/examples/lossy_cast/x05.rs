use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
enum ConfigError {
    Malformed(usize),
    Missing(&'static str),
    BadNumber { key: &'static str, raw: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Malformed(line) => write!(f, "line {line} is not key = value"),
            ConfigError::Missing(key) => write!(f, "missing key {key}"),
            ConfigError::BadNumber { key, raw } => write!(f, "{key} has non-numeric value {raw}"),
        }
    }
}

#[derive(Debug)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: u8,
    timeout_ms: u32,
}

fn parse_pairs(text: &str) -> Result<HashMap<&str, &str>, ConfigError> {
    let mut map = HashMap::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (k, v) = line.split_once('=').ok_or(ConfigError::Malformed(n + 1))?;
        map.insert(k.trim(), v.trim());
    }
    Ok(map)
}

fn number(map: &HashMap<&str, &str>, key: &'static str) -> Result<i64, ConfigError> {
    let raw = map.get(key).ok_or(ConfigError::Missing(key))?;
    raw.parse::<i64>().map_err(|_| ConfigError::BadNumber {
        key,
        raw: raw.to_string(),
    })
}

fn load(text: &str) -> Result<ServerConfig, ConfigError> {
    let map = parse_pairs(text)?;
    let host = map.get("host").ok_or(ConfigError::Missing("host"))?.to_string();
    Ok(ServerConfig {
        host,
        port: number(&map, "port")? as u16,
        workers: number(&map, "workers")? as u8,
        timeout_ms: number(&map, "timeout_ms")? as u32,
    })
}

fn main() {
    let text = "# server\nhost = 0.0.0.0\nport = 8080\nworkers = 4\ntimeout_ms = 2500\n";
    match load(text) {
        Ok(cfg) => println!("{cfg:?}"),
        Err(e) => println!("config error: {e}"),
    }
    match load("host = x\nport\n") {
        Ok(cfg) => println!("{cfg:?}"),
        Err(e) => println!("config error: {e}"),
    }
}
