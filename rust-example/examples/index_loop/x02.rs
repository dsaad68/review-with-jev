fn running_totals(deposits: &mut [u64]) {
    for i in 1..deposits.len() {
        deposits[i] += deposits[i - 1];
    }
}

fn first_day_reaching(totals: &[u64], goal: u64) -> Option<usize> {
    totals.iter().position(|&t| t >= goal)
}

fn main() {
    let mut deposits: Vec<u64> = vec![120, 45, 300, 80, 15, 260];
    running_totals(&mut deposits);
    println!("cumulative: {:?}", deposits);
    match first_day_reaching(&deposits, 500) {
        Some(day) => println!("goal reached on day {}", day + 1),
        None => println!("goal not reached"),
    }
}
