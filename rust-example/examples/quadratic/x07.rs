use std::fmt;

struct Customer {
    id: u64,
    name: String,
    discount_pct: u32,
}

struct Order {
    customer_id: u64,
    amount_cents: u64,
}

struct Invoice<'a> {
    customer: &'a str,
    gross: u64,
    net: u64,
}

impl fmt::Display for Invoice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<10} gross {:>4}.{:02} net {:>4}.{:02}",
            self.customer,
            self.gross / 100,
            self.gross % 100,
            self.net / 100,
            self.net % 100
        )
    }
}

fn build_invoices<'a>(customers: &'a [Customer], orders: &[Order]) -> (Vec<Invoice<'a>>, usize) {
    let mut invoices = Vec::with_capacity(orders.len());
    let mut orphaned = 0;
    for order in orders {
        match customers.iter().find(|c| c.id == order.customer_id) {
            Some(c) => {
                let net = order.amount_cents * u64::from(100 - c.discount_pct) / 100;
                invoices.push(Invoice {
                    customer: &c.name,
                    gross: order.amount_cents,
                    net,
                });
            }
            None => orphaned += 1,
        }
    }
    (invoices, orphaned)
}

fn main() {
    let customers: Vec<Customer> = ["ada", "brian", "chen", "dora", "emeka"]
        .iter()
        .enumerate()
        .map(|(i, n)| Customer {
            id: 1000 + i as u64,
            name: n.to_string(),
            discount_pct: (i as u32 * 5) % 20,
        })
        .collect();
    let orders: Vec<Order> = (0..12)
        .map(|k| Order {
            customer_id: 1000 + (k * 3) % 7,
            amount_cents: 1999 + k * 1250,
        })
        .collect();
    let (invoices, orphaned) = build_invoices(&customers, &orders);
    for inv in &invoices {
        println!("{}", inv);
    }
    println!("{} invoices, {} orphaned orders", invoices.len(), orphaned);
}
