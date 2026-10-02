use std::fmt;

trait Shape {
    fn covers(&self, x: i32, y: i32) -> bool;
    fn on_edge(&self, x: i32, y: i32) -> bool;
}

struct Rect {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
}

impl Shape for Rect {
    fn covers(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.left + self.width && y >= self.top && y < self.top + self.height
    }

    fn on_edge(&self, x: i32, y: i32) -> bool {
        self.covers(x, y)
            && (x == self.left
                || x == self.left + self.width - 1
                || y == self.top
                || y == self.top + self.height - 1)
    }
}

struct Circle {
    cx: i32,
    cy: i32,
    r: i32,
}

impl Shape for Circle {
    fn covers(&self, x: i32, y: i32) -> bool {
        let (dx, dy) = (x - self.cx, y - self.cy);
        dx * dx + dy * dy <= self.r * self.r
    }

    fn on_edge(&self, x: i32, y: i32) -> bool {
        self.covers(x, y)
            && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|(ox, oy)| !self.covers(x + ox, y + oy))
    }
}

#[derive(Clone, Copy)]
enum Fill {
    Solid,
    Outline,
}

#[derive(Clone, Copy)]
enum Mirror {
    None,
    Horizontal,
}

struct Layer<'a> {
    name: &'a str,
    shape: Box<dyn Shape + 'a>,
    glyph: char,
    fill: Fill,
    visible: bool,
}

impl<'a> Layer<'a> {
    fn new(name: &'a str, shape: Box<dyn Shape + 'a>, glyph: char, fill: Fill) -> Self {
        Layer { name, shape, glyph, fill, visible: true }
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn paints(&self, x: i32, y: i32) -> bool {
        self.visible
            && match self.fill {
                Fill::Solid => self.shape.covers(x, y),
                Fill::Outline => self.shape.on_edge(x, y),
            }
    }
}

struct Canvas<'a> {
    width: i32,
    height: i32,
    layers: Vec<Layer<'a>>,
    mirror: Mirror,
}

impl<'a> Canvas<'a> {
    fn layer_mut(&mut self, name: &str) -> Option<&mut Layer<'a>> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    fn pixel(&self, x: i32, y: i32) -> char {
        let sx = match self.mirror {
            Mirror::None => x,
            Mirror::Horizontal => self.width - 1 - x,
        };
        self.layers
            .iter()
            .rev()
            .find(|layer| layer.paints(sx, y))
            .map_or('.', |layer| layer.glyph)
    }
}

impl fmt::Display for Canvas<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            let row: String = (0..self.width).map(|x| self.pixel(x, y)).collect();
            writeln!(f, "{row}")?;
        }
        Ok(())
    }
}

fn main() {
    let mut canvas = Canvas {
        width: 24,
        height: 10,
        mirror: Mirror::None,
        layers: vec![
            Layer::new("sky", Box::new(Rect { left: 0, top: 0, width: 24, height: 6 }), '~', Fill::Solid),
            Layer::new("sun", Box::new(Circle { cx: 18, cy: 3, r: 3 }), 'o', Fill::Outline),
            Layer::new("house", Box::new(Rect { left: 3, top: 4, width: 8, height: 6 }), '#', Fill::Outline),
            Layer::new("door", Box::new(Rect { left: 6, top: 7, width: 2, height: 3 }), '|', Fill::Solid),
        ],
    };
    print!("{canvas}");

    if let Some(sky) = canvas.layer_mut("sky") {
        sky.set_visible(false);
    }
    canvas.mirror = Mirror::Horizontal;
    println!();
    print!("{canvas}");

    let hidden: Vec<&str> = canvas.layers.iter().filter(|l| !l.visible).map(|l| l.name).collect();
    println!("hidden layers: {}", hidden.join(", "));
}
