use std::collections::HashMap;
use std::fmt;

struct Item {
    sku: String,
    qty: u32,
    price_cents: u64,
    note: String,
}

#[derive(Debug)]
enum RowError {
    MissingField(&'static str),
    BadNumber { field: &'static str, value: String },
}

impl fmt::Display for RowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RowError::MissingField(name) => write!(f, "missing field {name}"),
            RowError::BadNumber { field, value } => write!(f, "{field} is not a number: {value:?}"),
        }
    }
}

fn number<T: std::str::FromStr>(fields: &[&str], idx: usize, field: &'static str) -> Result<T, RowError> {
    let raw = fields.get(idx).ok_or(RowError::MissingField(field))?;
    raw.parse()
        .map_err(|_| RowError::BadNumber { field, value: raw.to_string() })
}

fn parse_row(row: &str) -> Result<Item, RowError> {
    let fields: Vec<&str> = row.split(',').map(str::trim).collect();
    let sku = fields.first().filter(|s| !s.is_empty()).ok_or(RowError::MissingField("sku"))?;
    let qty = number(&fields, 1, "qty")?;
    let price_cents = number(&fields, 2, "price_cents")?;
    let note = fields.get(3).copied().unwrap_or("").to_string();
    Ok(Item { sku: sku.to_string(), qty, price_cents, note })
}

struct Import {
    items: Vec<Item>,
    rejected: usize,
}

fn import(text: &str) -> Import {
    let mut items = Vec::new();
    let mut rejected = 0;
    for (n, row) in text.lines().enumerate().skip(1) {
        match parse_row(row) {
            Ok(item) => items.push(item),
            Err(e) => {
                rejected += 1;
                eprintln!("line {}: {e}", n + 1);
            }
        }
    }
    Import { items, rejected }
}

fn main() {
    let csv = "sku,qty,price_cents,note\nTEA-01,12,450,\nTEA-02,x,450,\nMUG-01,3,1299,chipped box\n,4,100\nMUG-02,7\nTEA-03,5,500";
    let result = import(csv);
    let mut value_by_line: HashMap<&str, u64> = HashMap::new();
    for item in &result.items {
        let line = item.sku.split('-').next().unwrap_or(&item.sku);
        *value_by_line.entry(line).or_insert(0) += u64::from(item.qty) * item.price_cents;
        if !item.note.is_empty() {
            println!("{}: {}", item.sku, item.note);
        }
    }
    let mut lines: Vec<_> = value_by_line.into_iter().collect();
    lines.sort();
    for (line, cents) in lines {
        println!("{line}: {}.{:02}", cents / 100, cents % 100);
    }
    println!("{} imported, {} rejected", result.items.len(), result.rejected);
}
