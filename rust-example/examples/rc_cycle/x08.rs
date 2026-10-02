use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Symbol {
    Acme,
    Globex,
    Initech,
}

#[derive(Debug, Clone, Copy)]
struct Tick {
    symbol: Symbol,
    price_cents: u64,
}

#[derive(Debug)]
struct Order {
    trader: u32,
    symbol: Symbol,
    limit_cents: u64,
}

trait Subscriber: Send {
    fn on_tick(&mut self, tick: Tick) -> Option<Order>;
    fn report(&self, prices: &HashMap<Symbol, u64>) -> String;
}

struct Exchange {
    venue: String,
    subscribers: Mutex<Vec<Arc<Mutex<dyn Subscriber>>>>,
    last: Mutex<HashMap<Symbol, u64>>,
    book: Mutex<Vec<Order>>,
}

struct MomentumTrader {
    id: u32,
    exchange: Arc<Exchange>,
    previous: HashMap<Symbol, u64>,
    holdings: HashMap<Symbol, u32>,
    cash_cents: u64,
}

impl Subscriber for MomentumTrader {
    fn on_tick(&mut self, tick: Tick) -> Option<Order> {
        let before = self.previous.insert(tick.symbol, tick.price_cents)?;
        if tick.price_cents <= before || self.cash_cents < tick.price_cents {
            return None;
        }
        self.cash_cents -= tick.price_cents;
        *self.holdings.entry(tick.symbol).or_insert(0) += 1;
        Some(Order { trader: self.id, symbol: tick.symbol, limit_cents: tick.price_cents })
    }

    fn report(&self, prices: &HashMap<Symbol, u64>) -> String {
        let marked: u64 = self
            .holdings
            .iter()
            .map(|(symbol, qty)| prices.get(symbol).copied().unwrap_or(0) * u64::from(*qty))
            .sum();
        format!(
            "trader {} on {}: cash {} + positions {} cents",
            self.id, self.exchange.venue, self.cash_cents, marked
        )
    }
}

impl Exchange {
    fn open(venue: &str) -> Arc<Exchange> {
        Arc::new(Exchange {
            venue: venue.to_string(),
            subscribers: Mutex::new(Vec::new()),
            last: Mutex::new(HashMap::new()),
            book: Mutex::new(Vec::new()),
        })
    }

    fn admit(self: &Arc<Self>, id: u32, cash_cents: u64) {
        let trader = MomentumTrader {
            id,
            exchange: Arc::clone(self),
            previous: HashMap::new(),
            holdings: HashMap::new(),
            cash_cents,
        };
        self.subscribers.lock().unwrap().push(Arc::new(Mutex::new(trader)));
    }

    fn publish(&self, tick: Tick) -> usize {
        self.last.lock().unwrap().insert(tick.symbol, tick.price_cents);
        let subscribers = self.subscribers.lock().unwrap().clone();
        let orders: Vec<Order> = subscribers
            .iter()
            .filter_map(|sub| sub.lock().unwrap().on_tick(tick))
            .collect();
        let placed = orders.len();
        self.book.lock().unwrap().extend(orders);
        placed
    }

    fn prices(&self) -> HashMap<Symbol, u64> {
        self.last.lock().unwrap().clone()
    }
}

fn price_walk(start: u64, steps: usize, seed: u64) -> Vec<u64> {
    let mut state = seed;
    let mut price = start;
    let mut out = Vec::with_capacity(steps);
    for _ in 0..steps {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let up = (state >> 33) % 2 == 0;
        price = if up { price + 5 } else { price.saturating_sub(4) };
        out.push(price);
    }
    out
}

fn main() {
    let exchange = Exchange::open("NORTH");
    for id in 1..=3 {
        exchange.admit(id, 50_000);
    }

    let feeds = [(Symbol::Acme, 1_000), (Symbol::Globex, 2_500), (Symbol::Initech, 700)];
    let mut handles = Vec::new();
    for (n, (symbol, start)) in feeds.into_iter().enumerate() {
        let exchange = Arc::clone(&exchange);
        handles.push(thread::spawn(move || {
            let walk = price_walk(start, 40, 7 + n as u64);
            walk.into_iter().map(|price_cents| exchange.publish(Tick { symbol, price_cents })).sum::<usize>()
        }));
    }
    let placed: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();

    let prices = exchange.prices();
    let subscribers = exchange.subscribers.lock().unwrap().clone();
    for sub in &subscribers {
        let line = sub.lock().unwrap().report(&prices);
        println!("{line}");
    }
    let (booked, busiest, priciest) = {
        let book = exchange.book.lock().unwrap();
        let busiest = book.iter().map(|o| o.trader).max();
        let priciest = book.iter().max_by_key(|o| o.limit_cents).map(|o| (o.symbol, o.limit_cents));
        (book.len(), busiest, priciest)
    };
    println!("{placed} orders placed, {booked} booked, busiest {busiest:?}, priciest {priciest:?}");
    println!("exchange handles alive: {}", Arc::strong_count(&exchange));
}
