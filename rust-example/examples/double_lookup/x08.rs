use std::collections::HashMap;
use std::fmt;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Topic {
    Deploy,
    Alert,
    Audit,
}

#[derive(Debug, Clone)]
struct Event {
    topic: Topic,
    source: String,
    payload: u64,
}

trait Handler: Send {
    fn name(&self) -> &str;
    fn handle(&mut self, event: &Event) -> Option<String>;
}

struct Counter {
    label: String,
    seen: u64,
}

impl Handler for Counter {
    fn name(&self) -> &str {
        &self.label
    }

    fn handle(&mut self, event: &Event) -> Option<String> {
        self.seen += 1;
        Some(format!("{} #{} from {}", self.label, self.seen, event.source))
    }
}

struct Threshold {
    limit: u64,
}

impl Handler for Threshold {
    fn name(&self) -> &str {
        "threshold"
    }

    fn handle(&mut self, event: &Event) -> Option<String> {
        if event.payload > self.limit {
            Some(format!("payload {} over {}", event.payload, self.limit))
        } else {
            None
        }
    }
}

struct Bus {
    handlers: HashMap<Topic, Vec<Box<dyn Handler>>>,
}

impl fmt::Debug for Bus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut m = f.debug_map();
        for (topic, hs) in &self.handlers {
            let names: Vec<&str> = hs.iter().map(|h| h.name()).collect();
            m.entry(topic, &names);
        }
        m.finish()
    }
}

impl Bus {
    fn new() -> Self {
        Bus { handlers: HashMap::new() }
    }

    fn subscribe(&mut self, topic: Topic, handler: Box<dyn Handler>) {
        if !self.handlers.contains_key(&topic) {
            self.handlers.insert(topic, Vec::new());
        }
        self.handlers.get_mut(&topic).unwrap().push(handler);
    }

    fn dispatch(&mut self, event: &Event) -> Vec<String> {
        match self.handlers.get_mut(&event.topic) {
            Some(hs) => hs.iter_mut().filter_map(|h| h.handle(event)).collect(),
            None => Vec::new(),
        }
    }
}

fn produce(tx: mpsc::Sender<Event>, source: &'static str, topics: Vec<(Topic, u64)>) {
    for (topic, payload) in topics {
        let event = Event { topic, source: source.to_string(), payload };
        if tx.send(event).is_err() {
            break;
        }
    }
}

fn main() {
    let mut bus = Bus::new();
    bus.subscribe(Topic::Deploy, Box::new(Counter { label: "deploys".into(), seen: 0 }));
    bus.subscribe(Topic::Alert, Box::new(Counter { label: "alerts".into(), seen: 0 }));
    bus.subscribe(Topic::Alert, Box::new(Threshold { limit: 80 }));
    println!("{bus:?}");

    let (tx, rx) = mpsc::channel();
    let producers: Vec<_> = [
        ("ci", vec![(Topic::Deploy, 1), (Topic::Audit, 3), (Topic::Deploy, 2)]),
        ("monitor", vec![(Topic::Alert, 42), (Topic::Alert, 97), (Topic::Alert, 81)]),
    ]
    .into_iter()
    .map(|(source, topics)| {
        let tx = tx.clone();
        thread::spawn(move || produce(tx, source, topics))
    })
    .collect();
    drop(tx);

    let mut log = Vec::new();
    for event in rx {
        log.extend(bus.dispatch(&event));
    }
    for p in producers {
        p.join().expect("producer thread panicked");
    }

    log.sort();
    for line in &log {
        println!("{line}");
    }
}
