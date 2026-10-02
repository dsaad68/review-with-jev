struct Span {
    name: &'static str,
    start_us: u64,
    end_us: u64,
}

fn duration_us(span: &Span) -> u64 {
    span.end_us - span.start_us
}

fn main() {
    let spans = [
        Span { name: "parse", start_us: 1_000, end_us: 1_850 },
        Span { name: "plan", start_us: 1_850, end_us: 2_100 },
        Span { name: "execute", start_us: 2_100, end_us: 9_400 },
        Span { name: "serialize", start_us: 9_400, end_us: 9_760 },
    ];
    let total: u64 = spans.iter().map(duration_us).sum();
    for s in &spans {
        println!("{:<10} {:>6}us", s.name, duration_us(s));
    }
    if let Some(slowest) = spans.iter().max_by_key(|s| duration_us(s)) {
        println!("slowest stage: {}", slowest.name);
    }
    println!("total {total}us");
}
