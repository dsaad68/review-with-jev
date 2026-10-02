use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn count_words(text: &str, tag: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();
        if !cleaned.is_empty() {
            *counts.entry(format!("{tag}:{cleaned}")).or_insert(0) += 1;
        }
    }
    counts
}

fn main() {
    let chapters = Arc::new(vec![
        String::from("The sea was calm. The ship was slow."),
        String::from("A storm came and the ship rocked."),
        String::from("Morning light. The sea was calm again."),
    ]);
    let totals = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    let mut tag = String::from("ch");

    let mut handles = Vec::new();
    for i in 0..chapters.len() {
        let chapters = Arc::clone(&chapters);
        let totals = Arc::clone(&totals);
        let mut label = tag.clone();
        label.push_str(&i.to_string());
        handles.push(thread::spawn(move || {
            let local = count_words(&chapters[i], &label);
            let mut guard = totals.lock().unwrap();
            for (k, v) in local {
                *guard.entry(k).or_insert(0) += v;
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    tag.push_str("apters");

    let totals = totals.lock().unwrap();
    let mut keys: Vec<&String> = totals.keys().collect();
    keys.sort();
    println!("{} {} processed, {} distinct entries", chapters.len(), tag, keys.len());
    for k in keys.iter().filter(|k| k.ends_with(":the") || k.ends_with(":sea")) {
        println!("{k} = {}", totals[*k]);
    }
}
