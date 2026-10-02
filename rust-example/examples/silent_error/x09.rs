use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;

trait Tokenizer: Sync {
    fn tokens<'a>(&self, text: &'a str) -> Vec<&'a str>;
}

struct Words;

impl Tokenizer for Words {
    fn tokens<'a>(&self, text: &'a str) -> Vec<&'a str> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .collect()
    }
}

struct WithoutStopWords<T> {
    inner: T,
    stop: HashSet<&'static str>,
}

impl<T: Tokenizer> Tokenizer for WithoutStopWords<T> {
    fn tokens<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut words = self.inner.tokens(text);
        words.retain(|w| !self.stop.contains(w.to_ascii_lowercase().as_str()));
        words
    }
}

fn merge(into: &mut HashMap<String, usize>, from: HashMap<String, usize>) {
    for (word, n) in from {
        *into.entry(word).or_insert(0) += n;
    }
}

fn scan_shard<T: Tokenizer>(tokenizer: &T, files: &[PathBuf]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for path in files {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        for word in tokenizer.tokens(&text) {
            *counts.entry(word.to_lowercase()).or_insert(0) += 1;
        }
    }
    counts
}

fn index_corpus<T: Tokenizer>(tokenizer: &T, files: &[PathBuf], threads: usize) -> HashMap<String, usize> {
    let chunk = files.len().div_ceil(threads.max(1)).max(1);
    thread::scope(|s| {
        let handles: Vec<_> = files
            .chunks(chunk)
            .map(|part| s.spawn(move || scan_shard(tokenizer, part)))
            .collect();
        let mut total = HashMap::new();
        for handle in handles {
            merge(&mut total, handle.join().expect("shard worker panicked"));
        }
        total
    })
}

fn top_words(counts: &HashMap<String, usize>, k: usize) -> Vec<(&str, usize)> {
    let mut ranked: Vec<(&str, usize)> = counts.iter().map(|(w, &n)| (w.as_str(), n)).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    ranked.truncate(k);
    ranked
}

fn write_corpus(dir: &Path) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(dir)?;
    let docs: [(&str, &[u8]); 5] = [
        ("a.txt", b"The river runs past the mill and the mill turns."),
        ("b.txt", b"A mill by the river; the river floods in spring."),
        ("c.txt", b"Spring brings rain, rain brings the river up."),
        ("d.txt", &[0xff, 0xfe, b'r', b'i', b'v', b'e', b'r']),
        ("e.txt", b"Grain goes to the mill, flour comes from the mill."),
    ];
    let mut paths = Vec::with_capacity(docs.len() + 1);
    for (name, body) in docs {
        let path = dir.join(name);
        fs::write(&path, body)?;
        paths.push(path);
    }
    paths.push(dir.join("missing.txt"));
    Ok(paths)
}

fn main() -> io::Result<()> {
    let dir = std::env::temp_dir().join("corpus_word_index");
    let files = write_corpus(&dir)?;

    let tokenizer = WithoutStopWords {
        inner: Words,
        stop: ["the", "a", "and", "by", "in", "to", "from"].into_iter().collect(),
    };
    let counts = index_corpus(&tokenizer, &files, 3);

    println!("indexed {} files, {} distinct words", files.len(), counts.len());
    for (word, n) in top_words(&counts, 5) {
        println!("{word:>8} {n}");
    }

    let _ = fs::remove_dir_all(&dir);
    Ok(())
}
