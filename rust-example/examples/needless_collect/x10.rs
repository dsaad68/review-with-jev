use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Sku(u32);

impl fmt::Display for Sku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SKU-{:05}", self.0)
    }
}

#[derive(Debug)]
struct Item {
    sku: Sku,
    name: String,
    warehouse: &'static str,
    quantity: u32,
    reorder_at: u32,
}

trait Ledger {
    fn adjust(&mut self, sku: Sku, delta: i64) -> Result<u32, String>;
}

struct Inventory {
    items: HashMap<Sku, Item>,
}

impl Ledger for Inventory {
    fn adjust(&mut self, sku: Sku, delta: i64) -> Result<u32, String> {
        let item = self
            .items
            .get_mut(&sku)
            .ok_or_else(|| format!("unknown {sku}"))?;
        let next = i64::from(item.quantity) + delta;
        if next < 0 {
            return Err(format!("{sku} would go negative ({next})"));
        }
        item.quantity = next as u32;
        Ok(item.quantity)
    }
}

impl Inventory {
    fn new(items: Vec<Item>) -> Self {
        Inventory {
            items: items.into_iter().map(|i| (i.sku, i)).collect(),
        }
    }

    fn sorted_skus(&self) -> Vec<Sku> {
        let mut skus: Vec<Sku> = self.items.keys().copied().collect();
        skus.sort_unstable();
        skus
    }

    fn by_warehouse(&self) -> BTreeMap<&'static str, Vec<&Item>> {
        let mut groups: BTreeMap<&'static str, Vec<&Item>> = BTreeMap::new();
        for item in self.items.values() {
            groups.entry(item.warehouse).or_default().push(item);
        }
        for list in groups.values_mut() {
            list.sort_by_key(|i| i.sku);
        }
        groups
    }

    fn restock_low(&mut self, batch: u32) -> Vec<(Sku, u32)> {
        let low: Vec<Sku> = self
            .items
            .values()
            .filter(|i| i.quantity <= i.reorder_at)
            .map(|i| i.sku)
            .collect();
        let mut done = Vec::new();
        for sku in low {
            if let Ok(q) = self.adjust(sku, i64::from(batch)) {
                done.push((sku, q));
            }
        }
        done.sort_unstable();
        done
    }
}

fn apply_all<L: Ledger>(ledger: &mut L, moves: &[(Sku, i64)]) -> Vec<String> {
    let mut rejected = Vec::new();
    for &(sku, delta) in moves {
        match ledger.adjust(sku, delta) {
            Ok(q) => println!("{sku} now {q}"),
            Err(e) => rejected.push(e),
        }
    }
    rejected
}

fn label_range(skus: &[Sku], from: Sku, to: Sku) -> String {
    let start = skus.partition_point(|s| *s < from);
    let end = skus.partition_point(|s| *s <= to);
    let mut out = String::new();
    for (i, sku) in skus[start..end].iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write!(out, "{sku}").ok();
    }
    out
}

fn main() {
    let mut inventory = Inventory::new(vec![
        Item { sku: Sku(120), name: "bolt M6".into(), warehouse: "north", quantity: 40, reorder_at: 50 },
        Item { sku: Sku(87), name: "washer 6mm".into(), warehouse: "north", quantity: 900, reorder_at: 200 },
        Item { sku: Sku(301), name: "hinge".into(), warehouse: "south", quantity: 3, reorder_at: 10 },
        Item { sku: Sku(15), name: "bracket".into(), warehouse: "south", quantity: 75, reorder_at: 20 },
        Item { sku: Sku(222), name: "spring".into(), warehouse: "east", quantity: 12, reorder_at: 12 },
    ]);

    let skus = inventory.sorted_skus();
    println!("catalog: {}", label_range(&skus, Sku(0), Sku(999)));
    println!("mid range: {}", label_range(&skus, Sku(80), Sku(230)));
    match skus.binary_search(&Sku(301)) {
        Ok(pos) => println!("{} is at position {pos} of {}", Sku(301), skus.len()),
        Err(_) => println!("{} not stocked", Sku(301)),
    }

    for (warehouse, items) in inventory.by_warehouse() {
        let mut line = String::new();
        for item in &items {
            write!(line, " {}x{}", item.quantity, item.name).ok();
        }
        println!("{warehouse} ({} items):{line}", items.len());
    }

    let restocked = inventory.restock_low(100);
    for (sku, q) in &restocked {
        println!("restocked {sku} to {q}");
    }

    let rejected = apply_all(&mut inventory, &[(Sku(301), -500), (Sku(15), -25), (Sku(999), 4)]);
    println!("{} adjustment(s) rejected", rejected.len());
    for reason in &rejected {
        println!("  {reason}");
    }
}
