use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct AccountId(u32);

#[derive(Debug, Clone, Copy, PartialEq)]
enum TransferError {
    UnknownAccount(AccountId),
    InsufficientFunds { needed: f64, available: f64 },
    SameAccount,
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferError::UnknownAccount(id) => write!(f, "unknown account {}", id.0),
            TransferError::InsufficientFunds { needed, available } => {
                write!(f, "needs {needed:.2}, has {available:.2}")
            }
            TransferError::SameAccount => write!(f, "source and destination are the same"),
        }
    }
}

impl std::error::Error for TransferError {}

trait FeeSchedule: Send + Sync {
    fn fee_for(&self, amount: f64) -> f64;
}

struct PercentFee {
    percent: f64,
    minimum: f64,
}

impl FeeSchedule for PercentFee {
    fn fee_for(&self, amount: f64) -> f64 {
        (amount * self.percent / 100.0).max(self.minimum)
    }
}

struct Bank<F: FeeSchedule> {
    balances: Mutex<HashMap<AccountId, f64>>,
    collected_fees: Mutex<f64>,
    fees: F,
}

impl<F: FeeSchedule> Bank<F> {
    fn new(fees: F, opening: &[(u32, f64)]) -> Self {
        let balances = opening.iter().map(|&(id, amt)| (AccountId(id), amt)).collect();
        Bank { balances: Mutex::new(balances), collected_fees: Mutex::new(0.0), fees }
    }

    fn transfer(&self, from: AccountId, to: AccountId, amount: f64) -> Result<f64, TransferError> {
        if from == to {
            return Err(TransferError::SameAccount);
        }
        let fee = self.fees.fee_for(amount);
        let needed = amount + fee;
        {
            let mut balances = self.balances.lock().expect("balances poisoned");
            let [source, dest] = balances.get_disjoint_mut([&from, &to]);
            let source = source.ok_or(TransferError::UnknownAccount(from))?;
            let dest = dest.ok_or(TransferError::UnknownAccount(to))?;
            if *source < needed {
                return Err(TransferError::InsufficientFunds { needed, available: *source });
            }
            *source -= needed;
            *dest += amount;
        }
        *self.collected_fees.lock().expect("fees poisoned") += fee;
        Ok(fee)
    }

    fn total_holdings(&self) -> f64 {
        let held: f64 = self.balances.lock().expect("balances poisoned").values().sum();
        held + *self.collected_fees.lock().expect("fees poisoned")
    }

    fn snapshot(&self) -> Vec<(AccountId, f64)> {
        let mut rows: Vec<(AccountId, f64)> =
            self.balances.lock().expect("balances poisoned").iter().map(|(k, v)| (*k, *v)).collect();
        rows.sort_by_key(|r| r.0);
        rows
    }
}

fn main() {
    let bank = Arc::new(Bank::new(
        PercentFee { percent: 0.35, minimum: 0.25 },
        &[(1, 1_000.10), (2, 250.20), (3, 75.30), (4, 0.0)],
    ));
    let opening_total = bank.total_holdings();

    let plans: Vec<Vec<(u32, u32, f64)>> = vec![
        vec![(1, 2, 19.99), (1, 3, 120.05), (2, 4, 33.33)],
        vec![(3, 1, 10.10), (2, 2, 5.00), (4, 9, 1.00)],
        vec![(1, 4, 0.10), (3, 4, 200.00), (2, 1, 0.70)],
    ];

    let handles: Vec<_> = plans
        .into_iter()
        .map(|plan| {
            let bank = Arc::clone(&bank);
            thread::spawn(move || {
                let mut log = Vec::new();
                for (from, to, amount) in plan {
                    let outcome = bank.transfer(AccountId(from), AccountId(to), amount);
                    log.push(match outcome {
                        Ok(fee) => format!("{from}->{to} {amount:.2} ok (fee {fee:.2})"),
                        Err(e) => format!("{from}->{to} {amount:.2} rejected: {e}"),
                    });
                }
                log
            })
        })
        .collect();

    let mut lines: Vec<String> = Vec::new();
    for h in handles {
        lines.extend(h.join().expect("teller thread panicked"));
    }
    lines.sort();
    for line in &lines {
        println!("{line}");
    }

    for (id, balance) in bank.snapshot() {
        println!("account {}: {balance:.2}", id.0);
    }
    let closing_total = bank.total_holdings();
    println!("opening {opening_total:.2} closing {closing_total:.2} balanced={}", opening_total == closing_total);
}
