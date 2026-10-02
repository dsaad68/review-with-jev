use std::collections::HashMap;

struct Session {
    user: String,
    last_seen: u64,
    requests: u32,
}

struct SessionStore {
    sessions: HashMap<u64, Session>,
}

impl SessionStore {
    fn new() -> Self {
        SessionStore { sessions: HashMap::new() }
    }

    fn touch(&mut self, id: u64, user: &str, now: u64) {
        let entry = self.sessions.entry(id).or_insert_with(|| Session {
            user: user.to_string(),
            last_seen: now,
            requests: 0,
        });
        entry.last_seen = now;
        entry.requests += 1;
    }

    fn evict_idle(&mut self, now: u64, ttl: u64) -> u32 {
        let stale: Vec<u64> = self
            .sessions
            .iter()
            .filter(|(_, s)| now.saturating_sub(s.last_seen) > ttl)
            .map(|(id, _)| *id)
            .collect();
        let mut dropped_requests = 0;
        for id in stale {
            if let Some(s) = self.sessions.remove(&id) {
                println!("evicting {} after {} requests", s.user, s.requests);
                dropped_requests += s.requests;
            }
        }
        dropped_requests
    }
}

fn main() {
    let mut store = SessionStore::new();
    store.touch(1, "ana", 100);
    store.touch(2, "bo", 105);
    store.touch(1, "ana", 110);
    store.touch(3, "cy", 180);
    store.touch(2, "bo", 120);

    let dropped = store.evict_idle(200, 60);
    println!("dropped {dropped} requests, {} sessions remain", store.sessions.len());
}
