use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;

#[derive(Debug)]
struct Item {
    category: String,
    quantity: u32,
    unit_cost_cents: u64,
}

struct Event<'a> {
    kind: &'a str,
    sku: &'a str,
    amount: u32,
}

#[derive(Debug)]
enum InventoryError {
    UnknownSku(String),
    Insufficient { sku: String, wanted: u32, have: u32 },
    UnsupportedEvent(String),
}

trait Ledger {
    fn record(&mut self, event: &Event<'_>) -> Result<(), InventoryError>;
}

struct Warehouse {
    items: HashMap<String, Item>,
    log: Vec<String>,
}

impl Ledger for Warehouse {
    fn record(&mut self, event: &Event<'_>) -> Result<(), InventoryError> {
        let item = self
            .items
            .get_mut(event.sku)
            .ok_or_else(|| InventoryError::UnknownSku(event.sku.to_string()))?;
        match event.kind {
            "restock" => item.quantity += event.amount,
            "sale" | "damage" => {
                if item.quantity < event.amount {
                    return Err(InventoryError::Insufficient {
                        sku: event.sku.to_string(),
                        wanted: event.amount,
                        have: item.quantity,
                    });
                }
                item.quantity -= event.amount;
            }
            "audit" => item.quantity = event.amount,
            other => return Err(InventoryError::UnsupportedEvent(other.to_string())),
        }
        self.log.push(format!("{} {} x{}", event.kind, event.sku, event.amount));
        Ok(())
    }
}

fn storage_rule(category: &str) -> (&'static str, u32) {
    match category {
        "perishable" => ("cold room", 7),
        "fragile" => ("padded shelf", 90),
        "hazardous" => ("cage", 30),
        _ => ("general aisle", 365),
    }
}

fn reorder_threshold(item: &Item) -> u32 {
    if item.category == "perishable" {
        20
    } else if item.category == "fragile" {
        5
    } else {
        10
    }
}

fn apply_all<L: Ledger>(ledger: &mut L, events: &[Event<'_>]) -> Vec<InventoryError> {
    events.iter().filter_map(|e| ledger.record(e).err()).collect()
}

fn valuation_by_category(items: &HashMap<String, Item>) -> HashMap<&str, u64> {
    let totals = Mutex::new(HashMap::new());
    let all: Vec<&Item> = items.values().collect();
    thread::scope(|scope| {
        for chunk in all.chunks(2) {
            let totals = &totals;
            scope.spawn(move || {
                for item in chunk {
                    let value = u64::from(item.quantity) * item.unit_cost_cents;
                    let mut guard = totals.lock().unwrap();
                    *guard.entry(item.category.as_str()).or_insert(0) += value;
                }
            });
        }
    });
    totals.into_inner().unwrap()
}

fn main() {
    let mut warehouse = Warehouse { items: HashMap::new(), log: Vec::new() };
    let stock = [
        ("MLK-1", "perishable", 40, 129),
        ("GLS-7", "fragile", 12, 899),
        ("BLT-3", "standard", 300, 15),
        ("SOL-2", "hazardous", 25, 2_450),
    ];
    for (sku, category, quantity, unit_cost_cents) in stock {
        warehouse.items.insert(
            sku.to_string(),
            Item { category: category.to_string(), quantity, unit_cost_cents },
        );
    }

    let events = [
        Event { kind: "sale", sku: "MLK-1", amount: 25 },
        Event { kind: "damage", sku: "GLS-7", amount: 8 },
        Event { kind: "restock", sku: "BLT-3", amount: 50 },
        Event { kind: "sale", sku: "GLS-7", amount: 10 },
        Event { kind: "retrun", sku: "MLK-1", amount: 2 },
        Event { kind: "audit", sku: "SOL-2", amount: 22 },
        Event { kind: "sale", sku: "XYZ-0", amount: 1 },
    ];

    for err in apply_all(&mut warehouse, &events) {
        println!("rejected: {err:?}");
    }
    println!("applied {} event(s)", warehouse.log.len());

    let mut skus: Vec<&String> = warehouse.items.keys().collect();
    skus.sort();
    for sku in skus {
        let item = &warehouse.items[sku];
        let (place, shelf_days) = storage_rule(&item.category);
        let flag = if item.quantity < reorder_threshold(item) { "REORDER" } else { "" };
        println!("{sku}: {} in {place} (max {shelf_days}d) {flag}", item.quantity);
    }

    let mut totals: Vec<_> = valuation_by_category(&warehouse.items).into_iter().collect();
    totals.sort();
    for (category, cents) in totals {
        println!("{category}: ${}.{:02}", cents / 100, cents % 100);
    }
}
