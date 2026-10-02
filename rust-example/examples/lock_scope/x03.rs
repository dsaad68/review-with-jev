use std::collections::HashMap;
use std::sync::{mpsc, mpsc::Sender, Mutex};
use std::thread;

fn reserve(stock: &Mutex<HashMap<String, u32>>, item: &str, qty: u32, notify: &Sender<(String, u32)>) -> bool {
    let remaining = {
        let mut stock = stock.lock().unwrap();
        match stock.get_mut(item) {
            Some(level) if *level >= qty => {
                *level -= qty;
                *level
            }
            _ => return false,
        }
    };
    notify.send((item.to_string(), remaining)).is_ok()
}

fn main() {
    let stock = Mutex::new(HashMap::from([("bolt".to_string(), 10)]));
    let (tx, rx) = mpsc::channel::<(String, u32)>();
    let watcher = thread::spawn(move || rx.iter().for_each(|(item, left)| println!("{item}: {left} left")));
    for qty in [3, 4, 5] {
        println!("reserved {qty}: {}", reserve(&stock, "bolt", qty, &tx));
    }
    drop(tx);
    watcher.join().unwrap();
}
