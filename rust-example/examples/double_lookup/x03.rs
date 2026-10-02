use std::collections::HashMap;

fn leg_minutes(arrivals: &HashMap<&str, u32>, from: &str, to: &str) -> Option<u32> {
    let depart = arrivals.get(from)?;
    let arrive = arrivals.get(to)?;
    arrive.checked_sub(*depart)
}

fn main() {
    let arrivals: HashMap<&str, u32> = [
        ("Harbor", 0),
        ("Market", 7),
        ("Library", 15),
        ("Stadium", 26),
        ("Airport", 48),
    ]
    .into_iter()
    .collect();

    let trips = [("Harbor", "Library"), ("Market", "Airport"), ("Stadium", "Market"), ("Harbor", "Zoo")];
    for (from, to) in trips {
        match leg_minutes(&arrivals, from, to) {
            Some(m) => println!("{from} -> {to}: {m} min"),
            None => println!("{from} -> {to}: no direct leg"),
        }
    }
}
