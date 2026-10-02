struct Thermometer {
    label: &'static str,
    calibration: f64,
}

fn calibrate(readings: &mut [f64], probe: &Thermometer) {
    for i in 0..readings.len() {
        readings[i] = readings[i] + probe.calibration;
    }
}

fn main() {
    let probe = Thermometer {
        label: "greenhouse-north",
        calibration: -0.4,
    };
    let mut readings = vec![21.4, 22.0, 19.8, 23.1, 20.6];
    calibrate(&mut readings, &probe);
    println!("probe {}", probe.label);
    for r in &readings {
        println!("{r:.1} C");
    }
}
