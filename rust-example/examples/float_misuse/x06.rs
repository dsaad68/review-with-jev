#[derive(Debug, Clone, Copy)]
enum Curve {
    Linear,
    EaseIn,
    EaseOut,
    Smooth,
}

impl Curve {
    fn apply(self, t: f64) -> f64 {
        match self {
            Curve::Linear => t,
            Curve::EaseIn => t * t,
            Curve::EaseOut => t * (2.0 - t),
            Curve::Smooth => t * t * (3.0 - 2.0 * t),
        }
    }
}

struct Tween {
    from: f64,
    to: f64,
    curve: Curve,
}

impl Tween {
    fn value_at(&self, t: f64) -> f64 {
        self.from + (self.to - self.from) * self.curve.apply(t.clamp(0.0, 1.0))
    }

    fn keyframes(&self, step: f64) -> Vec<(f64, f64)> {
        let mut frames = Vec::new();
        let mut t = 0.0;
        while t <= 1.0 {
            frames.push((t, self.value_at(t)));
            t += step;
        }
        frames
    }
}

fn main() {
    let tweens = [
        Tween { from: 0.0, to: 100.0, curve: Curve::Linear },
        Tween { from: 10.0, to: 250.0, curve: Curve::EaseIn },
        Tween { from: 1.0, to: 0.0, curve: Curve::EaseOut },
        Tween { from: -40.0, to: 40.0, curve: Curve::Smooth },
    ];
    for tw in &tweens {
        let frames = tw.keyframes(0.1);
        let last = frames.last().map_or(f64::NAN, |f| f.1);
        println!("{:?}: {} frames, last value {last:.3}", tw.curve, frames.len());
        let line: Vec<String> = frames.iter().map(|(_, v)| format!("{v:.1}")).collect();
        println!("  {}", line.join(" "));
    }
}
