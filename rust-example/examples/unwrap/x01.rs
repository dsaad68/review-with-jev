#[derive(Debug)]
enum Scale {
    Celsius,
    Fahrenheit,
}

fn parse_reading(raw: &str) -> Option<(f64, Scale)> {
    let trimmed = raw.trim();
    let (number, scale) = if let Some(n) = trimmed.strip_suffix('C') {
        (n, Scale::Celsius)
    } else if let Some(n) = trimmed.strip_suffix('F') {
        (n, Scale::Fahrenheit)
    } else {
        return None;
    };
    let value = number.trim().parse::<f64>().ok()?;
    Some((value, scale))
}

fn to_kelvin(value: f64, scale: &Scale) -> f64 {
    match scale {
        Scale::Celsius => value + 273.15,
        Scale::Fahrenheit => (value - 32.0) * 5.0 / 9.0 + 273.15,
    }
}

fn main() {
    let (value, scale) = parse_reading("21.5C").unwrap();
    println!("{:?} {} -> {:.2}K", scale, value, to_kelvin(value, &scale));
}
