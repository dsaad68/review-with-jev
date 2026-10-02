#[derive(Debug)]
struct Listing {
    address: String,
    bedrooms: u8,
    asking_cents: u64,
    floor_area_m2: u32,
}

impl Listing {
    fn new(address: &str, bedrooms: u8, asking_cents: u64, floor_area_m2: u32) -> Self {
        Listing {
            address: address.to_string(),
            bedrooms,
            asking_cents,
            floor_area_m2,
        }
    }

    fn cents_per_m2(&self) -> u64 {
        self.asking_cents / u64::from(self.floor_area_m2.max(1))
    }
}

fn largest(listings: &[Listing]) -> Option<&Listing> {
    let mut best: Option<&Listing> = None;
    for l in listings {
        match best {
            None => best = Some(l),
            Some(b) if l.floor_area_m2 > b.floor_area_m2 => best = Some(l),
            _ => {}
        }
    }
    best
}

fn cheapest_per_m2(listings: &[Listing]) -> Option<&Listing> {
    listings.iter().min_by_key(|l| l.cents_per_m2())
}

fn main() {
    let listings = vec![
        Listing::new("12 Elm Row", 2, 31_500_000, 68),
        Listing::new("4 Quay St", 3, 52_000_000, 104),
        Listing::new("77 Hill Rd", 4, 61_250_000, 131),
        Listing::new("9 Mill Ln", 1, 18_900_000, 41),
    ];
    if let Some(l) = largest(&listings) {
        println!("largest: {} ({} m2, {} bed)", l.address, l.floor_area_m2, l.bedrooms);
    }
    if let Some(l) = cheapest_per_m2(&listings) {
        println!("best value: {} at {} cents/m2", l.address, l.cents_per_m2());
    }
}
