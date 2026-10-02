use std::sync::{Arc, Mutex};
use std::thread;

fn checksum(seed: u64, rounds: u32) -> u64 {
    (0..rounds).fold(seed, |acc, r| {
        acc.wrapping_mul(6364136223846793005).wrapping_add(u64::from(r))
    })
}

fn main() {
    let latest = Arc::new(Mutex::new(17u64));
    let mut handles = Vec::new();
    for _ in 0..4 {
        let latest = Arc::clone(&latest);
        handles.push(thread::spawn(move || {
            let seed = *latest.lock().unwrap();
            let value = checksum(seed, 200_000);
            *latest.lock().unwrap() = value;
            value
        }));
    }
    for handle in handles {
        println!("{}", handle.join().unwrap());
    }
    println!("final {}", latest.lock().unwrap());
}
