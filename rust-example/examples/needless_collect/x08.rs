use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

trait Tokenizer: Send + Sync + 'static {
    fn tokens<'a>(&self, text: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a>;
}

#[derive(Clone)]
struct Whitespace {
    min_len: usize,
}

impl Tokenizer for Whitespace {
    fn tokens<'a>(&self, text: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        let min_len = self.min_len;
        Box::new(
            text.split(|c: char| !c.is_alphanumeric())
                .filter(move |w| w.len() >= min_len),
        )
    }
}

#[derive(Default)]
struct Frequencies {
    counts: HashMap<String, usize>,
    documents: usize,
}

impl Frequencies {
    fn absorb(&mut self, other: Frequencies) {
        for (word, n) in other.counts {
            *self.counts.entry(word).or_insert(0) += n;
        }
        self.documents += other.documents;
    }

    fn top(&self, k: usize) -> Vec<(&str, usize)> {
        let mut ranked: Vec<(&str, usize)> = self
            .counts
            .iter()
            .map(|(w, n)| (w.as_str(), *n))
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        ranked.truncate(k);
        ranked
    }
}

fn count_partition<T: Tokenizer>(tokenizer: &T, docs: &[String]) -> Frequencies {
    let mut freq = Frequencies::default();
    for doc in docs {
        for token in tokenizer.tokens(doc) {
            *freq.counts.entry(token.to_lowercase()).or_insert(0) += 1;
        }
        freq.documents += 1;
    }
    freq
}

fn parallel_count<T>(tokenizer: T, docs: Vec<String>, workers: usize) -> Frequencies
where
    T: Tokenizer + Clone,
{
    let workers = workers.max(1);
    let mut buckets: Vec<Vec<String>> = (0..workers).map(|_| Vec::new()).collect();
    for (i, doc) in docs.into_iter().enumerate() {
        buckets[i % workers].push(doc);
    }

    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::with_capacity(workers);
    for bucket in buckets {
        let tx = tx.clone();
        let tok = tokenizer.clone();
        handles.push(thread::spawn(move || {
            let partial = count_partition(&tok, &bucket);
            let _ = tx.send(partial);
        }));
    }
    drop(tx);

    let mut total = Frequencies::default();
    for partial in rx {
        total.absorb(partial);
    }
    for h in handles {
        h.join().expect("worker panicked");
    }
    total
}

fn main() {
    let corpus = [
        "The quick brown fox jumps over the lazy dog",
        "A quick test of the parallel word counter",
        "Rust threads make the counter fast and the code safe",
        "The dog sleeps while the fox runs",
        "Counting words in parallel is a classic exercise",
        "Safe concurrency without data races is the goal",
    ];
    let docs: Vec<String> = corpus.iter().map(|s| s.to_string()).collect();

    let freq = parallel_count(Whitespace { min_len: 3 }, docs, 3);
    println!("{} documents, {} distinct words", freq.documents, freq.counts.len());
    for (word, n) in freq.top(5) {
        println!("{word:>10} {n}");
    }
}
