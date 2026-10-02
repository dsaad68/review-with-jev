#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

fn dist_sq(a: Point, b: Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn has_right_angle(a: Point, b: Point, c: Point) -> bool {
    let mut sides = [dist_sq(a, b), dist_sq(b, c), dist_sq(c, a)];
    sides.sort_by(f64::total_cmp);
    sides[0] + sides[1] == sides[2]
}

fn main() {
    let p = |x, y| Point { x, y };
    let shapes = [
        (p(0.0, 0.0), p(3.0, 0.0), p(0.0, 4.0)),
        (p(0.1, 0.1), p(0.4, 0.1), p(0.1, 0.5)),
        (p(0.0, 0.0), p(2.0, 0.0), p(1.0, 1.5)),
    ];
    for (a, b, c) in shapes {
        println!("{a:?} {b:?} {c:?} -> right angle: {}", has_right_angle(a, b, c));
    }
}
