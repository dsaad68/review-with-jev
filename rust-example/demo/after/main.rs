fn last_word<'a>(words: &[&'a str]) -> &'a str {
    words[words.len() - 1]
}

fn parse_counts(text: &str) -> Vec<u32> {
    text.split(',').filter_map(|t| t.trim().parse().ok()).collect()
}

fn main() {
    let inputs = vec!["hello", "world", "foo bar", "rust", "copy on write"];

    let mut cleaned: Vec<String> = Vec::new();
    for i in 0..inputs.len() {
        let input = inputs[i];
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

    println!("last: {}", last_word(&inputs));
    println!("counts: {:?}", parse_counts("3, 7, x, 12"));
}
