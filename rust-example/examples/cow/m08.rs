#[derive(Debug)]
struct Endpoint {
    name: String,
    url: String,
}

fn with_scheme(raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else {
        format!("https://{}", raw)
    }
}

fn host_of(url: &str) -> &str {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    rest.split(|c| c == '/' || c == '?' || c == ':').next().unwrap_or(rest)
}

fn is_internal(url: &str, domains: &[&str]) -> bool {
    let host = host_of(url);
    domains.iter().any(|d| host == *d || host.ends_with(&format!(".{}", d)))
}

fn build(entries: &[(&str, &str)]) -> Vec<Endpoint> {
    entries
        .iter()
        .map(|(name, raw)| Endpoint {
            name: name.to_string(),
            url: with_scheme(raw.trim()),
        })
        .collect()
}

fn main() {
    let entries = [
        ("api", "https://api.example.com/v1"),
        ("docs", "docs.example.com"),
        ("status", "http://status.other.io:8080/health"),
        ("cdn", "cdn.example.com/assets?v=2"),
    ];
    let endpoints = build(&entries);
    let internal = ["example.com"];
    for e in &endpoints {
        println!(
            "{:<7} {:<40} host={} internal={}",
            e.name,
            e.url,
            host_of(&e.url),
            is_internal(&e.url, &internal)
        );
    }
    let probe = with_scheme("https://already.example.com");
    println!("{} ({} bytes)", probe, probe.len());
}
