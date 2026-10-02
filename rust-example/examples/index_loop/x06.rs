#[derive(Debug)]
struct Item {
    sku: String,
    on_hand: u32,
    reorder_point: u32,
    batch_size: u32,
}

struct Warehouse {
    items: Vec<Item>,
}

impl Warehouse {
    fn new() -> Self {
        Warehouse { items: Vec::new() }
    }

    fn add(&mut self, sku: &str, on_hand: u32, reorder_point: u32, batch_size: u32) {
        self.items.push(Item {
            sku: sku.to_string(),
            on_hand,
            reorder_point,
            batch_size,
        });
    }

    fn restock(&mut self) -> u32 {
        let mut ordered = 0;
        for i in 0..self.items.len() {
            if self.items[i].on_hand < self.items[i].reorder_point {
                self.items[i].on_hand += self.items[i].batch_size;
                ordered += 1;
            }
        }
        ordered
    }

    fn report(&self) {
        for item in &self.items {
            println!(
                "{:<8} on hand {:>4} (reorder at {})",
                item.sku, item.on_hand, item.reorder_point
            );
        }
    }
}

fn main() {
    let mut wh = Warehouse::new();
    wh.add("BOLT-M6", 40, 100, 500);
    wh.add("NUT-M6", 820, 100, 500);
    wh.add("WASHER", 12, 50, 200);
    wh.add("HINGE", 75, 20, 40);
    let ordered = wh.restock();
    println!("placed {ordered} orders");
    wh.report();
}
