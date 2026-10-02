use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
    prev: Link<T>,
}

struct Deque<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
}

impl<T> Deque<T> {
    fn new() -> Self {
        Deque { head: None, tail: None, len: 0 }
    }

    fn push_back(&mut self, value: T) {
        let node = Rc::new(RefCell::new(Node { value, next: None, prev: None }));
        match self.tail.take() {
            Some(old) => {
                node.borrow_mut().prev = Some(Rc::clone(&old));
                old.borrow_mut().next = Some(Rc::clone(&node));
            }
            None => self.head = Some(Rc::clone(&node)),
        }
        self.tail = Some(node);
        self.len += 1;
    }

    fn push_front(&mut self, value: T) {
        let node = Rc::new(RefCell::new(Node { value, next: None, prev: None }));
        match self.head.take() {
            Some(old) => {
                node.borrow_mut().next = Some(Rc::clone(&old));
                old.borrow_mut().prev = Some(Rc::clone(&node));
            }
            None => self.tail = Some(Rc::clone(&node)),
        }
        self.head = Some(node);
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        let old = self.head.take()?;
        match old.borrow_mut().next.take() {
            Some(next) => {
                next.borrow_mut().prev = None;
                self.head = Some(next);
            }
            None => self.tail = None,
        }
        self.len -= 1;
        Rc::into_inner(old).map(|cell| cell.into_inner().value)
    }

    fn pop_back(&mut self) -> Option<T> {
        let old = self.tail.take()?;
        match old.borrow_mut().prev.take() {
            Some(prev) => {
                prev.borrow_mut().next = None;
                self.tail = Some(prev);
            }
            None => self.head = None,
        }
        self.len -= 1;
        Rc::into_inner(old).map(|cell| cell.into_inner().value)
    }

    fn iter(&self) -> Iter<T> {
        Iter { cursor: self.head.clone() }
    }
}

struct Iter<T> {
    cursor: Link<T>,
}

impl<T: Clone> Iterator for Iter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let node = self.cursor.take()?;
        let inner = node.borrow();
        self.cursor = inner.next.clone();
        Some(inner.value.clone())
    }
}

#[derive(Clone, PartialEq)]
struct Page {
    url: String,
    dwell_ms: u64,
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} ms)", self.url, self.dwell_ms)
    }
}

struct History {
    pages: Deque<Page>,
    capacity: usize,
}

impl History {
    fn visit(&mut self, url: &str, dwell_ms: u64) -> Option<Page> {
        self.pages.push_back(Page { url: url.to_string(), dwell_ms });
        if self.pages.len > self.capacity {
            self.pages.pop_front()
        } else {
            None
        }
    }

    fn undo(&mut self) -> Option<Page> {
        self.pages.pop_back()
    }

    fn total_dwell(&self) -> u64 {
        self.pages.iter().map(|p| p.dwell_ms).sum()
    }
}

fn describe<T: fmt::Display + Clone>(items: &Deque<T>) -> String {
    items.iter().map(|item| item.to_string()).collect::<Vec<_>>().join(" | ")
}

fn main() {
    let mut history = History { pages: Deque::new(), capacity: 3 };
    let visits = [("/home", 1200), ("/docs", 5400), ("/docs/rc", 9100), ("/blog", 800), ("/about", 300)];
    for (url, dwell) in visits {
        if let Some(evicted) = history.visit(url, dwell) {
            println!("evicted {evicted}");
        }
    }
    println!("history: {}", describe(&history.pages));
    println!("dwell total: {} ms", history.total_dwell());
    if let Some(page) = history.undo() {
        println!("undid {page}");
    }
    history.pages.push_front(Page { url: "/start".to_string(), dwell_ms: 50 });
    println!("now: {} ({} pages)", describe(&history.pages), history.pages.len);
}
