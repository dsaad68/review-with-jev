use std::collections::HashSet;
use std::rc::Rc;

struct Target {
    name: String,
    cost_ms: u32,
    deps: Vec<Rc<Target>>,
}

fn target(name: &str, cost_ms: u32, deps: &[&Rc<Target>]) -> Rc<Target> {
    Rc::new(Target { name: name.to_string(), cost_ms, deps: deps.iter().map(|d| Rc::clone(d)).collect() })
}

fn build_order(root: &Rc<Target>) -> Vec<Rc<Target>> {
    fn visit(node: &Rc<Target>, done: &mut HashSet<*const Target>, order: &mut Vec<Rc<Target>>) {
        if !done.insert(Rc::as_ptr(node)) {
            return;
        }
        for dep in &node.deps {
            visit(dep, done, order);
        }
        order.push(Rc::clone(node));
    }
    let mut done = HashSet::new();
    let mut order = Vec::new();
    visit(root, &mut done, &mut order);
    order
}

fn critical_path(node: &Target) -> u32 {
    node.cost_ms + node.deps.iter().map(|d| critical_path(d)).max().unwrap_or(0)
}

fn main() {
    let libc = target("libc", 40, &[]);
    let zlib = target("zlib", 25, &[&libc]);
    let ssl = target("openssl", 120, &[&libc]);
    let curl = target("curl", 80, &[&zlib, &ssl]);
    let git = target("git", 150, &[&curl, &zlib, &libc]);

    let order = build_order(&git);
    let names: Vec<&str> = order.iter().map(|t| t.name.as_str()).collect();
    println!("build order: {}", names.join(" -> "));
    println!("serial time: {} ms", order.iter().map(|t| t.cost_ms).sum::<u32>());
    println!("critical path: {} ms", critical_path(&git));
    println!("libc shared by {} owners", Rc::strong_count(&libc) - 1);
}
