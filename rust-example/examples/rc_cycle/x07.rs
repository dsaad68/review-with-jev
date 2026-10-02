use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

struct Button {
    label: String,
    clicks: Cell<u32>,
    enabled: Cell<bool>,
    handlers: RefCell<Vec<Box<dyn Fn()>>>,
}

impl Button {
    fn new(label: &str) -> Rc<Button> {
        Rc::new(Button {
            label: label.to_string(),
            clicks: Cell::new(0),
            enabled: Cell::new(true),
            handlers: RefCell::new(Vec::new()),
        })
    }

    fn on_click(&self, handler: impl Fn() + 'static) {
        self.handlers.borrow_mut().push(Box::new(handler));
    }

    fn click(&self) {
        if !self.enabled.get() {
            println!("{} ignored", self.label);
            return;
        }
        for handler in self.handlers.borrow().iter() {
            handler();
        }
    }
}

fn wire_up(button: &Rc<Button>, limit: u32) {
    let me: Weak<Button> = Rc::downgrade(button);
    button.on_click(move || {
        if let Some(b) = me.upgrade() {
            let n = b.clicks.get() + 1;
            b.clicks.set(n);
            if n >= limit {
                b.enabled.set(false);
            }
        }
    });
    let me = Rc::downgrade(button);
    button.on_click(move || {
        if let Some(b) = me.upgrade() {
            println!("{} clicked ({} so far)", b.label, b.clicks.get());
        }
    });
}

fn main() {
    let submit = Button::new("Submit");
    wire_up(&submit, 2);
    for _ in 0..3 {
        submit.click();
    }
    println!("strong={} weak={}", Rc::strong_count(&submit), Rc::weak_count(&submit));
    let probe = Rc::downgrade(&submit);
    drop(submit);
    println!("released: {}", probe.upgrade().is_none());
}
