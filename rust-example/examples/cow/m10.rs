struct Sanitizer {
    replacements: Vec<(&'static str, &'static str)>,
    max_len: usize,
}

impl Sanitizer {
    fn new(max_len: usize) -> Self {
        Sanitizer {
            replacements: vec![
                ("\u{201c}", "\""),
                ("\u{201d}", "\""),
                ("\u{2019}", "'"),
                ("\u{2014}", "-"),
                ("\t", " "),
            ],
            max_len,
        }
    }

    fn clean(&self, input: &str) -> String {
        self.replacements
            .iter()
            .fold(input.to_string(), |acc, (from, to)| {
                if acc.contains(from) {
                    acc.replace(from, to)
                } else {
                    acc
                }
            })
    }

    fn truncate<'a>(&self, text: &'a str) -> &'a str {
        match text.char_indices().nth(self.max_len) {
            Some((idx, _)) => &text[..idx],
            None => text,
        }
    }

    fn process(&self, lines: &[&str]) -> Vec<String> {
        lines
            .iter()
            .map(|l| self.clean(l))
            .map(|c| self.truncate(&c).to_string())
            .collect()
    }
}

fn main() {
    let s = Sanitizer::new(24);
    let lines = [
        "plain ascii line",
        "\u{201c}Smart\u{201d} quotes \u{2014} and dashes",
        "it\u{2019}s\ttabbed",
        "a very long line that will definitely be cut short",
    ];
    for (orig, out) in lines.iter().zip(s.process(&lines)) {
        println!("{:?} -> {:?}", orig, out);
    }
    let unchanged = lines.iter().filter(|l| s.clean(l) == **l).count();
    println!("unchanged: {}", unchanged);
}
