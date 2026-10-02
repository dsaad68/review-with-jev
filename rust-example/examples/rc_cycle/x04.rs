use std::cell::RefCell;
use std::rc::{Rc, Weak};

type Link = Option<Rc<RefCell<Track>>>;

struct Track {
    title: String,
    seconds: u32,
    next: Link,
    prev: Option<Weak<RefCell<Track>>>,
}

struct Playlist {
    head: Link,
    tail: Link,
    len: usize,
}

impl Playlist {
    fn new() -> Self {
        Playlist { head: None, tail: None, len: 0 }
    }

    fn append(&mut self, title: &str, seconds: u32) {
        let track = Rc::new(RefCell::new(Track { title: title.to_string(), seconds, next: None, prev: None }));
        match self.tail.take() {
            Some(old) => {
                track.borrow_mut().prev = Some(Rc::downgrade(&old));
                old.borrow_mut().next = Some(Rc::clone(&track));
            }
            None => self.head = Some(Rc::clone(&track)),
        }
        self.tail = Some(track);
        self.len += 1;
    }

    fn total_seconds(&self) -> u32 {
        let mut total = 0;
        let mut cursor = self.head.clone();
        while let Some(node) = cursor {
            total += node.borrow().seconds;
            cursor = node.borrow().next.clone();
        }
        total
    }

    fn backwards(&self) -> Vec<String> {
        let mut titles = Vec::with_capacity(self.len);
        let mut cursor = self.tail.clone();
        while let Some(node) = cursor {
            titles.push(node.borrow().title.clone());
            cursor = node.borrow().prev.as_ref().and_then(Weak::upgrade);
        }
        titles
    }
}

fn main() {
    let mut list = Playlist::new();
    for (title, secs) in [("Intro", 95), ("Harbor", 241), ("Lanterns", 187), ("Outro", 60)] {
        list.append(title, secs);
    }
    println!("{} tracks, {} seconds", list.len, list.total_seconds());
    println!("reverse: {}", list.backwards().join(" <- "));
}
