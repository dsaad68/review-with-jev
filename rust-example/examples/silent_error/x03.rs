struct Tally {
    total: i64,
    accepted: usize,
    rejected: Vec<String>,
}

fn tally(readings: &[&str]) -> Tally {
    let mut tally = Tally { total: 0, accepted: 0, rejected: Vec::new() };
    for raw in readings {
        match raw.trim().parse::<i64>() {
            Ok(value) => {
                tally.total += value;
                tally.accepted += 1;
            }
            Err(e) => tally.rejected.push(format!("{raw:?}: {e}")),
        }
    }
    tally
}

fn main() {
    let readings = ["12", "-3", "seven", "40", "", "5"];
    let result = tally(&readings);
    println!("sum of {} readings: {}", result.accepted, result.total);
    for problem in &result.rejected {
        eprintln!("skipped reading {problem}");
    }
    println!("{} readings rejected", result.rejected.len());
}
