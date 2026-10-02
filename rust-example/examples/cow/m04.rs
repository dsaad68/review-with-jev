use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Article {
    title: String,
    tags: Vec<String>,
}

fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut last_dash = true;
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

fn hex_digest(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

fn permalink(article: &Article) -> String {
    let slug = slugify(&article.title);
    let short = &hex_digest(&article.title)[..8];
    format!("/posts/{}-{}", slug, short)
}

fn index_by_tag(articles: &[Article]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for a in articles {
        for t in &a.tags {
            map.entry(slugify(t)).or_default().push(permalink(a));
        }
    }
    map
}

fn main() {
    let articles = vec![
        Article { title: "Hello, World!".into(), tags: vec!["Intro".into(), "Rust Lang".into()] },
        Article { title: "Iterators in Depth".into(), tags: vec!["Rust Lang".into()] },
        Article { title: "  Spaces   everywhere ".into(), tags: vec!["Misc".into()] },
    ];
    for a in &articles {
        println!("{} -> {}", a.title, permalink(a));
    }
    let index = index_by_tag(&articles);
    let mut keys: Vec<_> = index.keys().collect();
    keys.sort();
    for k in keys {
        println!("{}: {:?}", k, index[k]);
    }
}
