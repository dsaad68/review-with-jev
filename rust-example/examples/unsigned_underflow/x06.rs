use std::borrow::Cow;

fn center(out: &mut String, text: &str, width: usize) {
    let len = text.chars().count();
    if len >= width {
        out.extend(text.chars().take(width));
        return;
    }
    let gap = width - len;
    let left = gap / 2;
    out.extend(std::iter::repeat(' ').take(left));
    out.push_str(text);
    out.extend(std::iter::repeat(' ').take(gap - left));
}

fn ellipsize(text: &str, width: usize) -> Cow<'_, str> {
    let len = text.chars().count();
    if len <= width {
        return Cow::Borrowed(text);
    }
    let keep = width.saturating_sub(1);
    let mut out: String = text.chars().take(keep).collect();
    out.push('…');
    Cow::Owned(out)
}

fn column_gap(a: usize, b: usize) -> usize {
    a.abs_diff(b)
}

fn main() {
    let width = 24;
    let border = "=".repeat(width);
    println!("{border}");
    let mut line = String::new();
    for heading in ["Quarterly Report", "An extremely long heading that overflows"] {
        line.clear();
        center(&mut line, heading, width);
        println!("{line}");
    }
    println!("{border}");
    let rows = [
        ("North", "steady growth in all segments"),
        ("South", "flat"),
        ("East", "recovering after a weak second quarter"),
    ];
    for (region, note) in rows {
        println!("{:<6}|{}", region, ellipsize(note, width - 7));
    }
    println!("{}", ellipsize("x", 0));
    let widths: Vec<usize> = rows.iter().map(|(_, n)| n.chars().count()).collect();
    for pair in widths.windows(2) {
        println!("gap between neighbours: {}", column_gap(pair[0], pair[1]));
    }
}
