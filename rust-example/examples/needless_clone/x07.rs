struct Student {
    name: &'static str,
    score: f64,
}

fn median(scores: &[f64]) -> Option<f64> {
    if scores.is_empty() {
        return None;
    }
    let mut sorted = scores.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

fn curve(scores: &[f64], target_top: f64) -> Vec<f64> {
    let top = scores.iter().cloned().fold(f64::MIN, f64::max);
    if top <= 0.0 {
        return vec![0.0; scores.len()];
    }
    scores.iter().map(|s| s * target_top / top).collect()
}

fn main() {
    let roster = [
        Student { name: "Ines", score: 71.0 },
        Student { name: "Tomas", score: 88.5 },
        Student { name: "Yuki", score: 64.0 },
        Student { name: "Ravi", score: 92.0 },
        Student { name: "Olga", score: 79.5 },
        Student { name: "Ben", score: 55.0 },
    ];
    let scores: Vec<f64> = roster.iter().map(|s| s.score).collect();
    let curved = curve(&scores, 100.0);

    match median(&scores) {
        Some(m) => println!("raw median: {m:.1}"),
        None => println!("no scores"),
    }
    if let Some(m) = median(&curved) {
        println!("curved median: {m:.1}");
    }
    for ((student, raw), adj) in roster.iter().zip(&scores).zip(&curved) {
        println!("{:<6} {raw:>5.1} -> {adj:>5.1}", student.name);
    }
}
