use std::collections::HashMap;

fn load_settings(text: &str) -> HashMap<&str, &str> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect()
}

fn listen_port(settings: &HashMap<&str, &str>) -> u16 {
    let raw = settings.get("port").unwrap();
    raw.parse::<u16>().unwrap()
}

fn worker_count(settings: &HashMap<&str, &str>) -> usize {
    settings
        .get("workers")
        .and_then(|w| w.parse().ok())
        .unwrap_or(4)
}

fn main() {
    let text = "host = 0.0.0.0\nport = 8080\nworkers = 12\n";
    let settings = load_settings(text);
    println!("port {}", listen_port(&settings));
    println!("workers {}", worker_count(&settings));
}
