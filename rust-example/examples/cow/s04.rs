fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    for c in title.trim().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_string()
}

fn main() {
    let titles = ["Hello World", "Rust 2021: What's New?", "  Spaces   everywhere "];
    for title in titles {
        println!("/blog/{}", slugify(title));
    }
}
