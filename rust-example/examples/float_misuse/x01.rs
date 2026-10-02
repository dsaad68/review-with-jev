const TOLERANCE_C: f64 = 0.05;

fn mean(samples: &[f64]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    Some(samples.iter().sum::<f64>() / samples.len() as f64)
}

fn agrees(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE_C
}

fn main() {
    let probe_a = [21.40, 21.38, 21.45, 21.41];
    let probe_b = [21.39, 21.43, 21.44, 21.40];
    let probe_c = [22.10, 22.05, 22.20, 22.15];

    let (Some(a), Some(b), Some(c)) = (mean(&probe_a), mean(&probe_b), mean(&probe_c)) else {
        return;
    };
    println!("a={a:.3} b={b:.3} c={c:.3}");
    println!("a~b: {}", agrees(a, b));
    println!("a~c: {}", agrees(a, c));
    if c > a + 0.5 {
        println!("probe c reads warm");
    }
}
