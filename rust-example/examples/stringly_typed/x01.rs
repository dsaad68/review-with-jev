use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum Roast { Light, Medium, Dark }
impl FromStr for Roast {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "light" => Ok(Roast::Light),
            "medium" => Ok(Roast::Medium),
            "dark" => Ok(Roast::Dark),
            _ => Err(()),
        }
    }
}
fn roast_minutes(roast: Roast) -> u32 {
    match roast {
        Roast::Light => 9,
        Roast::Medium => 11,
        Roast::Dark => 14,
    }
}
fn main() {
    for raw in ["Light", " dark", "medium", "burnt"] {
        match raw.parse::<Roast>() {
            Ok(roast) => println!("{:?}: {} min", roast, roast_minutes(roast)),
            Err(()) => println!("unrecognised roast {:?}", raw),
        }
    }
}
