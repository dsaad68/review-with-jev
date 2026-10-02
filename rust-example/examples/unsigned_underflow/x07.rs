use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
enum OrderError {
    UnknownSku(String),
    EmptyOrder,
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderError::UnknownSku(sku) => write!(f, "no product with sku {sku}"),
            OrderError::EmptyOrder => write!(f, "order has no lines"),
        }
    }
}

struct Warehouse {
    stock: HashMap<String, u32>,
}

impl Warehouse {
    fn new(items: &[(&str, u32)]) -> Self {
        Warehouse {
            stock: items.iter().map(|&(s, q)| (s.to_string(), q)).collect(),
        }
    }

    fn fulfil(&mut self, order: &[(&str, u32)]) -> Result<u32, OrderError> {
        if order.is_empty() {
            return Err(OrderError::EmptyOrder);
        }
        let mut shipped = 0;
        for &(sku, qty) in order {
            let level = self
                .stock
                .get_mut(sku)
                .ok_or_else(|| OrderError::UnknownSku(sku.to_string()))?;
            *level = *level - qty;
            shipped += qty;
        }
        Ok(shipped)
    }

    fn low_stock(&self, threshold: u32) -> Vec<(&str, u32)> {
        let mut low: Vec<(&str, u32)> = self
            .stock
            .iter()
            .filter(|(_, &q)| q < threshold)
            .map(|(s, &q)| (s.as_str(), q))
            .collect();
        low.sort_unstable();
        low
    }
}

fn main() {
    let mut wh = Warehouse::new(&[("BOLT-M6", 500), ("NUT-M6", 420), ("WASHER-6", 80)]);
    let orders: [&[(&str, u32)]; 3] = [
        &[("BOLT-M6", 120), ("NUT-M6", 120)],
        &[("WASHER-6", 60)],
        &[("SCREW-4", 10)],
    ];
    for order in orders {
        match wh.fulfil(order) {
            Ok(n) => println!("shipped {n} units"),
            Err(e) => println!("order failed: {e}"),
        }
    }
    println!("low stock: {:?}", wh.low_stock(100));
}
