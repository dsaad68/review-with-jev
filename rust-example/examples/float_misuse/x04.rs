use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Cents(i64);

impl fmt::Display for Cents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        write!(f, "{sign}${}.{:02}", abs / 100, abs % 100)
    }
}

#[derive(Debug)]
enum Entry {
    Deposit(Cents),
    Withdrawal(Cents),
    Interest { rate_bps: i64 },
}

struct Account {
    owner: &'static str,
    balance: Cents,
    history: Vec<Entry>,
}

impl Account {
    fn new(owner: &'static str) -> Self {
        Account { owner, balance: Cents(0), history: Vec::new() }
    }

    fn apply(&mut self, entry: Entry) {
        let delta = match &entry {
            Entry::Deposit(c) => c.0,
            Entry::Withdrawal(c) => -c.0,
            Entry::Interest { rate_bps } => {
                let raw = self.balance.0 * rate_bps;
                (raw + 5_000) / 10_000
            }
        };
        self.balance = Cents(self.balance.0 + delta);
        self.history.push(entry);
    }

    fn statement(&self) -> String {
        let mut out = format!("{}:\n", self.owner);
        for e in &self.history {
            let line = match e {
                Entry::Deposit(c) => format!("  deposit    {c}\n"),
                Entry::Withdrawal(c) => format!("  withdrawal {c}\n"),
                Entry::Interest { rate_bps } => {
                    format!("  interest   {}.{:02}%\n", rate_bps / 100, rate_bps % 100)
                }
            };
            out.push_str(&line);
        }
        out.push_str(&format!("  balance    {}\n", self.balance));
        out
    }
}

fn main() {
    let mut acct = Account::new("Noor");
    acct.apply(Entry::Deposit(Cents(120_050)));
    acct.apply(Entry::Withdrawal(Cents(33_333)));
    acct.apply(Entry::Interest { rate_bps: 125 });
    acct.apply(Entry::Deposit(Cents(10)));
    acct.apply(Entry::Deposit(Cents(20)));
    print!("{}", acct.statement());

    let expected = Cents(87_831);
    if acct.balance == expected {
        println!("balance reconciles");
    } else {
        println!("off by {}", Cents(acct.balance.0 - expected.0));
    }
}
