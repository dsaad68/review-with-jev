struct Reading {
    sensor: u32,
    celsius: f64,
}

fn to_csv(readings: &[Reading]) -> String {
    let mut out = String::from("sensor,celsius,fahrenheit\n");
    for r in readings {
        let f = r.celsius * 9.0 / 5.0 + 32.0;
        out = format!("{}{},{:.1},{:.1}\n", out, r.sensor, r.celsius, f);
    }
    out
}

fn main() {
    let readings: Vec<Reading> = (0..12)
        .map(|i| Reading {
            sensor: 100 + i,
            celsius: 18.5 + i as f64 * 0.75,
        })
        .collect();
    let csv = to_csv(&readings);
    print!("{}", csv);
    println!("bytes: {}", csv.len());
}
