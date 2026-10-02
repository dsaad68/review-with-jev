use std::collections::HashMap;

fn restock(shelf: &mut HashMap<String, u32>, item: &str, qty: u32) {
    if shelf.contains_key(item) {
        let current = shelf.get_mut(item).unwrap();
        *current += qty;
    } else {
        shelf.insert(item.to_string(), qty);
    }
}

fn main() {
    let mut shelf: HashMap<String, u32> = HashMap::new();
    restock(&mut shelf, "flour", 4);
    restock(&mut shelf, "sugar", 2);
    restock(&mut shelf, "flour", 3);
    restock(&mut shelf, "yeast", 1);
    restock(&mut shelf, "sugar", 5);

    let mut rows: Vec<(&String, &u32)> = shelf.iter().collect();
    rows.sort();
    for (name, qty) in rows {
        println!("{name:>6}: {qty}");
    }
}
