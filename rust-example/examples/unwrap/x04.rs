use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug)]
enum StockError {
    UnknownSku(String),
    Insufficient { sku: String, have: u32, want: u32 },
    BadQuantity(String),
}

struct Warehouse {
    levels: Mutex<HashMap<String, u32>>,
}

impl Warehouse {
    fn new(initial: &[(&str, u32)]) -> Self {
        let levels = initial.iter().map(|&(k, v)| (k.to_string(), v)).collect();
        Warehouse { levels: Mutex::new(levels) }
    }

    fn take(&self, sku: &str, qty: u32) -> Result<u32, StockError> {
        let mut levels = self.levels.lock().unwrap();
        let have = levels
            .get_mut(sku)
            .ok_or_else(|| StockError::UnknownSku(sku.to_string()))?;
        if *have < qty {
            return Err(StockError::Insufficient { sku: sku.to_string(), have: *have, want: qty });
        }
        *have -= qty;
        Ok(*have)
    }
}

fn parse_order(line: &str) -> Result<(&str, u32), StockError> {
    let (sku, qty) = line
        .split_once(':')
        .ok_or_else(|| StockError::BadQuantity(line.to_string()))?;
    let qty = qty
        .trim()
        .parse::<u32>()
        .map_err(|_| StockError::BadQuantity(line.to_string()))?;
    Ok((sku.trim(), qty))
}

fn main() {
    let warehouse = Arc::new(Warehouse::new(&[("bolt", 40), ("nut", 25), ("washer", 10)]));
    let batches = vec![
        vec!["bolt:10", "nut:5", "washer:x"],
        vec!["bolt:15", "gear:1", "washer:4"],
        vec!["nut:30", "bolt:10"],
    ];

    let handles: Vec<_> = batches
        .into_iter()
        .enumerate()
        .map(|(id, batch)| {
            let wh = Arc::clone(&warehouse);
            thread::spawn(move || {
                for line in batch {
                    match parse_order(line).and_then(|(sku, qty)| wh.take(sku, qty)) {
                        Ok(left) => println!("worker {id}: {line} ok, {left} left"),
                        Err(e) => println!("worker {id}: {line} failed: {e:?}"),
                    }
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let levels = warehouse.levels.lock().unwrap();
    let mut skus: Vec<_> = levels.iter().collect();
    skus.sort();
    println!("{skus:?}");
}
