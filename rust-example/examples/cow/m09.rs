use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    id: u32,
    customer: String,
    items: Vec<String>,
    total_cents: u64,
}

struct Ledger {
    current: HashMap<u32, Order>,
    history: Vec<Order>,
}

impl Ledger {
    fn new() -> Self {
        Ledger { current: HashMap::new(), history: Vec::new() }
    }

    fn insert(&mut self, order: Order) {
        self.current.insert(order.id, order);
    }

    fn amend(&mut self, id: u32, extra_item: &str, extra_cents: u64) -> Option<Order> {
        let existing = self.current.get(&id)?;
        let mut updated = existing.clone();
        updated.items.push(extra_item.to_string());
        updated.total_cents += extra_cents;
        let previous = self.current.insert(id, updated.clone())?;
        self.history.push(previous);
        Some(updated)
    }
}

fn load(raw: String) -> Vec<Order> {
    raw.lines()
        .filter_map(|line| {
            let mut parts = line.split('|');
            let id = parts.next()?.trim().parse().ok()?;
            let customer = parts.next()?.trim().to_string();
            let items = parts.next()?.split(',').map(|s| s.trim().to_string()).collect();
            let total_cents = parts.next()?.trim().parse().ok()?;
            Some(Order { id, customer, items, total_cents })
        })
        .collect()
}

fn main() {
    let raw = String::from("1 | ana | pen, ink | 450\n2 | ben | paper | 300\n");
    let orders = load(raw);
    let mut ledger = Ledger::new();
    for o in orders {
        ledger.insert(o);
    }
    if let Some(o) = ledger.amend(1, "notebook", 799) {
        println!("amended: {:?}", o);
    }
    println!("history: {:?}", ledger.history);
    let mut ids: Vec<_> = ledger.current.keys().copied().collect();
    ids.sort();
    for id in ids {
        let o = &ledger.current[&id];
        println!("{} {} {:?} {}", o.id, o.customer, o.items, o.total_cents);
    }
}
