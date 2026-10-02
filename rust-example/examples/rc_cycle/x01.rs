use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Folder {
    name: String,
    parent: RefCell<Weak<Folder>>,
    children: RefCell<Vec<Rc<Folder>>>,
}

fn folder(name: &str) -> Rc<Folder> {
    let children = RefCell::new(Vec::new());
    Rc::new(Folder { name: name.to_string(), parent: RefCell::new(Weak::new()), children })
}

fn attach(parent: &Rc<Folder>, child: Rc<Folder>) {
    *child.parent.borrow_mut() = Rc::downgrade(parent);
    parent.children.borrow_mut().push(child);
}

fn full_path(node: &Rc<Folder>) -> String {
    match node.parent.borrow().upgrade() {
        Some(parent) => format!("{}/{}", full_path(&parent), node.name),
        None => format!("/{}", node.name),
    }
}

fn main() {
    let (root, docs, reports) = (folder("home"), folder("docs"), folder("reports"));
    attach(&root, Rc::clone(&docs));
    attach(&docs, Rc::clone(&reports));
    println!("{} (docs strong={} weak={})", full_path(&reports), Rc::strong_count(&docs), Rc::weak_count(&docs));
}
