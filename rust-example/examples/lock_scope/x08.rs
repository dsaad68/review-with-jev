use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AccountId(u32);

#[derive(Debug, PartialEq, Eq)]
enum TransferError {
    UnknownAccount(AccountId),
    InsufficientFunds { available: i64, requested: i64 },
    SameAccount,
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferError::UnknownAccount(id) => write!(f, "unknown account {}", id.0),
            TransferError::InsufficientFunds { available, requested } => {
                write!(f, "insufficient funds: {available} < {requested}")
            }
            TransferError::SameAccount => write!(f, "cannot transfer to the same account"),
        }
    }
}

impl std::error::Error for TransferError {}

#[derive(Debug, Clone)]
struct JournalEntry<K> {
    from: K,
    to: K,
    cents: i64,
}

trait Journal<K> {
    fn record(&mut self, entry: JournalEntry<K>);
    fn entries(&self) -> &[JournalEntry<K>];
}

struct MemoryJournal<K> {
    rows: Vec<JournalEntry<K>>,
}

impl<K> MemoryJournal<K> {
    fn new() -> Self {
        MemoryJournal { rows: Vec::new() }
    }
}

impl<K> Journal<K> for MemoryJournal<K> {
    fn record(&mut self, entry: JournalEntry<K>) {
        self.rows.push(entry);
    }

    fn entries(&self) -> &[JournalEntry<K>] {
        &self.rows
    }
}

struct Ledger<K, J> {
    balances: Mutex<HashMap<K, i64>>,
    journal: Mutex<J>,
}

impl<K, J> Ledger<K, J>
where
    K: Copy + Eq + Hash + Into<AccountId>,
    J: Journal<K>,
{
    fn new(opening: HashMap<K, i64>, journal: J) -> Self {
        Ledger { balances: Mutex::new(opening), journal: Mutex::new(journal) }
    }

    fn transfer(&self, from: K, to: K, cents: i64) -> Result<(), TransferError> {
        if from == to {
            return Err(TransferError::SameAccount);
        }
        let mut balances = self.balances.lock().unwrap();
        let [source, target] = balances.get_disjoint_mut([&from, &to]);
        let source = source.ok_or(TransferError::UnknownAccount(from.into()))?;
        let target = target.ok_or(TransferError::UnknownAccount(to.into()))?;
        if *source < cents {
            return Err(TransferError::InsufficientFunds { available: *source, requested: cents });
        }
        *source -= cents;
        *target += cents;
        let mut journal = self.journal.lock().unwrap();
        journal.record(JournalEntry { from, to, cents });
        Ok(())
    }

    fn balance(&self, account: K) -> Option<i64> {
        self.balances.lock().unwrap().get(&account).copied()
    }

    fn sent_by(&self, account: K) -> (usize, i64) {
        let journal = self.journal.lock().unwrap();
        journal
            .entries()
            .iter()
            .filter(|entry| entry.from == account && entry.to != account)
            .fold((0, 0), |(count, total), entry| (count + 1, total + entry.cents))
    }
}

impl From<AccountId> for u32 {
    fn from(id: AccountId) -> u32 {
        id.0
    }
}

fn main() {
    let ids: Vec<AccountId> = (1..=4).map(AccountId).collect();
    let opening: HashMap<AccountId, i64> = ids.iter().map(|&id| (id, 10_000)).collect();
    let ledger = Arc::new(Ledger::new(opening, MemoryJournal::new()));

    let mut tellers = Vec::new();
    for (n, &from) in ids.iter().enumerate() {
        let ledger = Arc::clone(&ledger);
        let to = ids[(n + 1) % ids.len()];
        tellers.push(thread::spawn(move || {
            let mut failures = Vec::new();
            for round in 0..50 {
                if let Err(err) = ledger.transfer(from, to, 150 + round) {
                    failures.push(err);
                }
            }
            failures
        }));
    }

    for teller in tellers {
        for failure in teller.join().unwrap() {
            println!("rejected: {failure}");
        }
    }

    for &id in &ids {
        let (count, total) = ledger.sent_by(id);
        println!("account {} -> {:?}, sent {count} transfers totalling {total}", u32::from(id), ledger.balance(id));
    }
}
