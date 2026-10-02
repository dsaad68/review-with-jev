fn main() {
    let inputs = vec!["hello", "world", "foo bar", "rust", "copy on write"];

    let mut cleaned: Vec<String> = Vec::new();
    for input in &inputs {
        let result = if input.contains(' ') {
            input.replace(' ', "_")
        } else {
            input.to_string()
        };
        cleaned.push(result);
    }

    for s in &cleaned {
        println!("{}", s);
    }
}
