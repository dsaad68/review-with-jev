use std::collections::HashMap;

fn group_scores(roster: &HashMap<u32, &str>, results: &[(u32, u8)]) -> HashMap<String, Vec<u8>> {
    let mut by_student: HashMap<String, Vec<u8>> = HashMap::new();
    for &(id, score) in results {
        if let Some(name) = roster.get(&id) {
            by_student.entry(name.to_string()).or_default().push(score);
        }
    }
    by_student
}

fn main() {
    let roster: HashMap<u32, &str> = [(1, "Ada"), (2, "Brian"), (3, "Chen")].into_iter().collect();
    let results = [(1, 91), (2, 78), (1, 85), (3, 99), (4, 50), (2, 88)];
    let grouped = group_scores(&roster, &results);

    let mut rows: Vec<(&String, &Vec<u8>)> = grouped.iter().collect();
    rows.sort();
    for (name, scores) in rows {
        let best = scores.iter().max().copied().unwrap_or(0);
        println!("{name}: {scores:?} best={best}");
    }
}
