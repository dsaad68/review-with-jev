use std::collections::HashMap;

struct OrderLine {
    sku: String,
    quantity: u32,
}

struct Catalog {
    prices: HashMap<String, u32>,
    stock: HashMap<String, u32>,
}

impl Catalog {
    fn price_order(&self, lines: &[OrderLine]) -> Result<u32, String> {
        let mut total = 0;
        for line in lines {
            let sku = line.sku.clone();
            let price = match self.prices.get(&sku) {
                Some(p) => *p,
                None => return Err(format!("unknown item {sku}")),
            };
            let available = self.stock.get(&sku).copied().unwrap_or(0);
            if available < line.quantity {
                return Err(format!("only {available} of {sku} left"));
            }
            total += price * line.quantity;
        }
        Ok(total)
    }
}

fn main() {
    let mut prices = HashMap::new();
    prices.insert(String::from("MUG-01"), 899);
    prices.insert(String::from("TEE-L"), 1999);
    prices.insert(String::from("CAP-RED"), 1499);
    let mut stock = HashMap::new();
    stock.insert(String::from("MUG-01"), 12);
    stock.insert(String::from("TEE-L"), 3);

    let catalog = Catalog { prices, stock };

    let orders = vec![
        vec![
            OrderLine { sku: String::from("MUG-01"), quantity: 2 },
            OrderLine { sku: String::from("TEE-L"), quantity: 1 },
        ],
        vec![OrderLine { sku: String::from("TEE-L"), quantity: 5 }],
        vec![OrderLine { sku: String::from("HAT-XL"), quantity: 1 }],
        vec![OrderLine { sku: String::from("CAP-RED"), quantity: 1 }],
    ];

    for (i, order) in orders.iter().enumerate() {
        match catalog.price_order(order) {
            Ok(cents) => println!("order {i}: ${}.{:02}", cents / 100, cents % 100),
            Err(e) => println!("order {i} rejected: {e}"),
        }
    }
}
