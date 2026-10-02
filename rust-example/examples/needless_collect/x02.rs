struct Reading {
    sensor: &'static str,
    celsius: f64,
}

fn overheated(readings: &[Reading], limit: f64) -> usize {
    let hot: Vec<&Reading> = readings.iter().filter(|r| r.celsius > limit).collect();
    hot.len()
}

fn main() {
    let readings = [
        Reading { sensor: "intake", celsius: 41.5 },
        Reading { sensor: "exhaust", celsius: 78.2 },
        Reading { sensor: "core", celsius: 91.0 },
        Reading { sensor: "ambient", celsius: 23.4 },
    ];
    for r in &readings {
        println!("{:>8}: {:.1}", r.sensor, r.celsius);
    }
    println!("{} sensors above limit", overheated(&readings, 75.0));
}
