#[derive(Debug, Clone, Copy, PartialEq)]
struct Size {
    width: u32,
    height: u32,
}

fn fit(src: Size, max_w: u32, max_h: u32, keep_aspect: bool, enlarge: bool) -> Option<Size> {
    if src.width == 0 || src.height == 0 || max_w == 0 || max_h == 0 {
        return None;
    }
    if !enlarge && src.width <= max_w && src.height <= max_h {
        return Some(src);
    }
    if !keep_aspect {
        return Some(Size { width: max_w, height: max_h });
    }
    let (w, h) = (u64::from(src.width), u64::from(src.height));
    let (bw, bh) = (u64::from(max_w), u64::from(max_h));
    let scaled = if w * bh <= h * bw {
        let width = u32::try_from(w * bh / h).unwrap_or(max_w);
        Size { width: width.max(1), height: max_h }
    } else {
        let height = u32::try_from(h * bw / w).unwrap_or(max_h);
        Size { width: max_w, height: height.max(1) }
    };
    Some(scaled)
}

fn main() {
    let photos = [
        ("beach.jpg", Size { width: 4032, height: 3024 }),
        ("banner.png", Size { width: 1500, height: 200 }),
        ("icon.png", Size { width: 48, height: 48 }),
        ("broken.gif", Size { width: 0, height: 120 }),
    ];

    for (name, size) in photos {
        let thumb = fit(size, 200, 200, true, false);
        let tile = fit(size, 64, 64, false, true);
        let preview = fit(size, 640, 480, true, true);
        match (thumb, tile, preview) {
            (Some(t), Some(c), Some(p)) => println!(
                "{name}: thumb {}x{}, tile {}x{}, preview {}x{}",
                t.width, t.height, c.width, c.height, p.width, p.height
            ),
            _ => println!("{name}: cannot scale {}x{}", size.width, size.height),
        }
    }
}
