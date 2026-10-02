use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

trait Filter {
    fn apply<'a>(&self, input: Cow<'a, str>) -> Cow<'a, str>;
}

struct Upper;
struct Trim;
struct HtmlEscape;
struct Truncate(usize);

impl Filter for Upper {
    fn apply<'a>(&self, input: Cow<'a, str>) -> Cow<'a, str> {
        if input.chars().any(char::is_lowercase) {
            Cow::Owned(input.to_uppercase())
        } else {
            input
        }
    }
}

impl Filter for Trim {
    fn apply<'a>(&self, input: Cow<'a, str>) -> Cow<'a, str> {
        match input {
            Cow::Borrowed(s) => Cow::Borrowed(s.trim()),
            Cow::Owned(s) => {
                let trimmed = s.trim();
                if trimmed.len() == s.len() {
                    Cow::Owned(s)
                } else {
                    Cow::Owned(trimmed.to_string())
                }
            }
        }
    }
}

impl Filter for HtmlEscape {
    fn apply<'a>(&self, input: Cow<'a, str>) -> Cow<'a, str> {
        if !input.contains(['<', '>', '&', '"']) {
            return input;
        }
        let mut out = String::with_capacity(input.len() + 16);
        for c in input.chars() {
            match c {
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '&' => out.push_str("&amp;"),
                '"' => out.push_str("&quot;"),
                other => out.push(other),
            }
        }
        Cow::Owned(out)
    }
}

impl Filter for Truncate {
    fn apply<'a>(&self, input: Cow<'a, str>) -> Cow<'a, str> {
        match input.char_indices().nth(self.0) {
            None => input,
            Some((idx, _)) => Cow::Owned(format!("{}...", &input[..idx])),
        }
    }
}

#[derive(Debug)]
enum Node<'t> {
    Text(&'t str),
    Var { name: &'t str, filters: Vec<&'t str> },
}

#[derive(Debug)]
enum TemplateError { Unclosed(usize), UnknownFilter(String), MissingVar(String) }

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::Unclosed(at) => write!(f, "unclosed tag at byte {}", at),
            TemplateError::UnknownFilter(n) => write!(f, "unknown filter '{}'", n),
            TemplateError::MissingVar(n) => write!(f, "missing variable '{}'", n),
        }
    }
}

fn parse(src: &str) -> Result<Vec<Node<'_>>, TemplateError> {
    let mut nodes = Vec::new();
    let mut rest = src;
    let mut offset = 0;
    while let Some(open) = rest.find("{{") {
        if open > 0 {
            nodes.push(Node::Text(&rest[..open]));
        }
        let after = &rest[open + 2..];
        let close = after.find("}}").ok_or(TemplateError::Unclosed(offset + open))?;
        let mut pieces = after[..close].split('|').map(str::trim);
        let name = pieces.next().unwrap_or("");
        nodes.push(Node::Var { name, filters: pieces.collect() });
        let consumed = open + 2 + close + 2;
        offset += consumed;
        rest = &rest[consumed..];
    }
    if !rest.is_empty() {
        nodes.push(Node::Text(rest));
    }
    Ok(nodes)
}

struct Engine {
    filters: HashMap<&'static str, Box<dyn Filter>>,
    strict: bool,
}

impl Engine {
    fn new(strict: bool) -> Self {
        let mut filters: HashMap<&'static str, Box<dyn Filter>> = HashMap::new();
        filters.insert("upper", Box::new(Upper));
        filters.insert("trim", Box::new(Trim));
        filters.insert("escape", Box::new(HtmlEscape));
        filters.insert("short", Box::new(Truncate(12)));
        Engine { filters, strict }
    }

    fn render<V: AsRef<str>>(&self, nodes: &[Node<'_>], ctx: &HashMap<&str, V>) -> Result<String, TemplateError> {
        let mut out = String::new();
        for node in nodes {
            match node {
                Node::Text(t) => out.push_str(t),
                Node::Var { name, filters } => {
                    let raw = match ctx.get(name) {
                        Some(v) => v.as_ref(),
                        None if self.strict => return Err(TemplateError::MissingVar(name.to_string())),
                        None => "",
                    };
                    let mut value: Cow<'_, str> = Cow::Borrowed(raw);
                    for fname in filters {
                        let filter = self
                            .filters
                            .get(fname)
                            .ok_or_else(|| TemplateError::UnknownFilter(fname.to_string()))?;
                        value = filter.apply(value);
                    }
                    out.push_str(&value);
                }
            }
        }
        Ok(out)
    }
}

fn main() {
    let template = "<h1>{{ title | trim | escape }}</h1>\n<p>By {{ author | upper }}: {{ summary | short | escape }}</p>\n<footer>{{ footer }}</footer>";
    let nodes = parse(template).expect("parse");
    let mut ctx: HashMap<&str, String> = HashMap::new();
    ctx.insert("title", "  Fish & Chips <tonight>  ".to_string());
    ctx.insert("author", "ADA".to_string());
    ctx.insert("summary", "A long story about a \"great\" dinner".to_string());

    let lenient = Engine::new(false);
    match lenient.render(&nodes, &ctx) {
        Ok(html) => println!("{}", html),
        Err(e) => println!("error: {}", e),
    }
    let strict = Engine::new(true);
    match strict.render(&nodes, &ctx) {
        Ok(html) => println!("{}", html),
        Err(e) => println!("strict error: {}", e),
    }
    let bad = parse("hello {{ name | shout }}").unwrap();
    let simple: HashMap<&str, &str> = [("name", "bob")].into_iter().collect();
    if let Err(e) = lenient.render(&bad, &simple) {
        println!("error: {}", e);
    }
    if let Err(e) = parse("oops {{ name") {
        println!("error: {}", e);
    }
}
