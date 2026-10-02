fn tail<'a>(lines: &'a [&'a str], n: usize) -> &'a [&'a str] {
    &lines[lines.len() - n..]
}

fn main() {
    let log = [
        "09:00:01 service started",
        "09:00:04 listening on port 8080",
        "09:01:13 GET /health 200",
        "09:02:40 GET /orders 200",
        "09:02:41 POST /orders 201",
        "09:03:05 GET /orders/17 404",
    ];
    for line in tail(&log, 3) {
        println!("{line}");
    }
    let errors: Vec<&str> = log.iter().copied().filter(|l| l.ends_with("404")).collect();
    for line in tail(&errors, 1) {
        println!("last error: {line}");
    }
}
