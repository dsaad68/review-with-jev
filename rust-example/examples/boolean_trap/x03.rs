struct WrapOptions {
    width: usize,
    indent_continuation: bool,
    number_lines: bool,
}
fn wrap(text: &str, opts: &WrapOptions) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > opts.width {
            lines.push(std::mem::take(&mut current));
            if opts.indent_continuation { current.push_str("  "); }
        } else if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    lines.push(current);
    if opts.number_lines {
        lines = lines.iter().enumerate().map(|(i, l)| format!("{:>2} {l}", i + 1)).collect();
    }
    lines
}

fn main() {
    let opts = WrapOptions { width: 24, indent_continuation: true, number_lines: false };
    for line in wrap("The wind was a torrent of darkness among the gusty trees", &opts) {
        println!("|{line}");
    }
}
