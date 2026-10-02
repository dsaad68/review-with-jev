use std::borrow::Cow;

fn escape_html(input: &str) -> Cow<'_, str> {
    if !input.contains(['<', '>', '&']) {
        return Cow::Borrowed(input);
    }
    let mut out = String::with_capacity(input.len() + 8);
    for c in input.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

fn main() {
    let comments = ["Nice post!", "a < b && c > d", "plain text"];
    for comment in comments {
        println!("<p>{}</p>", escape_html(comment));
    }
}
