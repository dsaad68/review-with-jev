use std::collections::HashMap;

fn display_name(names: &HashMap<u32, String>, user_id: u32) -> String {
    match names.get(&user_id) {
        Some(name) => name.clone(),
        None => format!("user-{}", user_id),
    }
}

fn main() {
    let mut names = HashMap::new();
    names.insert(1, "Alice".to_string());
    names.insert(2, "Bob".to_string());
    for id in [1, 2, 3] {
        println!("{} posted a comment", display_name(&names, id));
    }
}
