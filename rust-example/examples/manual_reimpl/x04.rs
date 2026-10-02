#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    Gateway,
    Database,
}

#[derive(Debug, Clone)]
struct Event {
    at_ms: u64,
    source: Source,
    message: String,
}

fn interleave(left: Vec<Event>, right: Vec<Event>) -> Vec<Event> {
    let mut merged = Vec::with_capacity(left.len() + right.len());
    let mut a = left.into_iter().peekable();
    let mut b = right.into_iter().peekable();
    loop {
        let take_left = match (a.peek(), b.peek()) {
            (Some(x), Some(y)) => x.at_ms <= y.at_ms,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => break,
        };
        let next = if take_left { a.next() } else { b.next() };
        if let Some(event) = next {
            merged.push(event);
        }
    }
    merged
}

fn ev(at_ms: u64, source: Source, message: &str) -> Event {
    Event {
        at_ms,
        source,
        message: message.to_string(),
    }
}

fn main() {
    let gateway = vec![
        ev(100, Source::Gateway, "request received"),
        ev(340, Source::Gateway, "upstream timeout"),
        ev(910, Source::Gateway, "response 504"),
    ];
    let database = vec![
        ev(120, Source::Database, "query start"),
        ev(335, Source::Database, "lock wait"),
        ev(880, Source::Database, "query cancelled"),
        ev(1200, Source::Database, "vacuum"),
    ];
    for e in interleave(gateway, database) {
        println!("{:>5} {:?}: {}", e.at_ms, e.source, e.message);
    }
}
