fn is_valid_handle(handle: &String) -> bool {
    let len = handle.chars().count();
    (3..=16).contains(&len)
        && handle.starts_with(|c: char| c.is_ascii_alphabetic())
        && handle.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn main() {
    let candidates = vec![
        String::from("ferris_99"),
        String::from("9lives"),
        String::from("ab"),
        String::from("crab_rave"),
        String::from("no-dashes"),
    ];

    for candidate in &candidates {
        let verdict = if is_valid_handle(candidate) { "ok" } else { "rejected" };
        println!("{candidate}: {verdict}");
    }

    let accepted = candidates.iter().filter(|c| is_valid_handle(c)).count();
    println!("{accepted} of {} accepted", candidates.len());
}
