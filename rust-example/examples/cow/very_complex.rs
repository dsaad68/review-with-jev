use std::collections::HashMap;
use std::fmt::Write as _;

trait Transform {
    fn apply(&self, input: &str) -> String;
    fn name(&self) -> &str;
}

struct Trim;
struct Lower;
struct Replace<'p> {
    from: &'p str,
    to: &'p str,
}
struct Pipeline<'p> {
    stages: Vec<Box<dyn Transform + 'p>>,
}

impl Transform for Trim {
    fn apply(&self, input: &str) -> String {
        input.trim().to_string()
    }
    fn name(&self) -> &str {
        "trim"
    }
}

impl Transform for Lower {
    fn apply(&self, input: &str) -> String {
        if input.chars().any(char::is_uppercase) {
            input.to_lowercase()
        } else {
            input.to_owned()
        }
    }
    fn name(&self) -> &str {
        "lower"
    }
}

impl<'p> Transform for Replace<'p> {
    fn apply(&self, input: &str) -> String {
        if input.contains(self.from) {
            input.replace(self.from, self.to)
        } else {
            String::from(input)
        }
    }
    fn name(&self) -> &str {
        "replace"
    }
}

impl<'p> Transform for Pipeline<'p> {
    fn apply(&self, input: &str) -> String {
        self.stages
            .iter()
            .fold(input.to_string(), |acc, stage| stage.apply(&acc))
    }
    fn name(&self) -> &str {
        "pipeline"
    }
}

struct Memo<'t> {
    inner: &'t dyn Transform,
    cache: HashMap<String, String>,
}

impl<'t> Memo<'t> {
    fn get(&mut self, key: &str) -> String {
        let inner = self.inner;
        self.cache
            .entry(key.to_string())
            .or_insert_with(|| inner.apply(key))
            .clone()
    }
}

fn render(template: &str, vars: &HashMap<&str, String>, depth: u8) -> String {
    if depth == 0 || !template.contains('{') {
        return template.to_string();
    }
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        match rest[start..].find('}') {
            Some(end) => {
                let key = &rest[start + 1..start + end];
                match vars.get(key) {
                    Some(v) => out.push_str(&render(v, vars, depth - 1)),
                    None => {
                        out.push('{');
                        out.push_str(key);
                        out.push('}');
                    }
                }
                rest = &rest[start + end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn normalize_path<'a, I>(segments: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut stack: Vec<String> = Vec::new();
    for seg in segments {
        match seg {
            "" | "." => continue,
            ".." => {
                stack.pop();
            }
            s => stack.push(s.to_string()),
        }
    }
    stack
}

fn main() {
    let pipeline = Pipeline {
        stages: vec![
            Box::new(Trim),
            Box::new(Lower),
            Box::new(Replace { from: " ", to: "-" }),
        ],
    };
    let mut memo = Memo {
        inner: &pipeline,
        cache: HashMap::new(),
    };

    let vars: HashMap<&str, String> = HashMap::from([
        ("root", "/srv/{env}".to_string()),
        ("env", "prod".to_string()),
        ("app", "{root}/apps".to_string()),
    ]);

    let raw = ["  My Docs ", "./reports", "{app}/../Q3 Summary", "readme", "readme"];
    let mut report = String::new();
    for entry in raw {
        let slug = memo.get(entry);
        let expanded = render(&slug, &vars, 4);
        let path = normalize_path(expanded.split('/'));
        let _ = writeln!(report, "{:<22} -> /{}", entry.trim(), path.join("/"));
    }
    print!("{report}");

    let names: Vec<&str> = pipeline.stages.iter().map(|s| s.name()).collect();
    println!("{} cached, stages: {}", memo.cache.len(), names.join(" > "));
}
