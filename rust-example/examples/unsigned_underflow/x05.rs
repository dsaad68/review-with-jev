struct Hit<'a> {
    line_no: usize,
    context: &'a [&'a str],
}

fn search<'a>(lines: &'a [&'a str], needle: &str, before: usize, after: usize) -> Vec<Hit<'a>> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains(needle))
        .map(|(idx, _)| {
            let start = idx - before;
            let end = (idx + after + 1).min(lines.len());
            Hit {
                line_no: idx + 1,
                context: &lines[start..end],
            }
        })
        .collect()
}

fn render(hits: &[Hit<'_>]) -> String {
    let mut out = String::new();
    for hit in hits {
        out.push_str(&format!("-- match at line {} --\n", hit.line_no));
        for line in hit.context {
            out.push_str("   ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn main() {
    let source = "worker_processes 4;\nevents {\n    worker_connections 1024;\n}\nhttp {\n    keepalive_timeout 65;\n    server {\n        listen 8080;\n        root /srv/www;\n        proxy_read_timeout 30;\n    }\n}\n";
    let lines: Vec<&str> = source.lines().collect();
    let hits = search(&lines, "listen", 3, 1);
    print!("{}", render(&hits));
    let hits = search(&lines, "timeout", 2, 2);
    print!("{}", render(&hits));
}
