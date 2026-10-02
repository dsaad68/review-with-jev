use std::collections::BTreeMap;

#[derive(Debug)]
struct Shipment {
    origin: String,
    weight_kg: f32,
    parcels: u16,
}

fn parse_manifest_line(line: &str) -> Shipment {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    Shipment {
        origin: fields[0].to_uppercase(),
        weight_kg: fields[1].parse().unwrap(),
        parcels: fields[2].parse().unwrap(),
    }
}

fn read_manifest(text: &str) -> Vec<Shipment> {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(parse_manifest_line)
        .collect()
}

fn totals_by_origin(shipments: &[Shipment]) -> BTreeMap<&str, (f32, u32)> {
    let mut totals: BTreeMap<&str, (f32, u32)> = BTreeMap::new();
    for s in shipments {
        let entry = totals.entry(s.origin.as_str()).or_insert((0.0, 0));
        entry.0 += s.weight_kg;
        entry.1 += u32::from(s.parcels);
    }
    totals
}

fn heaviest(shipments: &[Shipment]) -> Option<&Shipment> {
    shipments
        .iter()
        .max_by(|a, b| a.weight_kg.total_cmp(&b.weight_kg))
}

fn main() {
    let manifest = "\
rtm, 1200.5, 14
hamburg, 830.0, 9
rtm, 410.25, 3
antwerp, 2290.0, 31
hamburg, 75.5, 1
";
    let shipments = read_manifest(manifest);
    for (origin, (weight, parcels)) in totals_by_origin(&shipments) {
        println!("{origin:<10} {weight:>10.2} kg {parcels:>4} parcels");
    }
    if let Some(top) = heaviest(&shipments) {
        println!("heaviest: {:?}", top);
    }
}
