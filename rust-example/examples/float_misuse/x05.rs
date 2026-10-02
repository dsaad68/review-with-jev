const G: f64 = 9.81;
const DT: f64 = 0.001;
const MAX_STEPS: u32 = 20_000;

#[derive(Debug, Clone, Copy)]
struct Body {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

struct Flight {
    body: Body,
    peak: f64,
    landed_at: f64,
}

impl Flight {
    fn launch(speed: f64, angle_deg: f64) -> Self {
        let a = angle_deg.to_radians();
        let body = Body { x: 0.0, y: 0.0, vx: speed * a.cos(), vy: speed * a.sin() };
        Flight { body, peak: 0.0, landed_at: 0.0 }
    }

    fn run(&mut self) {
        for step in 1..=MAX_STEPS {
            let b = &mut self.body;
            b.vy -= G * DT;
            b.x += b.vx * DT;
            b.y += b.vy * DT;
            self.peak = self.peak.max(b.y);
            if b.y < 0.0 {
                self.landed_at = f64::from(step) * DT;
                return;
            }
        }
    }

    fn summary(&self) -> String {
        if self.landed_at == 0.0 {
            return format!("still airborne at x={:.2}", self.body.x);
        }
        format!("range {:.2} m, peak {:.2} m, time {:.3} s", self.body.x, self.peak, self.landed_at)
    }
}

fn analytic_range(speed: f64, angle_deg: f64) -> f64 {
    speed * speed * (2.0 * angle_deg.to_radians()).sin() / G
}

fn main() {
    for (speed, angle) in [(20.0, 45.0), (20.0, 30.0), (35.0, 60.0), (300.0, 89.0)] {
        let mut f = Flight::launch(speed, angle);
        f.run();
        let expected = analytic_range(speed, angle);
        let close = (f.body.x - expected).abs() / expected < 0.01;
        println!("v={speed} a={angle}: {} (analytic {expected:.2}, within 1%: {close})", f.summary());
    }
}
