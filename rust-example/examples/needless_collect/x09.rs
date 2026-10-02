use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq)]
enum Value<'src> {
    Int(i64),
    Bool(bool),
    Text(&'src str),
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Text(s) => write!(f, "\"{s}\""),
        }
    }
}

#[derive(Debug)]
enum ConfigError<'src> {
    Malformed { line: usize },
    Duplicate(&'src str),
}

struct Config<'src> {
    entries: HashMap<&'src str, Value<'src>>,
}

fn parse_value(raw: &str) -> Value<'_> {
    if let Ok(n) = raw.parse::<i64>() {
        return Value::Int(n);
    }
    match raw {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        other => Value::Text(other.trim_matches('"')),
    }
}

impl<'src> Config<'src> {
    fn parse(source: &'src str) -> Result<Self, ConfigError<'src>> {
        let mut entries = HashMap::new();
        for (idx, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, raw) = line
                .split_once('=')
                .ok_or(ConfigError::Malformed { line: idx + 1 })?;
            let key = key.trim();
            if entries.insert(key, parse_value(raw.trim())).is_some() {
                return Err(ConfigError::Duplicate(key));
            }
        }
        Ok(Config { entries })
    }

    fn section<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = (&'src str, &'a Value<'src>)> + 'a {
        self.entries
            .iter()
            .filter(move |(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (*k, v))
    }
}

trait Rule {
    fn name(&self) -> &str;
    fn check(&self, config: &Config<'_>) -> Result<(), String>;
}

struct RequireKeys<'a> {
    keys: &'a [&'a str],
}

impl Rule for RequireKeys<'_> {
    fn name(&self) -> &str {
        "required keys"
    }

    fn check(&self, config: &Config<'_>) -> Result<(), String> {
        let missing: Vec<&str> = self
            .keys
            .iter()
            .copied()
            .filter(|k| !config.entries.contains_key(k))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!("missing {}", missing.join(", ")))
        }
    }
}

struct PortRange {
    low: i64,
    high: i64,
}

impl Rule for PortRange {
    fn name(&self) -> &str {
        "port range"
    }

    fn check(&self, config: &Config<'_>) -> Result<(), String> {
        let bad_ports: Vec<(&str, i64)> = config
            .section("net.")
            .filter(|(k, _)| k.ends_with("port"))
            .filter_map(|(k, v)| match v {
                Value::Int(n) if *n < self.low || *n > self.high => Some((k, *n)),
                _ => None,
            })
            .collect();
        match bad_ports.len() {
            0 => Ok(()),
            n => Err(format!("{n} port(s) outside {}..={}", self.low, self.high)),
        }
    }
}

fn run_rules(config: &Config<'_>, rules: &[Box<dyn Rule + '_>]) -> usize {
    let mut failures = 0;
    for rule in rules {
        match rule.check(config) {
            Ok(()) => println!("[ok]   {}", rule.name()),
            Err(msg) => {
                println!("[fail] {}: {msg}", rule.name());
                failures += 1;
            }
        }
    }
    failures
}

fn main() {
    let source = "\
# service configuration
service.name = \"gateway\"
service.debug = false
net.http_port = 8080
net.admin_port = 70000
net.grpc_port = 0
net.timeout_ms = 2500
";
    let config = match Config::parse(source) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("config error: {e:?}");
            return;
        }
    };

    for (k, v) in config.section("service.") {
        println!("{k} = {v}");
    }

    let required = ["service.name", "net.http_port", "storage.path"];
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(RequireKeys { keys: &required }),
        Box::new(PortRange { low: 1, high: 65535 }),
    ];
    let failures = run_rules(&config, &rules);
    println!("{failures} rule(s) failed");
}
