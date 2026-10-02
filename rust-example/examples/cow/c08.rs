use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;

trait TextPass {
    fn name(&self) -> &'static str;
    fn run(&self, input: &str) -> String;
}

struct StripControl;
struct CollapseSpaces;
struct FoldQuotes;
struct LowerAscii;

impl TextPass for StripControl {
    fn name(&self) -> &'static str {
        "strip-control"
    }
    fn run(&self, input: &str) -> String {
        if !input.chars().any(|c| c.is_control() && c != '\n') {
            return input.to_string();
        }
        input.chars().filter(|c| !c.is_control() || *c == '\n').collect()
    }
}

impl TextPass for CollapseSpaces {
    fn name(&self) -> &'static str {
        "collapse-spaces"
    }
    fn run(&self, input: &str) -> String {
        let trimmed = input.trim();
        let doubled = trimmed.as_bytes().windows(2).any(|w| w[0].is_ascii_whitespace() && w[1].is_ascii_whitespace());
        if !doubled && trimmed.len() == input.len() {
            return input.to_string();
        }
        trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

impl TextPass for FoldQuotes {
    fn name(&self) -> &'static str {
        "fold-quotes"
    }
    fn run(&self, input: &str) -> String {
        if input.is_ascii() {
            return input.to_string();
        }
        input
            .chars()
            .map(|c| match c {
                '\u{2018}' | '\u{2019}' => '\'',
                '\u{201C}' | '\u{201D}' => '"',
                '\u{2013}' | '\u{2014}' => '-',
                other => other,
            })
            .collect()
    }
}

impl TextPass for LowerAscii {
    fn name(&self) -> &'static str {
        "lower-ascii"
    }
    fn run(&self, input: &str) -> String {
        input.to_ascii_lowercase()
    }
}

struct Pipeline {
    passes: Vec<Box<dyn TextPass>>,
    changed_by: HashMap<&'static str, usize>,
}

impl Pipeline {
    fn new() -> Self {
        Pipeline { passes: Vec::new(), changed_by: HashMap::new() }
    }

    fn with<P: TextPass + 'static>(mut self, pass: P) -> Self {
        self.passes.push(Box::new(pass));
        self
    }

    fn normalize(&mut self, input: &str) -> String {
        let mut current = input.to_string();
        for pass in &self.passes {
            let next = pass.run(&current);
            if next != current {
                *self.changed_by.entry(pass.name()).or_insert(0) += 1;
            }
            current = next;
        }
        current
    }
}

#[derive(Debug, Default)]
struct QueryReport {
    total: usize,
    distinct: usize,
    top_terms: Vec<(String, usize)>,
    by_length: BTreeMap<usize, usize>,
}

impl fmt::Display for QueryReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "queries: {} (distinct after normalization: {})", self.total, self.distinct)?;
        for (term, n) in &self.top_terms {
            writeln!(f, "  {:<10} {}", term, n)?;
        }
        write!(f, "  words-per-query histogram: {:?}", self.by_length)
    }
}

fn analyze<'q, I>(pipeline: &mut Pipeline, queries: I, top: usize) -> QueryReport
where
    I: IntoIterator<Item = &'q str>,
{
    let mut seen: HashSet<String> = HashSet::new();
    let mut terms: HashMap<String, usize> = HashMap::new();
    let mut report = QueryReport::default();
    for q in queries {
        report.total += 1;
        let normalized = pipeline.normalize(q);
        let words = normalized.split(' ').filter(|w| !w.is_empty());
        let mut count = 0;
        for w in words {
            count += 1;
            match terms.get_mut(w) {
                Some(n) => *n += 1,
                None => {
                    terms.insert(w.to_string(), 1);
                }
            }
        }
        *report.by_length.entry(count).or_insert(0) += 1;
        seen.insert(normalized);
    }
    report.distinct = seen.len();
    let mut ranked: Vec<(String, usize)> = terms.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(top);
    report.top_terms = ranked;
    report
}

fn main() {
    let queries = [
        "rust borrow checker",
        "Rust  borrow checker ",
        "async rust",
        "\u{201C}async\u{201D} rust",
        "tokio\tspawn",
        "tokio spawn",
        "serde json",
        "SERDE json",
        "rust lifetimes",
        "what is a lifetime\u{2014}really",
    ];
    let mut pipeline = Pipeline::new()
        .with(StripControl)
        .with(FoldQuotes)
        .with(CollapseSpaces)
        .with(LowerAscii);
    let report = analyze(&mut pipeline, queries.iter().copied(), 4);
    println!("{}", report);
    let mut changes: Vec<_> = pipeline.changed_by.iter().collect();
    changes.sort();
    println!("passes that changed input: {:?}", changes);
}
