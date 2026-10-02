use std::collections::VecDeque;
use std::fmt;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

trait Reducer: Send + Sync + 'static {
    type Event: Send + 'static;
    type State: Default + Send + Sync + fmt::Display + 'static;
    fn reduce(&self, state: &mut Self::State, event: Self::Event);
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Deposit(u64),
    Withdraw(u64),
    Interest(u32),
}

#[derive(Default)]
struct Ledger {
    balance: i128,
    history: VecDeque<i128>,
    rejected: u32,
}

impl fmt::Display for Ledger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "balance={:>8} rejected={} last=[", self.balance, self.rejected)?;
        for (i, h) in self.history.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{h}")?;
        }
        f.write_str("]")
    }
}

struct Bank {
    overdraft: i128,
    window: usize,
}

impl Reducer for Bank {
    type Event = Op;
    type State = Ledger;

    fn reduce(&self, s: &mut Ledger, e: Op) {
        let next = match e {
            Op::Deposit(n) => s.balance + n as i128,
            Op::Withdraw(n) if s.balance - n as i128 >= -self.overdraft => s.balance - n as i128,
            Op::Withdraw(_) => {
                s.rejected += 1;
                return;
            }
            Op::Interest(bp) => s.balance + s.balance * bp as i128 / 10_000,
        };
        s.history.push_back(next - s.balance);
        if s.history.len() > self.window {
            s.history.pop_front();
        }
        s.balance = next;
    }
}

struct Engine<R: Reducer> {
    reducer: Arc<R>,
    shards: Vec<Arc<RwLock<R::State>>>,
}

impl<R: Reducer> Engine<R> {
    fn new(reducer: R, shards: usize) -> Self {
        Self {
            reducer: Arc::new(reducer),
            shards: (0..shards)
                .map(|_| Arc::new(RwLock::new(R::State::default())))
                .collect(),
        }
    }

    fn run<I>(&self, events: I) -> Vec<String>
    where
        I: IntoIterator<Item = (usize, R::Event)>,
    {
        let (log_tx, log_rx) = mpsc::channel::<String>();
        let senders: Vec<mpsc::Sender<R::Event>> = self
            .shards
            .iter()
            .enumerate()
            .map(|(id, shard)| {
                let (tx, rx) = mpsc::channel::<R::Event>();
                let shard = Arc::clone(shard);
                let reducer = Arc::clone(&self.reducer);
                let log = log_tx.clone();
                thread::spawn(move || {
                    let mut applied = 0usize;
                    for event in rx {
                        reducer.reduce(&mut shard.write().unwrap(), event);
                        applied += 1;
                    }
                    let line = format!("shard {id}: {applied:>2} events -> {}", shard.read().unwrap());
                    log.send(line).unwrap();
                });
                tx
            })
            .collect();
        drop(log_tx);
        for (key, event) in events {
            senders[key % senders.len()].send(event).unwrap();
        }
        drop(senders);
        let mut lines: Vec<String> = log_rx.into_iter().collect();
        lines.sort();
        lines
    }

    fn fold<T>(&self, init: T, f: impl Fn(T, &R::State) -> T) -> T {
        self.shards
            .iter()
            .fold(init, |acc, s| f(acc, &s.read().unwrap()))
    }
}

fn main() {
    let engine = Engine::new(Bank { overdraft: 500, window: 4 }, 3);
    let events = (0..60u64).map(|i| {
        let op = match i % 4 {
            0 => Op::Deposit(100 + i * 3),
            1 => Op::Withdraw(250),
            2 => Op::Interest(125),
            _ => Op::Withdraw(40 * i),
        };
        ((i * 7) as usize, op)
    });
    for line in engine.run(events) {
        println!("{line}");
    }
    let (total, rejected) = engine.fold((0i128, 0u32), |(t, r), s| (t + s.balance, r + s.rejected));
    println!("total={total} rejected={rejected}");
}
