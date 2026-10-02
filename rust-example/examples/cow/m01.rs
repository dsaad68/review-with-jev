use std::collections::HashMap;

struct Settings {
    values: HashMap<String, String>,
    profile: String,
}

impl Settings {
    fn new(profile: &str) -> Self {
        Settings {
            values: HashMap::new(),
            profile: profile.to_string(),
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    fn resolve(&self, key: &str) -> String {
        let scoped = format!("{}.{}", self.profile, key);
        if let Some(v) = self.values.get(&scoped) {
            return v.clone();
        }
        match self.values.get(key) {
            Some(v) => v.clone(),
            None => format!("<unset:{}>", key),
        }
    }

    fn resolve_all(&self, keys: &[&str]) -> Vec<(String, String)> {
        keys.iter()
            .map(|k| (k.to_string(), self.resolve(k)))
            .collect()
    }
}

fn parse_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let (k, v) = line.split_once('=')?;
    Some((k.trim(), v.trim()))
}

fn main() {
    let source = "host = localhost\nport = 8080\nprod.host = example.org\n# ignored\n";
    let mut settings = Settings::new("prod");
    for line in source.lines() {
        if let Some((k, v)) = parse_line(line) {
            settings.set(k, v);
        }
    }
    for (k, v) in settings.resolve_all(&["host", "port", "timeout"]) {
        println!("{} => {}", k, v);
    }
    let port_len = settings.resolve("port").len();
    println!("port digits: {}", port_len);
}
