fn summarize(lines: Vec<String>) -> (usize, usize, usize) {
    let mut errors = 0;
    let mut warnings = 0;
    for line in &lines {
        if line.starts_with("ERROR") {
            errors += 1;
        } else if line.starts_with("WARN") {
            warnings += 1;
        }
    }
    (errors, warnings, lines.len())
}

fn main() {
    let lines = vec![
        String::from("INFO service started"),
        String::from("WARN disk at 81%"),
        String::from("ERROR connection refused"),
        String::from("INFO request served"),
        String::from("ERROR timeout after 30s"),
    ];
    let (errors, warnings, total) = summarize(lines.clone());
    println!("{total} lines: {errors} errors, {warnings} warnings");
}
