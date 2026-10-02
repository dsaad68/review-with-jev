fn unique_tags(raw: &[&str]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for tag in raw {
        let normalized = tag.trim().to_lowercase();
        if normalized.is_empty() {
            continue;
        }
        if !seen.contains(&normalized) {
            seen.push(normalized);
        }
    }
    seen
}

fn main() {
    let input = vec!["Rust", " rust", "Go", "go ", "", "Zig", "RUST", "zig", "Odin"];
    let tags = unique_tags(&input);
    for (i, tag) in tags.iter().enumerate() {
        println!("{}: {}", i + 1, tag);
    }
    println!("total {}", tags.len());
}
