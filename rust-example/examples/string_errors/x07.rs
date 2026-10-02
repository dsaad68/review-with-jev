#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
    Cart,
    Paid,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone, Copy)]
enum Event {
    Pay { amount_cents: u64 },
    Ship,
    Deliver,
    Cancel,
}

struct Order {
    id: u32,
    total_cents: u64,
    stage: Stage,
}

impl Order {
    fn apply(&mut self, event: Event) -> Result<Stage, String> {
        let next = match (self.stage, event) {
            (Stage::Cart, Event::Pay { amount_cents }) if amount_cents == self.total_cents => Stage::Paid,
            (Stage::Cart, Event::Pay { amount_cents }) => {
                return Err(format!(
                    "order {}: paid {amount_cents} but total is {}",
                    self.id, self.total_cents
                ))
            }
            (Stage::Paid, Event::Ship) => Stage::Shipped,
            (Stage::Shipped, Event::Deliver) => Stage::Delivered,
            (Stage::Cart | Stage::Paid, Event::Cancel) => Stage::Cancelled,
            (Stage::Shipped, Event::Cancel) => {
                return Err(format!("order {} already left the warehouse", self.id))
            }
            (Stage::Delivered | Stage::Cancelled, _) => {
                return Err(format!("order {} is closed ({:?})", self.id, self.stage))
            }
            (stage, event) => {
                return Err(format!("order {}: cannot {event:?} while {stage:?}", self.id))
            }
        };
        self.stage = next;
        Ok(next)
    }
}

fn main() {
    let mut orders = [
        Order { id: 101, total_cents: 4_999, stage: Stage::Cart },
        Order { id: 102, total_cents: 1_250, stage: Stage::Cart },
    ];
    let script = [
        (0, Event::Ship),
        (0, Event::Pay { amount_cents: 4_000 }),
        (0, Event::Pay { amount_cents: 4_999 }),
        (0, Event::Ship),
        (0, Event::Cancel),
        (1, Event::Cancel),
        (1, Event::Pay { amount_cents: 1_250 }),
        (0, Event::Deliver),
    ];
    for (slot, event) in script {
        let order = &mut orders[slot];
        match order.apply(event) {
            Ok(stage) => println!("order {} -> {stage:?}", order.id),
            Err(reason) => println!("refused: {reason}"),
        }
    }
}
