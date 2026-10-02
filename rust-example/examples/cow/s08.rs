use std::collections::HashMap;

fn price_for(prices: &HashMap<String, u32>, item: &str) -> Option<u32> {
    let key = item.to_string();
    prices.get(&key).copied()
}

fn main() {
    let mut prices = HashMap::new();
    prices.insert("apple".to_string(), 120);
    prices.insert("bread".to_string(), 250);
    let cart = ["apple", "bread", "milk"];
    let mut total = 0;
    for item in cart {
        match price_for(&prices, item) {
            Some(p) => total += p,
            None => println!("no price for {}", item),
        }
    }
    println!("total: {}", total);
}
