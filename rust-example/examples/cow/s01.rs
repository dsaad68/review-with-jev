fn clean_value(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].replace("\\\"", "\"")
    } else {
        trimmed.to_string()
    }
}

fn main() {
    let lines = ["host = localhost ", "name = \"my \\\"app\\\"\"", "port = 8080"];
    for line in lines {
        if let Some((key, value)) = line.split_once('=') {
            println!("{} => {}", key.trim(), clean_value(value));
        }
    }
}
