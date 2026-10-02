fn average(readings: &Vec<f64>) -> Option<f64> {
    if readings.is_empty() {
        return None;
    }
    Some(readings.iter().sum::<f64>() / readings.len() as f64)
}

fn above(readings: &[f64], limit: f64) -> usize {
    readings.iter().filter(|r| **r > limit).count()
}

fn main() {
    let morning = vec![18.5, 19.2, 21.0, 22.4];
    let night: Vec<f64> = Vec::new();

    match average(&morning) {
        Some(avg) => println!("morning average {avg:.2}"),
        None => println!("no morning data"),
    }
    match average(&night) {
        Some(avg) => println!("night average {avg:.2}"),
        None => println!("no night data"),
    }
    println!("{} morning readings above 20", above(&morning, 20.0));
}
