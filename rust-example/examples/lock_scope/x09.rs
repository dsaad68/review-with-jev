use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

trait Task: Send + 'static {
    type Output: Send + 'static;
    fn label(&self) -> &str;
    fn run(self) -> Self::Output;
}

struct Digest {
    name: String,
    payload: Arc<[u8]>,
    rounds: u32,
}

impl Task for Digest {
    type Output = u64;

    fn label(&self) -> &str {
        &self.name
    }

    fn run(self) -> u64 {
        let mut state: u64 = 0xcbf2_9ce4_8422_2325;
        for _ in 0..self.rounds {
            for &byte in self.payload.iter() {
                state ^= u64::from(byte);
                state = state.wrapping_mul(0x0100_0000_01b3);
            }
        }
        state
    }
}

struct Progress {
    message: String,
}

struct Pool<T: Task> {
    running: Mutex<HashMap<String, JoinHandle<T::Output>>>,
    finished: Mutex<Vec<(String, T::Output)>>,
}

impl<T: Task> Pool<T> {
    fn new() -> Self {
        Pool { running: Mutex::new(HashMap::new()), finished: Mutex::new(Vec::new()) }
    }

    fn spawn(&self, task: T, progress: Sender<Progress>) {
        let key = task.label().to_string();
        let handle = thread::spawn(move || {
            let label = task.label().to_string();
            let output = task.run();
            let _ = progress.send(Progress { message: format!("{label} done") });
            output
        });
        self.running.lock().unwrap().insert(key, handle);
    }

    fn wait_all(&self) -> usize {
        let mut running = self.running.lock().unwrap();
        let mut completed = 0;
        for (name, handle) in running.drain() {
            let output = handle.join().unwrap();
            self.finished.lock().unwrap().push((name, output));
            completed += 1;
        }
        completed
    }

    fn pending(&self) -> usize {
        self.running.lock().unwrap().len()
    }

    fn results(&self) -> Vec<(String, T::Output)>
    where
        T::Output: Clone,
    {
        let mut out = self.finished.lock().unwrap().clone();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

fn main() {
    let pool: Arc<Pool<Digest>> = Arc::new(Pool::new());
    let (tx, rx) = mpsc::channel();
    let listener = thread::spawn(move || rx.iter().map(|p: Progress| p.message).collect::<Vec<_>>());

    let corpus: Vec<(&str, Vec<u8>)> = vec![
        ("alpha", b"the quick brown fox".to_vec()),
        ("beta", b"jumps over the lazy dog".to_vec()),
        ("gamma", (0u8..=255).collect()),
    ];

    for (name, bytes) in corpus {
        let payload: Arc<[u8]> = Arc::from(bytes);
        for rounds in [1_000, 2_000] {
            pool.spawn(
                Digest { name: format!("{name}-{rounds}"), payload: Arc::clone(&payload), rounds },
                tx.clone(),
            );
        }
    }
    drop(tx);

    println!("pending before wait: {}", pool.pending());
    let completed = pool.wait_all();
    println!("completed {completed}");
    for (name, digest) in pool.results() {
        println!("{name}: {digest:016x}");
    }
    let mut messages = listener.join().unwrap();
    messages.sort();
    println!("{} progress messages, first: {:?}", messages.len(), messages.first());
}
