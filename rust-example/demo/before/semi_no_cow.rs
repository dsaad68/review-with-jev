struct Record {
    label: String,
    tags: Vec<String>,
    score: f64,
}

fn parse(input: String) -> Vec<Record> {
    input
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let mut parts = line.split(',');
            let label = format!("{:03}-{}", i, parts.next()?.trim());
            let score = parts.next()?.trim().parse::<f64>().ok()? * 1.5;
            let tags = parts.map(|t| format!("#{}", t.trim())).collect();
            Some(Record { label, tags, score })
        })
        .collect()
}

fn main() {
    let data = (1..=5)
        .map(|i| format!("item{i},{},red,blue", i * 3))
        .collect::<Vec<_>>()
        .join("\n");
    let records = parse(data);
    for r in &records {
        println!("{:<12} {:>6.1} {}", r.label, r.score, r.tags.join(" "));
    }
}
