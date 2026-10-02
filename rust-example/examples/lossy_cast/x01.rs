fn luminance(r: u8, g: u8, b: u8) -> u8 {
    let weighted = r as u32 * 299 + g as u32 * 587 + b as u32 * 114;
    u8::try_from(weighted / 1000).unwrap_or(u8::MAX)
}

fn invert(pixel: (u8, u8, u8)) -> (u8, u8, u8) {
    (u8::MAX - pixel.0, u8::MAX - pixel.1, u8::MAX - pixel.2)
}

fn main() {
    let pixels = [(255u8, 0u8, 0u8), (12, 200, 99), (255, 255, 255), (0, 0, 0)];
    for (r, g, b) in pixels {
        let y = luminance(r, g, b);
        let (ir, ig, ib) = invert((r, g, b));
        println!("rgb({r}, {g}, {b}) -> luma {y}, inverted rgb({ir}, {ig}, {ib})");
    }
    let total: u64 = pixels
        .iter()
        .map(|&(r, g, b)| luminance(r, g, b) as u64)
        .sum();
    println!("total luminance {total}");
}
