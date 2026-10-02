use std::collections::HashMap;

struct Config {
    values: HashMap<String, String>,
}

impl Config {
    fn get(&self, key: &str, default: &str) -> String {
        match self.values.get(key) {
            Some(v) if v.starts_with('$') => {
                std::env::var(&v[1..]).unwrap_or_else(|_| default.to_string())
            }
            Some(v) => v.clone(),
            None => default.to_string(),
        }
    }
}

fn escape(s: &str) -> String {
    if !s.contains(['<', '>', '&']) {
        return s.to_owned();
    }
    s.chars().fold(String::with_capacity(s.len()), |mut out, c| {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            _ => out.push(c),
        }
        out
    })
}

fn main() {
    let config = Config {
        values: HashMap::from([
            ("title".into(), "Tom & Jerry".into()),
            ("user".into(), "$USER".into()),
            ("footer".into(), "plain text".into()),
        ]),
    };

    for key in ["title", "user", "footer", "missing"] {
        let raw = config.get(key, "<none>");
        println!("{key}: {}", escape(&raw));
    }
}
