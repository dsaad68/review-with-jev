struct Summary {
    lowest: i32,
    highest: i32,
    total: i64,
}

fn summarize(elevations: &[i32]) -> Option<Summary> {
    let (&first, rest) = elevations.split_first()?;
    let mut s = Summary {
        lowest: first,
        highest: first,
        total: i64::from(first),
    };
    for &e in rest {
        s.lowest = s.lowest.min(e);
        s.highest = s.highest.max(e);
        s.total += i64::from(e);
    }
    Some(s)
}

fn main() {
    let trail = [312, 498, 655, 640, 802, 1120, 980, 1043];
    if let Some(s) = summarize(&trail) {
        println!("range {}..{} m, total climb basis {}", s.lowest, s.highest, s.total);
    }
}
