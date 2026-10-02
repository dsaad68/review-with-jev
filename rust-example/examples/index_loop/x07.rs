#[derive(Clone, Copy, Debug)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

struct Frame {
    width: usize,
    pixels: Vec<Pixel>,
}

fn scale(channel: u8, percent: u16) -> u8 {
    let scaled = u16::from(channel) * percent / 100;
    scaled.min(255) as u8
}

impl Frame {
    fn filled(width: usize, height: usize, seed: u8) -> Self {
        let pixels = (0..width * height)
            .map(|n| {
                let v = seed.wrapping_add((n % 251) as u8);
                Pixel { r: v, g: v / 2, b: 255 - v }
            })
            .collect();
        Frame { width, pixels }
    }

    fn brighten(&mut self, percent: u16) {
        for i in 0..self.pixels.len() {
            let p = self.pixels[i];
            self.pixels[i] = Pixel {
                r: scale(p.r, percent),
                g: scale(p.g, percent),
                b: scale(p.b, percent),
            };
        }
    }

    fn luminance(&self) -> f64 {
        let total: f64 = self
            .pixels
            .iter()
            .map(|p| 0.299 * f64::from(p.r) + 0.587 * f64::from(p.g) + 0.114 * f64::from(p.b))
            .sum();
        total / self.pixels.len().max(1) as f64
    }

    fn first_row(&self) -> &[Pixel] {
        &self.pixels[..self.width.min(self.pixels.len())]
    }
}

fn main() {
    let mut frame = Frame::filled(8, 4, 30);
    println!("before: {:.2}", frame.luminance());
    frame.brighten(140);
    println!("after:  {:.2}", frame.luminance());
    println!("row 0: {:?}", &frame.first_row()[..2]);
}
