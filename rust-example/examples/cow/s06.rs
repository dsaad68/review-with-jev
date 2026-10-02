fn normalize_tag(tag: &str) -> String {
    if tag.chars().any(|c| c.is_ascii_uppercase()) {
        tag.to_ascii_lowercase()
    } else {
        tag.to_string()
    }
}

fn main() {
    let tags = ["rust", "WebAssembly", "async", "CLI"];
    let mut normalized = Vec::new();
    for tag in tags {
        let t = normalize_tag(tag);
        if !normalized.contains(&t) {
            normalized.push(t);
        }
    }
    println!("{}", normalized.join(", "));
}
