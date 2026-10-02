use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct Retry {
    attempts: Cell<u32>,
    on_fail: RefCell<Option<Box<dyn Fn()>>>,
}

fn build(max: u32) -> Rc<Retry> {
    let retry = Rc::new(Retry { attempts: Cell::new(0), on_fail: RefCell::new(None) });
    let handle = Rc::clone(&retry);
    *retry.on_fail.borrow_mut() = Some(Box::new(move || {
        let n = handle.attempts.get() + 1;
        handle.attempts.set(n);
        let verdict = if n < max { "retrying" } else { "giving up" };
        println!("{verdict} after attempt {n}");
    }));
    retry
}

fn main() {
    let retry = build(3);
    for _ in 0..3 {
        if let Some(callback) = retry.on_fail.borrow().as_ref() {
            callback();
        }
    }
    println!("attempts={} refs={}", retry.attempts.get(), Rc::strong_count(&retry));
}
