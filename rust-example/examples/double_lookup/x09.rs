use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy)]
struct Order {
    id: u64,
    side: Side,
    price_ticks: u64,
    qty: u64,
}

#[derive(Debug, PartialEq)]
struct Fill {
    maker: u64,
    taker: u64,
    price_ticks: u64,
    qty: u64,
}

trait Book {
    fn submit(&mut self, order: Order) -> Vec<Fill>;
    fn depth(&self, side: Side, price_ticks: u64) -> u64;
}

#[derive(Default)]
struct LimitBook {
    bids: BTreeMap<u64, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
}

impl LimitBook {
    fn crosses(side: Side, limit: u64, level: u64) -> bool {
        match side {
            Side::Bid => level <= limit,
            Side::Ask => level >= limit,
        }
    }

    fn consume_level(queue: &mut VecDeque<Order>, level: u64, taker: &mut Order, fills: &mut Vec<Fill>) {
        while taker.qty > 0 {
            let Some(maker) = queue.front_mut() else { break };
            let traded = maker.qty.min(taker.qty);
            maker.qty -= traded;
            taker.qty -= traded;
            fills.push(Fill { maker: maker.id, taker: taker.id, price_ticks: level, qty: traded });
            if maker.qty == 0 {
                queue.pop_front();
            }
        }
    }
}

impl Book for LimitBook {
    fn submit(&mut self, mut order: Order) -> Vec<Fill> {
        let mut fills = Vec::new();
        while order.qty > 0 {
            let best = match order.side {
                Side::Bid => self.asks.first_entry(),
                Side::Ask => self.bids.last_entry(),
            };
            let Some(mut slot) = best else { break };
            let level = *slot.key();
            if !Self::crosses(order.side, order.price_ticks, level) {
                break;
            }
            Self::consume_level(slot.get_mut(), level, &mut order, &mut fills);
            if slot.get().is_empty() {
                slot.remove();
            }
        }
        if order.qty > 0 {
            let own = match order.side {
                Side::Bid => &mut self.bids,
                Side::Ask => &mut self.asks,
            };
            own.entry(order.price_ticks).or_default().push_back(order);
        }
        fills
    }

    fn depth(&self, side: Side, price_ticks: u64) -> u64 {
        let levels = match side {
            Side::Bid => &self.bids,
            Side::Ask => &self.asks,
        };
        levels.get(&price_ticks).map_or(0, level_qty)
    }
}

fn level_qty(queue: &VecDeque<Order>) -> u64 {
    queue.iter().map(|o| o.qty).sum()
}

impl fmt::Display for LimitBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (price, queue) in self.asks.iter().rev() {
            writeln!(f, "  ask {price:>5} x {}", level_qty(queue))?;
        }
        for (price, queue) in self.bids.iter().rev() {
            writeln!(f, "  bid {price:>5} x {}", level_qty(queue))?;
        }
        Ok(())
    }
}

fn run<B: Book + fmt::Display>(book: &mut B, orders: &[Order]) {
    for order in orders {
        for fill in book.submit(*order) {
            println!("fill {fill:?}");
        }
    }
    print!("{book}");
}

fn main() {
    let orders = [
        Order { id: 1, side: Side::Ask, price_ticks: 10_050, qty: 5 },
        Order { id: 2, side: Side::Ask, price_ticks: 10_040, qty: 3 },
        Order { id: 3, side: Side::Bid, price_ticks: 10_000, qty: 4 },
        Order { id: 4, side: Side::Bid, price_ticks: 10_045, qty: 6 },
        Order { id: 5, side: Side::Ask, price_ticks: 10_000, qty: 2 },
        Order { id: 6, side: Side::Bid, price_ticks: 10_045, qty: 1 },
    ];
    let mut book = LimitBook::default();
    run(&mut book, &orders);
    println!("bid depth at 10045: {}", book.depth(Side::Bid, 10_045));
    println!("ask depth at 10045: {}", book.depth(Side::Ask, 10_045));
}
