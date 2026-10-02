use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

trait Source<K, V> {
    fn load(&self, key: &K) -> Option<V>;
}

struct Slot<V> {
    value: V,
    hits: u32,
    dirty: bool,
}

struct Cache<K, V, S> {
    entries: Mutex<HashMap<K, Slot<V>>>,
    source: S,
    misses: AtomicUsize,
}

impl<K, V, S> Cache<K, V, S>
where
    K: Eq + Hash + Clone,
    V: Clone,
    S: Source<K, V>,
{
    fn new(source: S) -> Self {
        Cache { entries: Mutex::new(HashMap::new()), source, misses: AtomicUsize::new(0) }
    }

    fn get(&self, key: &K, refresh: bool, track: bool) -> Option<V> {
        if !refresh {
            let mut entries = self.entries.lock().unwrap();
            if let Some(slot) = entries.get_mut(key) {
                if track {
                    slot.hits += 1;
                }
                return Some(slot.value.clone());
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        let loaded = self.source.load(key)?;
        let mut entries = self.entries.lock().unwrap();
        let slot = match entries.entry(key.clone()) {
            Entry::Occupied(occupied) => {
                let slot = occupied.into_mut();
                slot.value = loaded;
                slot.dirty = false;
                slot
            }
            Entry::Vacant(vacant) => vacant.insert(Slot { value: loaded, hits: 0, dirty: false }),
        };
        if track {
            slot.hits += 1;
        }
        Some(slot.value.clone())
    }

    fn put(&self, key: K, value: V, overwrite: bool, mark_dirty: bool) -> bool {
        let mut entries = self.entries.lock().unwrap();
        match entries.entry(key) {
            Entry::Occupied(_) if !overwrite => false,
            Entry::Occupied(mut occupied) => {
                let slot = occupied.get_mut();
                slot.value = value;
                slot.dirty = mark_dirty;
                true
            }
            Entry::Vacant(vacant) => {
                vacant.insert(Slot { value, hits: 0, dirty: mark_dirty });
                true
            }
        }
    }

    fn stats(&self) -> (usize, u32, usize) {
        let entries = self.entries.lock().unwrap();
        let hits = entries.values().map(|e| e.hits).sum();
        let dirty = entries.values().filter(|e| e.dirty).count();
        (self.misses.load(Ordering::Relaxed), hits, dirty)
    }
}

struct PriceBook {
    prices: HashMap<u32, u64>,
    lookups: AtomicUsize,
}

impl Source<u32, u64> for PriceBook {
    fn load(&self, key: &u32) -> Option<u64> {
        self.lookups.fetch_add(1, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(2));
        self.prices.get(key).copied()
    }
}

fn main() {
    let book = PriceBook {
        prices: [(101, 1_999), (102, 450), (103, 12_000), (104, 75)].into_iter().collect(),
        lookups: AtomicUsize::new(0),
    };
    let cache = Cache::new(book);

    thread::scope(|scope| {
        for worker in 0..4u32 {
            let cache = &cache;
            scope.spawn(move || {
                for round in 0..5u32 {
                    let sku = 101 + (worker + round) % 5;
                    match cache.get(&sku, false, true) {
                        Some(cents) => println!("w{worker} sku {sku}: {cents}"),
                        None => println!("w{worker} sku {sku}: no price"),
                    }
                }
            });
        }
    });

    let replaced = cache.put(102, 499, true, true);
    let kept = cache.put(103, 1, false, true);
    let added = cache.put(200, 5_000, false, false);
    println!("replaced={replaced} kept_original={} added={added}", !kept);

    let fresh = cache.get(&102, true, false);
    println!("refreshed 102 -> {fresh:?}");

    let (misses, hits, dirty) = cache.stats();
    println!(
        "misses {misses}, hits {hits}, dirty {dirty}, source lookups {}",
        cache.source.lookups.load(Ordering::Relaxed)
    );
}
