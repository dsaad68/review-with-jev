struct Order {
    id: u32,
    subtotal_cents: u64,
    payment_method: u8,
    shipping: u8,
}

fn payment_fee_cents(order: &Order) -> u64 {
    match order.payment_method {
        1 => order.subtotal_cents * 29 / 1000 + 30,
        2 => order.subtotal_cents * 34 / 1000 + 49,
        3 => 0,
        _ => 0,
    }
}

fn shipping_cents(order: &Order) -> u64 {
    if order.shipping == 2 {
        1500
    } else if order.shipping == 1 {
        600
    } else if order.subtotal_cents >= 5000 {
        0
    } else {
        450
    }
}

fn payment_label(code: u8) -> &'static str {
    match code {
        1 => "card",
        2 => "wallet",
        3 => "bank transfer",
        _ => "unknown",
    }
}

fn dollars(cents: u64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

fn main() {
    let orders = [
        Order { id: 1001, subtotal_cents: 4_250, payment_method: 1, shipping: 0 },
        Order { id: 1002, subtotal_cents: 12_999, payment_method: 2, shipping: 2 },
        Order { id: 1003, subtotal_cents: 7_500, payment_method: 3, shipping: 1 },
        Order { id: 1004, subtotal_cents: 980, payment_method: 4, shipping: 0 },
    ];

    for order in &orders {
        let fee = payment_fee_cents(order);
        let ship = shipping_cents(order);
        let total = order.subtotal_cents + fee + ship;
        println!(
            "#{} via {:<13} subtotal {:>8} fee {:>6} ship {:>6} total {:>8}",
            order.id,
            payment_label(order.payment_method),
            dollars(order.subtotal_cents),
            dollars(fee),
            dollars(ship),
            dollars(total)
        );
    }

    let revenue: u64 = orders.iter().map(|o| o.subtotal_cents + shipping_cents(o)).sum();
    println!("revenue before fees: {}", dollars(revenue));
}
