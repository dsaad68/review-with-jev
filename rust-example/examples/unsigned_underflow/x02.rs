fn moving_average(values: &[u32], window: usize) -> Vec<f64> {
    if window == 0 || values.len() < window {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(values.len() - window + 1);
    let divisor = f64::from(u32::try_from(window).unwrap_or(u32::MAX));
    for w in values.windows(window) {
        let total: f64 = w.iter().map(|&v| f64::from(v)).sum();
        out.push(total / divisor);
    }
    out
}

fn main() {
    let daily_visitors = [120, 135, 128, 160, 210, 190, 175, 140];
    for window in [3, 7, 10] {
        let avg = moving_average(&daily_visitors, window);
        let shown: Vec<String> = avg.iter().map(|a| format!("{a:.1}")).collect();
        println!("window {window}: [{}]", shown.join(", "));
    }
}
