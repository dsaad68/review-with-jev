use std::collections::BTreeMap;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
enum Cell<'a> {
    Text(&'a str),
    Int(i64),
    Money(u64),
    Flag(bool),
}

trait ToRow {
    const HEADERS: &'static [&'static str];
    fn cells(&self) -> Vec<Cell<'_>>;
}

#[derive(Debug)]
struct Order {
    id: u32,
    customer: String,
    region: String,
    note: String,
    quantity: i64,
    total_cents: u64,
    express: bool,
}

impl ToRow for Order {
    const HEADERS: &'static [&'static str] = &["id", "customer", "region", "qty", "total", "express", "note"];

    fn cells(&self) -> Vec<Cell<'_>> {
        vec![
            Cell::Int(self.id as i64),
            Cell::Text(&self.customer),
            Cell::Text(&self.region),
            Cell::Int(self.quantity),
            Cell::Money(self.total_cents),
            Cell::Flag(self.express),
            Cell::Text(&self.note),
        ]
    }
}

struct RegionTotal<'a> {
    region: &'a str,
    orders: i64,
    cents: u64,
}

impl<'a> ToRow for RegionTotal<'a> {
    const HEADERS: &'static [&'static str] = &["region", "orders", "revenue"];

    fn cells(&self) -> Vec<Cell<'_>> {
        vec![Cell::Text(self.region), Cell::Int(self.orders), Cell::Money(self.cents)]
    }
}

struct CsvWriter<W: Write> {
    out: W,
    delimiter: char,
    rows: usize,
}

impl<W: Write> CsvWriter<W> {
    fn new(out: W, delimiter: char) -> Self {
        CsvWriter { out, delimiter, rows: 0 }
    }

    fn escape_field(&self, raw: &str) -> String {
        let needs_quotes = raw.contains(|c: char| c == self.delimiter || c == '"' || c == '\n' || c == '\r')
            || raw.starts_with(' ')
            || raw.ends_with(' ');
        if !needs_quotes {
            return raw.to_string();
        }
        let mut quoted = String::with_capacity(raw.len() + 2);
        quoted.push('"');
        for c in raw.chars() {
            if c == '"' {
                quoted.push('"');
            }
            quoted.push(c);
        }
        quoted.push('"');
        quoted
    }

    fn write_line<'s, I>(&mut self, fields: I) -> io::Result<()>
    where
        I: IntoIterator<Item = &'s str>,
    {
        for (i, field) in fields.into_iter().enumerate() {
            if i > 0 {
                write!(self.out, "{}", self.delimiter)?;
            }
            let escaped = self.escape_field(field);
            self.out.write_all(escaped.as_bytes())?;
        }
        self.out.write_all(b"\n")
    }

    fn write_all<R: ToRow>(&mut self, rows: &[R]) -> io::Result<()> {
        self.write_line(R::HEADERS.iter().copied())?;
        let mut scratch = String::new();
        for row in rows {
            let cells = row.cells();
            let mut rendered: Vec<String> = Vec::with_capacity(cells.len());
            for cell in &cells {
                scratch.clear();
                match cell {
                    Cell::Text(t) => scratch.push_str(t),
                    Cell::Int(n) => scratch.push_str(&n.to_string()),
                    Cell::Money(c) => scratch.push_str(&format!("{}.{:02}", c / 100, c % 100)),
                    Cell::Flag(b) => scratch.push_str(if *b { "Y" } else { "N" }),
                }
                rendered.push(scratch.clone());
            }
            self.write_line(rendered.iter().map(String::as_str))?;
            self.rows += 1;
        }
        Ok(())
    }

    fn into_inner(self) -> W {
        self.out
    }
}

fn summarize(orders: &[Order]) -> Vec<RegionTotal<'_>> {
    let mut acc: BTreeMap<&str, (i64, u64)> = BTreeMap::new();
    for o in orders {
        let e = acc.entry(o.region.as_str()).or_insert((0, 0));
        e.0 += 1;
        e.1 += o.total_cents;
    }
    acc.into_iter()
        .map(|(region, (orders, cents))| RegionTotal { region, orders, cents })
        .collect()
}

fn main() -> io::Result<()> {
    let orders = vec![
        Order { id: 1, customer: "Acme Corp".into(), region: "EU".into(), note: "".into(), quantity: 3, total_cents: 12999, express: false },
        Order { id: 2, customer: "Smith, Jones & Co".into(), region: "US".into(), note: "leave at \"back door\"".into(), quantity: 1, total_cents: 4550, express: true },
        Order { id: 3, customer: "Initech".into(), region: "US".into(), note: "two\nboxes".into(), quantity: 2, total_cents: 8000, express: false },
        Order { id: 4, customer: "Globex".into(), region: "APAC".into(), note: " fragile".into(), quantity: 10, total_cents: 99900, express: true },
    ];
    let mut writer = CsvWriter::new(Vec::new(), ',');
    writer.write_all(&orders)?;
    let rows = writer.rows;
    let bytes = writer.into_inner();
    print!("{}", String::from_utf8_lossy(&bytes));
    println!("-- {} rows, {} bytes", rows, bytes.len());

    let totals = summarize(&orders);
    let stdout = io::stdout();
    let mut tsv = CsvWriter::new(stdout.lock(), '\t');
    tsv.write_all(&totals)?;
    Ok(())
}
