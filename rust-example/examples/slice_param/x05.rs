trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &str;
}

struct Circle {
    radius: f64,
}

struct Rect {
    width: f64,
    height: f64,
}

struct Triangle {
    a: f64,
    b: f64,
    c: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    fn name(&self) -> &str {
        "circle"
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
    fn name(&self) -> &str {
        "rectangle"
    }
}

impl Shape for Triangle {
    fn area(&self) -> f64 {
        let s = (self.a + self.b + self.c) / 2.0;
        let product = s * (s - self.a) * (s - self.b) * (s - self.c);
        if product > 0.0 { product.sqrt() } else { 0.0 }
    }
    fn name(&self) -> &str {
        "triangle"
    }
}

fn describe(shape: &Box<dyn Shape>) -> String {
    format!("{} with area {:.2}", shape.name(), shape.area())
}

fn largest(shapes: &[Box<dyn Shape>]) -> Option<&dyn Shape> {
    shapes
        .iter()
        .map(|s| s.as_ref())
        .max_by(|a, b| a.area().total_cmp(&b.area()))
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 1.5 }),
        Box::new(Rect { width: 3.0, height: 2.5 }),
        Box::new(Triangle { a: 3.0, b: 4.0, c: 5.0 }),
        Box::new(Triangle { a: 1.0, b: 1.0, c: 5.0 }),
    ];

    for shape in &shapes {
        println!("{}", describe(shape));
    }

    if let Some(big) = largest(&shapes) {
        println!("largest is the {} ({:.2})", big.name(), big.area());
    }
}
