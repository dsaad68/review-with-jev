fn parse_ports(spec: &str) -> Vec<u16> {
    spec.split(',')
        .map(|part| part.trim().parse::<u16>().unwrap_or(0))
        .filter(|&port| port != 0)
        .collect()
}

fn describe(ports: &[u16]) -> String {
    let listed: Vec<String> = ports.iter().map(|p| format!(":{p}")).collect();
    listed.join(" ")
}

fn main() {
    let spec = "8080, 443, eighty, 70000, 22";
    let ports = parse_ports(spec);
    for port in &ports {
        println!("binding listener on port {port}");
    }
    println!("{} listeners: {}", ports.len(), describe(&ports));
}
