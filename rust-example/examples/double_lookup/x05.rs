use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Reservation {
    Held { sku: u32, qty: u32 },
    Short { sku: u32, wanted: u32, available: u32 },
    Unknown { sku: u32 },
}

struct Warehouse {
    stock: HashMap<u32, u32>,
    held: Vec<(u32, u32)>,
}

impl Warehouse {
    fn new(initial: &[(u32, u32)]) -> Self {
        Warehouse { stock: initial.iter().copied().collect(), held: Vec::new() }
    }

    fn reserve(&mut self, sku: u32, qty: u32) -> Reservation {
        match self.stock.get(&sku) {
            Some(&available) if available >= qty => {
                self.stock.insert(sku, available - qty);
                self.held.push((sku, qty));
                Reservation::Held { sku, qty }
            }
            Some(&available) => Reservation::Short { sku, wanted: qty, available },
            None => Reservation::Unknown { sku },
        }
    }

    fn release_all(&mut self) {
        for (sku, qty) in self.held.drain(..) {
            *self.stock.entry(sku).or_insert(0) += qty;
        }
    }
}

fn main() {
    let mut wh = Warehouse::new(&[(100, 5), (200, 1), (300, 12)]);
    let orders = [(100, 3), (200, 2), (300, 12), (100, 3), (999, 1)];

    for (sku, qty) in orders {
        let outcome = wh.reserve(sku, qty);
        match outcome {
            Reservation::Held { sku, qty } => println!("held {qty} of {sku}"),
            Reservation::Short { sku, wanted, available } => {
                println!("only {available} of {sku}, wanted {wanted}")
            }
            Reservation::Unknown { sku } => println!("unknown sku {sku}"),
        }
    }

    println!("after reservations: {:?}", sorted(&wh.stock));
    wh.release_all();
    println!("after release: {:?}", sorted(&wh.stock));
}

fn sorted(stock: &HashMap<u32, u32>) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = stock.iter().map(|(k, v)| (*k, *v)).collect();
    v.sort_unstable();
    v
}
