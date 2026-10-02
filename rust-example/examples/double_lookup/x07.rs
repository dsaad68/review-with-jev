use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct Bucket {
    tokens: u32,
    last_refill: u64,
}

#[derive(Debug, PartialEq)]
enum Verdict {
    Allowed,
    Throttled,
    Banned,
}

struct Limiter {
    capacity: u32,
    refill_every: u64,
    buckets: HashMap<String, Bucket>,
    banned: HashSet<String>,
}

impl Limiter {
    fn new(capacity: u32, refill_every: u64) -> Self {
        Limiter { capacity, refill_every, buckets: HashMap::new(), banned: HashSet::new() }
    }

    fn ban(&mut self, client: &str) {
        self.banned.insert(client.to_string());
        self.buckets.remove(client);
    }

    fn check(&mut self, client: &str, now: u64) -> Verdict {
        if self.banned.contains(client) {
            return Verdict::Banned;
        }
        let capacity = self.capacity;
        let bucket = self
            .buckets
            .entry(client.to_string())
            .or_insert(Bucket { tokens: capacity, last_refill: now });

        let elapsed = now.saturating_sub(bucket.last_refill);
        if elapsed >= self.refill_every {
            bucket.tokens = capacity;
            bucket.last_refill = now;
        }
        if bucket.tokens == 0 {
            return Verdict::Throttled;
        }
        bucket.tokens -= 1;
        Verdict::Allowed
    }
}

fn main() {
    let mut limiter = Limiter::new(2, 10);
    let traffic = [
        ("alice", 0),
        ("alice", 1),
        ("alice", 2),
        ("bob", 3),
        ("alice", 12),
        ("mallory", 13),
    ];
    for (client, t) in traffic {
        println!("t={t:>2} {client:<8} {:?}", limiter.check(client, t));
    }
    limiter.ban("mallory");
    println!("t=14 mallory  {:?}", limiter.check("mallory", 14));
    println!("tracked clients: {}", limiter.buckets.len());
}
