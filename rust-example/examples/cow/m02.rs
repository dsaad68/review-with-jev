use std::borrow::Cow;

#[derive(Debug)]
struct Comment {
    author: String,
    body: String,
}

fn escape_html(input: &str) -> Cow<'_, str> {
    if !input.contains(|c| matches!(c, '<' | '>' | '&' | '"')) {
        return Cow::Borrowed(input);
    }
    let mut out = String::with_capacity(input.len() + 16);
    for c in input.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    Cow::Owned(out)
}

fn render(comment: &Comment) -> String {
    format!(
        "<div class=\"comment\"><b>{}</b><p>{}</p></div>",
        escape_html(&comment.author),
        escape_html(&comment.body)
    )
}

fn render_thread(comments: &[Comment]) -> String {
    comments
        .iter()
        .map(render)
        .collect::<Vec<_>>()
        .join("\n")
}

fn count_escaped(comments: &[Comment]) -> usize {
    comments
        .iter()
        .filter(|c| matches!(escape_html(&c.body), Cow::Owned(_)))
        .count()
}

fn main() {
    let comments = vec![
        Comment { author: "alice".into(), body: "Looks good to me".into() },
        Comment { author: "bob".into(), body: "Use <b> & <i> tags".into() },
        Comment { author: "carol \"cj\"".into(), body: "Agreed".into() },
    ];
    println!("{}", render_thread(&comments));
    println!("bodies needing escape: {}", count_escaped(&comments));
    println!("{:?}", comments[0]);
}
