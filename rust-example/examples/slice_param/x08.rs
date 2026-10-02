use std::collections::{HashMap, HashSet};
use std::fmt;
use std::thread;

struct Document {
    id: u32,
    title: String,
    body: String,
}

impl Document {
    fn new(id: u32, title: &str, body: &str) -> Self {
        Document {
            id,
            title: title.to_string(),
            body: body.to_string(),
        }
    }
}

type Index = HashMap<String, HashSet<u32>>;

fn terms(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(|w| w.to_ascii_lowercase())
}

fn index_chunk(chunk: &[Document]) -> Index {
    let mut index = Index::new();
    for doc in chunk {
        for term in terms(&doc.title).chain(terms(&doc.body)) {
            index.entry(term).or_default().insert(doc.id);
        }
    }
    index
}

fn merge_into(target: &mut Index, part: Index) {
    for (term, ids) in part {
        target.entry(term).or_default().extend(ids);
    }
}

fn build_index(docs: &[Document], workers: usize) -> Index {
    let size = docs.len().div_ceil(workers.max(1)).max(1);
    thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in docs.chunks(size) {
            handles.push(scope.spawn(move || index_chunk(chunk)));
        }
        let mut index = Index::new();
        for handle in handles {
            merge_into(&mut index, handle.join().expect("indexer thread panicked"));
        }
        index
    })
}

trait Scorer {
    fn score(&self, doc: &Document, matched: usize, query_len: usize) -> f64;
}

struct Coverage;

impl Scorer for Coverage {
    fn score(&self, _doc: &Document, matched: usize, query_len: usize) -> f64 {
        matched as f64 / query_len as f64
    }
}

struct ShortTitleBoost {
    weight: f64,
}

impl Scorer for ShortTitleBoost {
    fn score(&self, doc: &Document, matched: usize, query_len: usize) -> f64 {
        let base = matched as f64 / query_len as f64;
        let title_words = terms(&doc.title).count() as f64;
        base * (1.0 + self.weight / (1.0 + title_words))
    }
}

struct Hit<'a> {
    doc: &'a Document,
    score: f64,
}

impl fmt::Display for Hit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{} {:<28} {:.3}", self.doc.id, self.doc.title, self.score)
    }
}

fn search<'a, S: Scorer>(docs: &'a [Document], index: &Index, query: &str, scorer: &S) -> Vec<Hit<'a>> {
    let query_terms: HashSet<String> = terms(query).collect();
    if query_terms.is_empty() {
        return Vec::new();
    }

    let mut matches: HashMap<u32, usize> = HashMap::new();
    for term in &query_terms {
        if let Some(ids) = index.get(term) {
            for id in ids {
                *matches.entry(*id).or_insert(0) += 1;
            }
        }
    }

    let by_id: HashMap<u32, &Document> = docs.iter().map(|d| (d.id, d)).collect();
    let mut hits = Vec::new();
    for (id, matched) in matches {
        if let Some(doc) = by_id.get(&id) {
            hits.push(Hit {
                doc: *doc,
                score: scorer.score(doc, matched, query_terms.len()),
            });
        }
    }
    hits.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.doc.id.cmp(&b.doc.id)));
    hits
}

fn report(label: &str, hits: &[Hit<'_>]) {
    println!("{label}: {} hits", hits.len());
    for hit in hits {
        println!("  {hit}");
    }
}

fn main() {
    let docs = vec![
        Document::new(1, "Borrowing basics", "References let code read data without taking ownership."),
        Document::new(2, "Threads and scoped spawning", "Scoped threads may borrow data from the parent stack."),
        Document::new(3, "Iterator adapters", "Adapters such as map and filter are lazy until consumed."),
        Document::new(4, "Ownership", "Every value has a single owner; moving transfers ownership."),
        Document::new(5, "Lifetimes in structs", "Structs holding references need lifetime parameters to borrow data."),
        Document::new(6, "Error handling", "Return Result and propagate failures with the question mark operator."),
    ];

    let index = build_index(&docs, 3);
    println!("indexed {} distinct terms", index.len());

    let query = "borrow data ownership";
    report("coverage", &search(&docs, &index, query, &Coverage));
    report("short titles", &search(&docs, &index, query, &ShortTitleBoost { weight: 2.0 }));
    report("empty", &search(&docs, &index, "a an", &Coverage));
}
