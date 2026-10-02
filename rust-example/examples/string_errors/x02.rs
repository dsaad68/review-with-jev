fn parse_rgb(hex: &str) -> Result<(u8, u8, u8), String> {
    let digits = hex
        .strip_prefix('#')
        .ok_or_else(|| format!("colour {hex:?} must start with '#'"))?;
    if digits.len() != 6 {
        return Err(format!("colour {hex:?} needs 6 hex digits, found {}", digits.len()));
    }
    let channel = |range: std::ops::Range<usize>| {
        let part = digits.get(range).ok_or_else(|| format!("colour {hex:?} is not ASCII"))?;
        u8::from_str_radix(part, 16).map_err(|e| format!("bad channel {part:?} in {hex:?}: {e}"))
    };
    Ok((channel(0..2)?, channel(2..4)?, channel(4..6)?))
}

fn luminance_bucket(r: u8, g: u8, b: u8) -> &'static str {
    let weighted = 299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b);
    if weighted > 128_000 { "light" } else { "dark" }
}

fn main() {
    for input in ["#1e90ff", "#FFF", "ff8800", "#12g456", "#fafafa"] {
        match parse_rgb(input) {
            Ok((r, g, b)) => println!("{input}: ({r}, {g}, {b}) {}", luminance_bucket(r, g, b)),
            Err(e) => println!("error: {e}"),
        }
    }
}
