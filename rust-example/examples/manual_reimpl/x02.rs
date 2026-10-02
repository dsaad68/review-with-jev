const RESERVED: [&str; 6] = ["admin", "root", "support", "billing", "noreply", "security"];

fn is_available(handle: &str) -> bool {
    let lowered = handle.to_lowercase();
    let mut taken = false;
    for word in RESERVED.iter() {
        if *word == lowered {
            taken = true;
            break;
        }
    }
    !taken
}

fn main() {
    for handle in ["Alice", "ROOT", "billing2", "Security"] {
        let verdict = if is_available(handle) { "ok" } else { "reserved" };
        println!("{handle:<10} {verdict}");
    }
}
