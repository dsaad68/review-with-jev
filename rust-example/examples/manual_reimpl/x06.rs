#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Level {
    Info,
    Warn,
    Error,
}

struct Entry<'a> {
    level: Level,
    route: &'a str,
    latency_ms: u32,
}

#[derive(Debug, Default)]
struct Digest<'a> {
    errors: usize,
    warned_routes: Vec<&'a str>,
    slowest: Option<(&'a str, u32)>,
    total_latency_ms: u64,
}

fn digest<'a>(entries: &[Entry<'a>]) -> Digest<'a> {
    let mut d = Digest::default();
    for e in entries {
        match e.level {
            Level::Error => d.errors += 1,
            Level::Warn => d.warned_routes.push(e.route),
            Level::Info => {}
        }
        if d.slowest.map_or(true, |(_, ms)| e.latency_ms > ms) {
            d.slowest = Some((e.route, e.latency_ms));
        }
        d.total_latency_ms += u64::from(e.latency_ms);
    }
    d
}

fn main() {
    let entries = [
        Entry { level: Level::Info, route: "/login", latency_ms: 42 },
        Entry { level: Level::Warn, route: "/search", latency_ms: 870 },
        Entry { level: Level::Error, route: "/checkout", latency_ms: 1203 },
        Entry { level: Level::Info, route: "/cart", latency_ms: 65 },
        Entry { level: Level::Warn, route: "/profile", latency_ms: 310 },
        Entry { level: Level::Error, route: "/checkout", latency_ms: 2210 },
    ];
    let d = digest(&entries);
    println!("errors: {}", d.errors);
    println!("warnings on: {:?}", d.warned_routes);
    if let Some((route, ms)) = d.slowest {
        println!("slowest: {route} at {ms} ms");
    }
    println!("mean latency: {} ms", d.total_latency_ms / entries.len() as u64);
}
