use std::fmt;
use std::ops::{Add, Mul, Sub};

const EPS: f64 = 1e-9;

#[derive(Debug, Clone, Copy)]
struct V2 {
    x: f64,
    y: f64,
}

impl V2 {
    const fn new(x: f64, y: f64) -> Self {
        V2 { x, y }
    }

    fn dot(self, o: V2) -> f64 {
        self.x * o.x + self.y * o.y
    }

    fn len(self) -> f64 {
        self.dot(self).sqrt()
    }

    fn normalized(self) -> Option<V2> {
        let l = self.len();
        (l > EPS).then(|| self * (1.0 / l))
    }
}

impl Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        V2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 {
        V2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f64> for V2 {
    type Output = V2;
    fn mul(self, k: f64) -> V2 {
        V2::new(self.x * k, self.y * k)
    }
}

impl fmt::Display for V2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.3}, {:.3})", self.x, self.y)
    }
}

struct Ray {
    origin: V2,
    dir: V2,
}

trait Shape {
    fn label(&self) -> &str;
    fn hit(&self, ray: &Ray) -> Option<f64>;
    fn area(&self) -> f64;
}

struct Circle {
    name: String,
    center: V2,
    radius: f64,
}

impl Shape for Circle {
    fn label(&self) -> &str {
        &self.name
    }

    fn hit(&self, ray: &Ray) -> Option<f64> {
        let oc = ray.origin - self.center;
        let b = oc.dot(ray.dir);
        let c = oc.dot(oc) - self.radius * self.radius;
        let disc = b * b - c;
        if disc < 0.0 {
            return None;
        }
        let root = disc.sqrt();
        [-b - root, -b + root].into_iter().find(|t| *t > EPS)
    }

    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

struct Segment {
    name: String,
    a: V2,
    b: V2,
}

impl Shape for Segment {
    fn label(&self) -> &str {
        &self.name
    }

    fn hit(&self, ray: &Ray) -> Option<f64> {
        let edge = self.b - self.a;
        let denom = ray.dir.x * edge.y - ray.dir.y * edge.x;
        if denom.abs() < EPS {
            return None;
        }
        let diff = self.a - ray.origin;
        let t = (diff.x * edge.y - diff.y * edge.x) / denom;
        let u = (diff.x * ray.dir.y - diff.y * ray.dir.x) / denom;
        (t > EPS && (-EPS..=1.0 + EPS).contains(&u)).then_some(t)
    }

    fn area(&self) -> f64 {
        0.0
    }
}

fn closest<'s>(scene: &'s [Box<dyn Shape>], ray: &Ray) -> Option<(&'s dyn Shape, f64)> {
    scene
        .iter()
        .filter_map(|shape| shape.hit(ray).map(|t| (shape.as_ref(), t)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

fn main() {
    let scene: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { name: "lamp".into(), center: V2::new(5.0, 0.0), radius: 1.0 }),
        Box::new(Circle { name: "pillar".into(), center: V2::new(9.0, 2.0), radius: 2.5 }),
        Box::new(Segment { name: "wall".into(), a: V2::new(12.0, -10.0), b: V2::new(12.0, 10.0) }),
    ];

    let origin = V2::new(0.0, 0.0);
    let targets = [V2::new(1.0, 0.0), V2::new(1.0, 0.3), V2::new(1.0, -0.6), V2::new(-1.0, 0.0)];
    for target in targets {
        let Some(dir) = target.normalized() else { continue };
        let ray = Ray { origin, dir };
        match closest(&scene, &ray) {
            Some((shape, t)) => {
                let point = ray.origin + ray.dir * t;
                println!("dir {dir} hits {} at {point} (t={t:.3})", shape.label());
            }
            None => println!("dir {dir} escapes"),
        }
    }

    let total_area: f64 = scene.iter().map(|s| s.area()).sum();
    let expected = std::f64::consts::PI * (1.0 + 6.25);
    println!("solid area {total_area:.4}, matches expected: {}", (total_area - expected).abs() < 1e-6);
}
