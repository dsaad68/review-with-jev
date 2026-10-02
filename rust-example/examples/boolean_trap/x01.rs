fn format_name(first: &str, last: &str, surname_first: bool, shout: bool) -> String {
    let joined = if surname_first {
        format!("{last}, {first}")
    } else {
        format!("{first} {last}")
    };
    if shout {
        joined.to_uppercase()
    } else {
        joined
    }
}

fn main() {
    let people = [("Grace", "Hopper"), ("Alan", "Turing"), ("Ada", "Lovelace")];
    let mut directory: Vec<String> = people
        .iter()
        .map(|(first, last)| format_name(first, last, true, false))
        .collect();
    directory.sort();
    for entry in &directory {
        println!("{entry}");
    }
    for (first, last) in people {
        println!("{}", format_name(first, last, false, true));
    }
}
