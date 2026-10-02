use std::fs;
use std::path::Path;

fn load_scores(path: &Path) -> Vec<u32> {
    let text = fs::read_to_string(path).expect("scores file");
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().parse::<u32>().expect("score"))
        .collect()
}

fn average(scores: &[u32]) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    scores.iter().map(|&s| s as f64).sum::<f64>() / scores.len() as f64
}

fn main() {
    let path = std::env::temp_dir().join("x03_scores.txt");
    fs::write(&path, "88\n92\n\n75\n100\n").unwrap();
    let scores = load_scores(&path);
    println!("{} scores, mean {:.1}", scores.len(), average(&scores));
}
