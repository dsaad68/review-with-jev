use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

trait Clock: Send + Sync {
    fn now_ms(&self) -> u64;
}

struct SteppingClock {
    base: u64,
    step: u64,
    ticks: AtomicU64,
}

impl Clock for SteppingClock {
    fn now_ms(&self) -> u64 {
        self.base + self.ticks.fetch_add(1, Ordering::Relaxed) * self.step
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Decision {
    Allowed,
    Throttled,
}

struct Limiter<C: Clock> {
    clock: C,
    window_ms: u64,
    max_hits: usize,
    hits: Mutex<HashMap<String, VecDeque<u64>>>,
}

impl<C: Clock> Limiter<C> {
    fn new(clock: C, window_ms: u64, max_hits: usize) -> Self {
        Limiter {
            clock,
            window_ms,
            max_hits,
            hits: Mutex::new(HashMap::new()),
        }
    }

    fn check(&self, client: &str) -> Decision {
        let now = self.clock.now_ms();
        let cutoff = now - self.window_ms;
        let mut hits = self.hits.lock().unwrap();
        let recent = hits.entry(client.to_string()).or_default();
        while recent.front().is_some_and(|&t| t <= cutoff) {
            recent.pop_front();
        }
        if recent.len() < self.max_hits {
            recent.push_back(now);
            Decision::Allowed
        } else {
            Decision::Throttled
        }
    }

    fn retry_after_ms(&self, client: &str) -> Option<u64> {
        let now = self.clock.now_ms();
        let hits = self.hits.lock().unwrap();
        let recent = hits.get(client)?;
        if recent.len() < self.max_hits {
            return None;
        }
        let oldest = *recent.front()?;
        Some((oldest + self.window_ms).saturating_sub(now))
    }
}

fn simulate<C: Clock + 'static>(
    limiter: Arc<Limiter<C>>,
    clients: &[&'static str],
    requests_per_thread: usize,
) -> HashMap<(&'static str, Decision), u32> {
    let handles: Vec<_> = clients
        .iter()
        .map(|&client| {
            let limiter = Arc::clone(&limiter);
            thread::spawn(move || {
                let mut tally: HashMap<(&'static str, Decision), u32> = HashMap::new();
                for _ in 0..requests_per_thread {
                    *tally.entry((client, limiter.check(client))).or_insert(0) += 1;
                }
                tally
            })
        })
        .collect();
    let mut merged = HashMap::new();
    for h in handles {
        for (key, n) in h.join().unwrap() {
            *merged.entry(key).or_insert(0) += n;
        }
    }
    merged
}

fn main() {
    let clock = SteppingClock {
        base: 50_000,
        step: 3,
        ticks: AtomicU64::new(0),
    };
    let limiter = Arc::new(Limiter::new(clock, 1_000, 40));
    let clients = ["mobile-app", "partner-api", "batch-export"];
    let tally = simulate(Arc::clone(&limiter), &clients, 120);
    let mut rows: Vec<_> = tally.into_iter().collect();
    rows.sort_by_key(|&((client, decision), _)| (client, decision == Decision::Throttled));
    for ((client, decision), n) in rows {
        println!("{client:<14} {decision:?}: {n}");
    }
    for client in clients {
        match limiter.retry_after_ms(client) {
            Some(ms) => println!("{client} may retry in {ms}ms"),
            None => println!("{client} has spare capacity"),
        }
    }
}
