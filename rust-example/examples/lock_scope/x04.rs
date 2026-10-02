use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

struct PrimeTable {
    computed: Mutex<HashMap<u32, u32>>,
}

fn sieve(limit: u32) -> u32 {
    let size = limit as usize;
    let mut composite = vec![false; size + 1];
    let mut count = 0;
    for n in 2..=size {
        if !composite[n] {
            count += 1;
            let mut multiple = n * n;
            while multiple <= size {
                composite[multiple] = true;
                multiple += n;
            }
        }
    }
    count
}

impl PrimeTable {
    fn new() -> Self {
        PrimeTable { computed: Mutex::new(HashMap::new()) }
    }

    fn count_below(&self, limit: u32) -> u32 {
        let mut computed = self.computed.lock().unwrap();
        *computed.entry(limit).or_insert_with(|| sieve(limit))
    }

    fn known_limits(&self) -> usize {
        self.computed.lock().unwrap().len()
    }
}

fn main() {
    let table = Arc::new(PrimeTable::new());
    let mut workers = Vec::new();
    for limit in [100_000, 250_000, 100_000, 500_000] {
        let table = Arc::clone(&table);
        workers.push(thread::spawn(move || (limit, table.count_below(limit))));
    }
    for worker in workers {
        let (limit, count) = worker.join().unwrap();
        println!("{count} primes up to {limit}");
    }
    println!("cached {} limits", table.known_limits());
}
