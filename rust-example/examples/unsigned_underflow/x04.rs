use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq)]
enum LedgerError {
    UnknownAccount(u32),
    InsufficientFunds { account: u32, balance: u64, requested: u64 },
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::UnknownAccount(id) => write!(f, "account {id} does not exist"),
            LedgerError::InsufficientFunds { account, balance, requested } => {
                write!(f, "account {account} has {balance} cents, wanted {requested}")
            }
        }
    }
}

struct Ledger {
    balances: HashMap<u32, u64>,
}

impl Ledger {
    fn deposit(&mut self, account: u32, cents: u64) {
        *self.balances.entry(account).or_insert(0) += cents;
    }

    fn withdraw(&mut self, account: u32, cents: u64) -> Result<u64, LedgerError> {
        let balance = self.balances.get_mut(&account).ok_or(LedgerError::UnknownAccount(account))?;
        let remaining = balance.checked_sub(cents).ok_or(LedgerError::InsufficientFunds {
            account,
            balance: *balance,
            requested: cents,
        })?;
        *balance = remaining;
        Ok(remaining)
    }

    fn transfer(&mut self, from: u32, to: u32, cents: u64) -> Result<(), LedgerError> {
        if from == to {
            return Ok(());
        }
        match self.balances.get_disjoint_mut([&from, &to]) {
            [Some(src), Some(dst)] => {
                *src = src.checked_sub(cents).ok_or(LedgerError::InsufficientFunds {
                    account: from,
                    balance: *src,
                    requested: cents,
                })?;
                *dst += cents;
                Ok(())
            }
            [None, _] => Err(LedgerError::UnknownAccount(from)),
            [_, None] => Err(LedgerError::UnknownAccount(to)),
        }
    }
}

fn main() {
    let mut ledger = Ledger { balances: HashMap::new() };
    ledger.deposit(1, 50_00);
    ledger.deposit(2, 12_50);
    let ops = [(1, 2, 20_00), (2, 1, 40_00), (1, 3, 1_00), (2, 1, 32_50)];
    for (from, to, cents) in ops {
        match ledger.transfer(from, to, cents) {
            Ok(()) => println!("moved {cents} cents from {from} to {to}"),
            Err(e) => println!("rejected: {e}"),
        }
    }
    match ledger.withdraw(2, 5_00) {
        Ok(left) => println!("cash withdrawal ok, {left} cents left"),
        Err(e) => println!("cash withdrawal rejected: {e}"),
    }
    let mut rows: Vec<(&u32, &u64)> = ledger.balances.iter().collect();
    rows.sort_unstable();
    println!("balances in cents: {rows:?}");
}
