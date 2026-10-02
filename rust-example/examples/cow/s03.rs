fn sorted_readings(values: &[i32]) -> Vec<i32> {
    if values.windows(2).all(|w| w[0] <= w[1]) {
        return values.to_vec();
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

fn median(values: &[i32]) -> i32 {
    let sorted = sorted_readings(values);
    sorted[sorted.len() / 2]
}

fn main() {
    let morning = vec![12, 14, 15, 18, 21];
    let evening = vec![19, 11, 16, 13, 17];
    println!("morning median: {}", median(&morning));
    println!("evening median: {}", median(&evening));
}
