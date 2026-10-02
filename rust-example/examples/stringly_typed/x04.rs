use std::collections::HashMap;

struct Message<'a> {
    author: &'a str,
    body: &'a str,
}

fn parse_line(line: &str) -> Option<Message<'_>> {
    let (author, body) = line.split_once(':')?;
    let author = author.trim();
    if author.is_empty() {
        return None;
    }
    Some(Message { author, body: body.trim() })
}

fn mentions(body: &str) -> impl Iterator<Item = &str> {
    body.split_whitespace()
        .filter_map(|w| w.strip_prefix('@'))
        .map(|w| w.trim_end_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
}

fn main() {
    let transcript = "\
alice: morning @bob, did the build pass?
bob: yes, green on all targets
carol: @alice @bob retro moved to 3pm
garbage line without separator
bob: thanks @carol!
alice: @carol works for me";

    let mut messages = Vec::new();
    let mut skipped = 0;
    for line in transcript.lines() {
        match parse_line(line) {
            Some(m) => messages.push(m),
            None => skipped += 1,
        }
    }

    let mut sent: HashMap<&str, usize> = HashMap::new();
    let mut mentioned: HashMap<&str, usize> = HashMap::new();
    for m in &messages {
        *sent.entry(m.author).or_insert(0) += 1;
        for name in mentions(m.body) {
            *mentioned.entry(name).or_insert(0) += 1;
        }
    }

    let mut authors: Vec<_> = sent.into_iter().collect();
    authors.sort();
    for (author, count) in authors {
        let tagged = mentioned.get(author).copied().unwrap_or(0);
        println!("{author}: sent {count}, mentioned {tagged}");
    }
    println!("skipped {skipped} malformed line(s)");
}
