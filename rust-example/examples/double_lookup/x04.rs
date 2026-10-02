use std::collections::HashMap;
use std::fmt;

struct Settings {
    defaults: HashMap<&'static str, String>,
    overrides: HashMap<String, String>,
}

#[derive(Debug)]
struct MissingSetting(String);

impl fmt::Display for MissingSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no value for setting `{}`", self.0)
    }
}

impl Settings {
    fn new() -> Self {
        let mut defaults = HashMap::new();
        defaults.insert("theme", "light".to_string());
        defaults.insert("font_size", "14".to_string());
        defaults.insert("autosave", "on".to_string());
        Settings { defaults, overrides: HashMap::new() }
    }

    fn set(&mut self, key: &str, value: &str) {
        self.overrides.insert(key.to_string(), value.to_string());
    }

    fn resolve(&self, key: &str) -> Result<&str, MissingSetting> {
        if self.overrides.contains_key(key) {
            return Ok(&self.overrides[key]);
        }
        self.defaults
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| MissingSetting(key.to_string()))
    }

    fn describe(&self, keys: &[&str]) -> String {
        let mut out = String::new();
        for key in keys {
            match self.resolve(key) {
                Ok(v) => out.push_str(&format!("{key} = {v}\n")),
                Err(e) => out.push_str(&format!("{e}\n")),
            }
        }
        out
    }
}

fn main() {
    let mut settings = Settings::new();
    settings.set("theme", "dark");
    settings.set("line_numbers", "relative");

    let report = settings.describe(&["theme", "font_size", "line_numbers", "tab_width"]);
    print!("{report}");
}
