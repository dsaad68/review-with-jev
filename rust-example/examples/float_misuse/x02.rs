struct LineItem {
    name: &'static str,
    unit_price: f64,
    quantity: u32,
}

fn subtotal(items: &[LineItem]) -> f64 {
    items.iter().map(|i| i.unit_price * f64::from(i.quantity)).sum()
}

fn shipping(total: f64) -> f64 {
    if total >= 50.0 { 0.0 } else { 4.99 }
}

fn main() {
    let cart = [
        LineItem { name: "notebook", unit_price: 3.10, quantity: 3 },
        LineItem { name: "pen", unit_price: 1.15, quantity: 6 },
        LineItem { name: "stapler", unit_price: 12.40, quantity: 1 },
    ];
    for item in &cart {
        println!("{:<9} {:>2} x {:>6.2}", item.name, item.quantity, item.unit_price);
    }
    let sub = subtotal(&cart);
    let total = sub + shipping(sub);
    println!("subtotal {sub:.2}, total {total:.2}");
}
