#[derive(Clone, Copy, Debug)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2 { x: self.x - o.x, y: self.y - o.y }
    }
    fn scale(self, k: f64) -> Vec2 {
        Vec2 { x: self.x * k, y: self.y * k }
    }
    fn add(self, o: Vec2) -> Vec2 {
        Vec2 { x: self.x + o.x, y: self.y + o.y }
    }
    fn len_sq(self) -> f64 { self.x * self.x + self.y * self.y }
}

struct Body {
    pos: Vec2,
    vel: Vec2,
    mass: f64,
}

fn accelerations(bodies: &[Body], softening: f64) -> Vec<Vec2> {
    let mut acc = vec![Vec2 { x: 0.0, y: 0.0 }; bodies.len()];
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            let d = bodies[j].pos.sub(bodies[i].pos);
            let r2 = d.len_sq() + softening * softening;
            let inv = 1.0 / (r2 * r2.sqrt());
            acc[i] = acc[i].add(d.scale(bodies[j].mass * inv));
            acc[j] = acc[j].sub(d.scale(bodies[i].mass * inv));
        }
    }
    acc
}

fn step(bodies: &mut [Body], dt: f64) {
    let acc = accelerations(bodies, 0.01);
    for (b, a) in bodies.iter_mut().zip(acc) {
        b.vel = b.vel.add(a.scale(dt));
        b.pos = b.pos.add(b.vel.scale(dt));
    }
}

fn energy(bodies: &[Body]) -> f64 {
    let kinetic: f64 = bodies.iter().map(|b| 0.5 * b.mass * b.vel.len_sq()).sum();
    let mut potential = 0.0;
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            let r = bodies[j].pos.sub(bodies[i].pos).len_sq().sqrt();
            potential -= bodies[i].mass * bodies[j].mass / r;
        }
    }
    kinetic + potential
}

fn main() {
    let mut bodies: Vec<Body> = (0..8)
        .map(|i| {
            let t = i as f64 * std::f64::consts::TAU / 8.0;
            Body {
                pos: Vec2 { x: t.cos() * 5.0, y: t.sin() * 5.0 },
                vel: Vec2 { x: -t.sin() * 0.3, y: t.cos() * 0.3 },
                mass: 1.0 + (i % 3) as f64,
            }
        })
        .collect();
    let start = energy(&bodies);
    for _ in 0..200 {
        step(&mut bodies, 0.01);
    }
    println!("energy {:.5} -> {:.5}", start, energy(&bodies));
    for b in &bodies {
        println!("({:7.3}, {:7.3})", b.pos.x, b.pos.y);
    }
}
