use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Currency {
    Eur,
    Usd,
    Gbp,
    Jpy,
}

impl Currency {
    fn from_code(code: &str) -> Option<Currency> {
        match code {
            "EUR" => Some(Currency::Eur),
            "USD" => Some(Currency::Usd),
            "GBP" => Some(Currency::Gbp),
            "JPY" => Some(Currency::Jpy),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum LedgerError {
    Format(String),
    Currency(String),
    Amount(String),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::Format(s) => write!(f, "bad entry '{s}'"),
            LedgerError::Currency(s) => write!(f, "unknown currency '{s}'"),
            LedgerError::Amount(s) => write!(f, "bad amount '{s}'"),
        }
    }
}

struct Entry<'a> {
    account: &'a str,
    amount_cents: i64,
    currency: Currency,
}

trait Converter {
    fn to_base(&self, cents: i64, from: Currency) -> i64;
}

struct RateTable {
    per_eur: HashMap<Currency, f64>,
}

impl RateTable {
    fn from_feed(feed: &str) -> Result<RateTable, LedgerError> {
        let mut per_eur = HashMap::new();
        per_eur.insert(Currency::Eur, 1.0);
        for line in feed.lines().filter(|l| !l.trim().is_empty()) {
            let (code, rate) = line.split_once(' ').ok_or_else(|| LedgerError::Format(line.to_string()))?;
            let cur = Currency::from_code(code).ok_or_else(|| LedgerError::Currency(code.to_string()))?;
            let rate: f64 = rate.trim().parse().map_err(|_| LedgerError::Amount(rate.to_string()))?;
            per_eur.insert(cur, rate);
        }
        Ok(RateTable { per_eur })
    }
}

impl Converter for RateTable {
    fn to_base(&self, cents: i64, from: Currency) -> i64 {
        let rate = self.per_eur.get(&from).expect("rate for currency");
        (cents as f64 / rate).round() as i64
    }
}

fn parse_amount(s: &str) -> Result<i64, LedgerError> {
    let value: f64 = s.parse().map_err(|_| LedgerError::Amount(s.to_string()))?;
    Ok((value * 100.0).round() as i64)
}

fn parse_entry(line: &str) -> Result<Entry<'_>, LedgerError> {
    let mut parts = line.split_whitespace();
    let (Some(account), Some(amount), Some(code), None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
        return Err(LedgerError::Format(line.to_string()));
    };
    let currency = Currency::from_code(code).ok_or_else(|| LedgerError::Currency(code.to_string()))?;
    Ok(Entry { account, amount_cents: parse_amount(amount)?, currency })
}

struct Ledger<'a, C: Converter> {
    converter: &'a C,
    balances: BTreeMap<&'a str, i64>,
    rejected: Vec<String>,
}

impl<'a, C: Converter> Ledger<'a, C> {
    fn new(converter: &'a C) -> Self {
        Ledger { converter, balances: BTreeMap::new(), rejected: Vec::new() }
    }

    fn post_all(&mut self, journal: &'a str) {
        for line in journal.lines().filter(|l| !l.trim().is_empty()) {
            match parse_entry(line) {
                Ok(e) => {
                    let base = self.converter.to_base(e.amount_cents, e.currency);
                    *self.balances.entry(e.account).or_insert(0) += base;
                }
                Err(err) => self.rejected.push(err.to_string()),
            }
        }
    }
}

fn main() {
    let feed = "USD 1.0850\nGBP 0.8590\n";
    let journal = "\
alice 120.50 USD
bob 99.99 EUR
alice -20 GBP
carol 410 GBP
bob 12.3 CHF
dave seven EUR
";
    let rates = match RateTable::from_feed(feed) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    let mut ledger = Ledger::new(&rates);
    ledger.post_all(journal);
    for (account, cents) in &ledger.balances {
        println!("{account:<6} {:>10.2} EUR", *cents as f64 / 100.0);
    }
    for r in &ledger.rejected {
        println!("rejected: {r}");
    }
}
