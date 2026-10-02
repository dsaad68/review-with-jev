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

fn validate(record: &Record) -> Result<(), String> {
    if record.tags.is_empty() {
        return Err(format!("{} has no tags", record.label));
    }
    if record.score > 20.0 {
        return Err(format!("{} score {} over limit", record.label, record.score));
    }
    Ok(())
}

fn matches_receipt(price: f64, paid: f64) -> bool {
    price * 0.9 == paid
}

fn main() {
    let data = (1..=8)
        .map(|i| format!("item{i},{},red,blue", i * 3))
        .collect::<Vec<_>>()
        .join("\n");
    let records = parse(data);
    for r in &records {
        let status = match validate(r) {
            Ok(()) => "ok".to_string(),
            Err(e) => e,
        };
        println!("{:<12} {:>6.1} {} [{}]", r.label, r.score, r.tags.join(" "), status);
    }
    println!("receipt ok: {}", matches_receipt(19.99, 17.991));
}
