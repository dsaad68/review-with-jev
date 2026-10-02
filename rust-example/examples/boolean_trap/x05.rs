#[derive(Clone, Copy)]
enum Align { Left, Right, Center }
#[derive(Clone, Copy)]
enum Border { Plain, Boxed }

struct Table {
    headers: Vec<String>,
    aligns: Vec<Align>,
    rows: Vec<Vec<String>>,
    border: Border,
}

impl Table {
    fn new(headers: &[&str]) -> Self {
        Table {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            aligns: vec![Align::Left; headers.len()],
            rows: Vec::new(),
            border: Border::Plain,
        }
    }

    fn align(mut self, column: usize, align: Align) -> Self {
        if let Some(slot) = self.aligns.get_mut(column) {
            *slot = align;
        }
        self
    }

    fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    fn row(&mut self, cells: Vec<String>) {
        self.rows.push(cells);
    }

    fn render(&self) -> String {
        let widths: Vec<usize> = (0..self.headers.len())
            .map(|c| {
                let body = self.rows.iter().map(|r| r.get(c).map_or(0, |s| s.chars().count()));
                body.chain(std::iter::once(self.headers[c].chars().count())).max().unwrap_or(0)
            })
            .collect();
        let (left, sep, right) = match self.border {
            Border::Plain => ("", "  ", ""),
            Border::Boxed => ("| ", " | ", " |"),
        };
        let mut out = String::new();
        for cells in std::iter::once(&self.headers).chain(self.rows.iter()) {
            let padded: Vec<String> = widths
                .iter()
                .zip(&self.aligns)
                .enumerate()
                .map(|(c, (&w, align))| {
                    let text = cells.get(c).map_or("", |s| s.as_str());
                    match align {
                        Align::Left => format!("{text:<w$}"),
                        Align::Right => format!("{text:>w$}"),
                        Align::Center => format!("{text:^w$}"),
                    }
                })
                .collect();
            out.push_str(&format!("{left}{}{right}\n", padded.join(sep)));
        }
        out
    }
}

fn main() {
    let mut table = Table::new(&["planet", "moons", "type"])
        .align(1, Align::Right)
        .align(2, Align::Center)
        .border(Border::Boxed);
    for (name, moons, kind) in [("Mercury", 0, "rocky"), ("Jupiter", 95, "gas giant"), ("Neptune", 16, "ice giant")] {
        table.row(vec![name.to_string(), moons.to_string(), kind.to_string()]);
    }
    print!("{}", table.render());
}
