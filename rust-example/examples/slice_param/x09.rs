use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Region {
    North,
    South,
    East,
    West,
}

#[derive(Debug)]
struct Order {
    id: u32,
    region: Region,
    cents: u64,
    items: u32,
}

trait Discount: Send + Sync {
    fn apply(&self, order: &Order) -> u64;
    fn label(&self) -> &str;
}

struct Bulk {
    threshold: u32,
    percent: u64,
}

struct Flat {
    cents: u64,
}

impl Discount for Bulk {
    fn apply(&self, order: &Order) -> u64 {
        if order.items >= self.threshold {
            order.cents * self.percent / 100
        } else {
            0
        }
    }
    fn label(&self) -> &str {
        "bulk"
    }
}

impl Discount for Flat {
    fn apply(&self, order: &Order) -> u64 {
        self.cents.min(order.cents)
    }
    fn label(&self) -> &str {
        "flat"
    }
}

struct Money(u64);

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}.{:02}", self.0 / 100, self.0 % 100)
    }
}

fn best_discount<'a>(order: &Order, offers: &'a [Box<dyn Discount>]) -> Option<(&'a str, u64)> {
    offers
        .iter()
        .map(|d| (d.label(), d.apply(order)))
        .filter(|(_, saved)| *saved > 0)
        .max_by_key(|(_, saved)| *saved)
}

fn revenue_by_region(orders: &Vec<Order>) -> HashMap<Region, u64> {
    let mut totals = HashMap::new();
    for order in orders {
        *totals.entry(order.region).or_insert(0) += order.cents;
    }
    totals
}

fn settle(batch: &[Order], offers: &[Box<dyn Discount>], ledger: &Mutex<Vec<(u32, u64)>>) {
    let mut settled = Vec::with_capacity(batch.len());
    for order in batch {
        let saved = best_discount(order, offers).map_or(0, |(_, s)| s);
        settled.push((order.id, order.cents - saved));
    }
    ledger.lock().expect("ledger poisoned").extend(settled);
}

fn main() {
    let orders = Arc::new(vec![
        Order { id: 101, region: Region::North, cents: 12_500, items: 12 },
        Order { id: 102, region: Region::South, cents: 800, items: 1 },
        Order { id: 103, region: Region::East, cents: 45_000, items: 30 },
        Order { id: 104, region: Region::North, cents: 3_200, items: 4 },
        Order { id: 105, region: Region::West, cents: 9_900, items: 10 },
        Order { id: 106, region: Region::East, cents: 150, items: 2 },
    ]);

    let offers: Arc<Vec<Box<dyn Discount>>> = Arc::new(vec![
        Box::new(Bulk { threshold: 10, percent: 15 }),
        Box::new(Flat { cents: 500 }),
    ]);

    let ledger = Arc::new(Mutex::new(Vec::new()));
    let half = orders.len() / 2;
    let mut workers = Vec::new();
    for (start, end) in [(0, half), (half, orders.len())] {
        let orders = Arc::clone(&orders);
        let offers = Arc::clone(&offers);
        let ledger = Arc::clone(&ledger);
        workers.push(thread::spawn(move || {
            settle(&orders[start..end], &offers, &ledger);
        }));
    }
    for worker in workers {
        worker.join().expect("worker panicked");
    }

    let mut settled = ledger.lock().expect("ledger poisoned");
    settled.sort_by_key(|(id, _)| *id);
    for (id, net) in settled.iter() {
        println!("order {id}: {}", Money(*net));
    }

    for order in orders.iter() {
        if let Some((label, saved)) = best_discount(order, &offers) {
            println!("order {} best offer {label} saves {}", order.id, Money(saved));
        }
    }

    let totals = revenue_by_region(&orders);
    for region in [Region::North, Region::South, Region::East, Region::West] {
        let gross = totals.get(&region).copied().unwrap_or(0);
        println!("{region:?}: {}", Money(gross));
    }
}
