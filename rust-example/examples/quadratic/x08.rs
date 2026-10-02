use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Write};

const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

trait Priced {
    fn sku(&self) -> &str;
    fn unit_price(&self) -> u32;
    fn category(&self) -> Category;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Category {
    Grocery,
    Hardware,
    Toys,
}

struct Product {
    sku: String,
    title: String,
    price: u32,
    category: Category,
}

impl Priced for Product {
    fn sku(&self) -> &str {
        &self.sku
    }
    fn unit_price(&self) -> u32 {
        self.price
    }
    fn category(&self) -> Category {
        self.category
    }
}

struct Catalog<'a, P: Priced> {
    by_sku: HashMap<&'a str, &'a P>,
    price_points: Vec<u32>,
}

impl<'a, P: Priced> Catalog<'a, P> {
    fn new(items: &'a [P]) -> Self {
        let by_sku = items.iter().map(|p| (p.sku(), p)).collect();
        let mut price_points: Vec<u32> = items.iter().map(|p| p.unit_price()).collect();
        price_points.sort_unstable();
        price_points.dedup();
        Catalog { by_sku, price_points }
    }

    fn get(&self, sku: &str) -> Option<&'a P> {
        self.by_sku.get(sku).copied()
    }

    fn percentile(&self, price: u32) -> f64 {
        let rank = match self.price_points.binary_search(&price) {
            Ok(i) | Err(i) => i,
        };
        rank as f64 * 100.0 / self.price_points.len().max(1) as f64
    }
}

struct Sale<'s> {
    sku: &'s str,
    day: usize,
    qty: u32,
}

fn parse_sales(text: &str) -> Result<Vec<Sale<'_>>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let mut parts = line.split(',');
        let (Some(sku), Some(day), Some(qty)) = (parts.next(), parts.next(), parts.next()) else {
            return Err(format!("line {}: missing field", n + 1));
        };
        let day = DAYS
            .iter()
            .position(|d| *d == day.trim())
            .ok_or_else(|| format!("line {}: unknown day {:?}", n + 1, day))?;
        let qty = qty
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("line {}: {}", n + 1, e))?;
        out.push(Sale { sku: sku.trim(), day, qty });
    }
    Ok(out)
}

fn weekly_grid<'a, 's, P: Priced>(
    catalog: &Catalog<'a, P>,
    sales: &[Sale<'s>],
) -> (HashMap<&'a str, [u64; 7]>, Vec<&'s str>) {
    let mut grid: HashMap<&'a str, [u64; 7]> = HashMap::new();
    let mut unknown = Vec::new();
    for s in sales {
        match catalog.get(s.sku) {
            Some(p) => {
                grid.entry(p.sku()).or_insert([0; 7])[s.day] +=
                    u64::from(s.qty) * u64::from(p.unit_price());
            }
            None => unknown.push(s.sku),
        }
    }
    (grid, unknown)
}

fn render<P: Priced>(catalog: &Catalog<'_, P>, grid: &HashMap<&str, [u64; 7]>) -> Result<String, fmt::Error> {
    let mut rows: Vec<(&str, &[u64; 7])> = grid.iter().map(|(k, v)| (*k, v)).collect();
    rows.sort_by_key(|(sku, _)| *sku);
    let mut out = String::new();
    write!(out, "{:<8}", "sku")?;
    for d in DAYS {
        write!(out, "{:>8}", d)?;
    }
    writeln!(out, "{:>6}", "pct")?;
    let mut by_category: BTreeMap<Category, u64> = BTreeMap::new();
    for (sku, week) in &rows {
        write!(out, "{:<8}", sku)?;
        for cents in week.iter() {
            write!(out, "{:>5}.{:02}", cents / 100, cents % 100)?;
        }
        if let Some(p) = catalog.get(sku) {
            writeln!(out, "{:>6.1}", catalog.percentile(p.unit_price()))?;
            *by_category.entry(p.category()).or_insert(0) += week.iter().sum::<u64>();
        }
    }
    for (cat, total) in &by_category {
        writeln!(out, "{:?}: {}.{:02}", cat, total / 100, total % 100)?;
    }
    Ok(out)
}

fn main() {
    let products = vec![
        Product { sku: "G100".into(), title: "Oat milk".into(), price: 249, category: Category::Grocery },
        Product { sku: "G101".into(), title: "Rye bread".into(), price: 389, category: Category::Grocery },
        Product { sku: "H200".into(), title: "Hex keys".into(), price: 1299, category: Category::Hardware },
        Product { sku: "H201".into(), title: "Wood glue".into(), price: 549, category: Category::Hardware },
        Product { sku: "T300".into(), title: "Kite".into(), price: 1899, category: Category::Toys },
    ];
    let sales_text = "G100,Mon,3\nG101,Mon,1\nH200,Tue,2\nT300,Sat,1\nG100,Sat,6\nX999,Sun,4\nH201,Fri,2\nG101,Sun,2";
    let catalog = Catalog::new(&products);
    for p in &products {
        println!("{} {:<10} {:>5.1}%", p.sku, p.title, catalog.percentile(p.price));
    }
    let sales = match parse_sales(sales_text) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let (grid, unknown) = weekly_grid(&catalog, &sales);
    match render(&catalog, &grid) {
        Ok(report) => print!("{}", report),
        Err(e) => eprintln!("render failed: {}", e),
    }
    if !unknown.is_empty() {
        println!("unknown skus: {:?}", unknown);
    }
}
