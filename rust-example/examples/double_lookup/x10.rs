use std::collections::HashMap;
use std::thread;

#[derive(Debug, Clone, Copy)]
struct Posting {
    doc: usize,
    count: u32,
}

trait Tokenizer: Sync {
    fn terms<'t>(&self, text: &'t str) -> Vec<&'t str>;
}

struct WordSplitter {
    min_len: usize,
}

impl Tokenizer for WordSplitter {
    fn terms<'t>(&self, text: &'t str) -> Vec<&'t str> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= self.min_len)
            .collect()
    }
}

struct Index<'a> {
    postings: HashMap<&'a str, Vec<Posting>>,
    totals: HashMap<&'a str, u32>,
    docs: usize,
}

impl<'a> Index<'a> {
    fn new() -> Self {
        Index { postings: HashMap::new(), totals: HashMap::new(), docs: 0 }
    }

    fn add_total(&mut self, term: &'a str, n: u32) {
        let updated = match self.totals.get(term) {
            Some(existing) => existing + n,
            None => n,
        };
        self.totals.insert(term, updated);
    }

    fn absorb(&mut self, doc: usize, counts: HashMap<&'a str, u32>) {
        for (term, count) in counts {
            self.postings.entry(term).or_default().push(Posting { doc, count });
            self.add_total(term, count);
        }
        self.docs += 1;
    }

    fn build<T: Tokenizer>(docs: &[&'a str], tokenizer: &T) -> Self {
        let per_doc: Vec<HashMap<&'a str, u32>> = thread::scope(|s| {
            let handles: Vec<_> = docs
                .iter()
                .map(|text| {
                    s.spawn(move || {
                        let mut counts: HashMap<&'a str, u32> = HashMap::new();
                        for term in tokenizer.terms(text) {
                            *counts.entry(term).or_insert(0) += 1;
                        }
                        counts
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().expect("tokenizer thread panicked")).collect()
        });

        let mut index = Index::new();
        for (doc, counts) in per_doc.into_iter().enumerate() {
            index.absorb(doc, counts);
        }
        index
    }

    fn search(&self, query: &[&str]) -> Vec<(usize, u32)> {
        let mut scores: HashMap<usize, u32> = HashMap::new();
        for term in query {
            let Some(list) = self.postings.get(term) else { continue };
            for p in list {
                *scores.entry(p.doc).or_insert(0) += p.count;
            }
        }
        let mut ranked: Vec<(usize, u32)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        ranked
    }

    fn most_common(&self, n: usize) -> Vec<(&'a str, u32)> {
        let mut all: Vec<(&'a str, u32)> = self.totals.iter().map(|(t, c)| (*t, *c)).collect();
        all.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        all.truncate(n);
        all
    }
}

fn main() {
    let docs = [
        "the river runs past the old mill and the mill wheel turns",
        "a mill town grew along the river bank",
        "trains replaced the river barges and the town shrank",
        "the old wheel still turns for visitors",
    ];
    let tokenizer = WordSplitter { min_len: 3 };
    let index = Index::build(&docs, &tokenizer);

    println!("indexed {} documents, {} terms", index.docs, index.totals.len());
    for (term, count) in index.most_common(5) {
        println!("{term:<8} {count}");
    }
    for query in [&["mill", "wheel"][..], &["town"][..], &["canal"][..]] {
        let hits = index.search(query);
        println!("{query:?} -> {hits:?}");
    }
}
