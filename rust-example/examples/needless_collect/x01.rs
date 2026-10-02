fn distinct_tags(raw: &str) -> Vec<String> {
    let mut tags: Vec<String> = raw
        .split(',')
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

fn main() {
    let input = "Rust, systems,  rust, Performance, , SYSTEMS, safety";
    let tags = distinct_tags(input);
    println!("{} distinct tags", tags.len());
    for tag in &tags {
        println!("- {tag}");
    }
}
