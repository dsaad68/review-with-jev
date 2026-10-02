use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone)]
struct Product {
    sku: String,
    title: String,
    tags: Vec<String>,
    price_cents: u64,
}

impl Product {
    fn new(sku: &str, title: &str, tags: &[&str], price_cents: u64) -> Self {
        Product {
            sku: sku.to_string(),
            title: title.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            price_cents,
        }
    }
}

trait Ranker {
    fn score(&self, product: &Product, hits: usize) -> i64;
}

struct HitsThenPrice;

impl Ranker for HitsThenPrice {
    fn score(&self, product: &Product, hits: usize) -> i64 {
        hits as i64 * 1_000_000 - product.price_cents as i64
    }
}

struct SearchIndex {
    products: Vec<Product>,
    postings: HashMap<String, BTreeSet<usize>>,
    aliases: HashMap<String, String>,
}

fn clean_term(raw: &str) -> String {
    let mut cleaned = String::new();
    for c in raw.chars().filter(|c| c.is_alphanumeric()) {
        cleaned.extend(c.to_lowercase());
    }
    cleaned
}

impl SearchIndex {
    fn new(products: Vec<Product>) -> Self {
        let mut postings: HashMap<String, BTreeSet<usize>> = HashMap::new();
        for (i, p) in products.iter().enumerate() {
            let words = p.title.split_whitespace().chain(p.tags.iter().map(String::as_str));
            for word in words {
                let term = clean_term(word);
                if !term.is_empty() {
                    postings.entry(term).or_default().insert(i);
                }
            }
        }
        SearchIndex { products, postings, aliases: HashMap::new() }
    }

    fn add_alias(&mut self, from: &str, to: &str) {
        self.aliases.insert(clean_term(from), clean_term(to));
    }

    fn lookup(&self, term: &String) -> Option<&BTreeSet<usize>> {
        let key = term.to_lowercase();
        let resolved = self.aliases.get(&key).unwrap_or(&key);
        self.postings.get(resolved)
    }

    fn search<R: Ranker>(&self, query: &str, ranker: &R) -> Vec<(&Product, i64)> {
        let mut hits: HashMap<usize, usize> = HashMap::new();
        for raw in query.split_whitespace() {
            let term = clean_term(raw);
            if let Some(ids) = self.lookup(&term) {
                for &id in ids {
                    *hits.entry(id).or_insert(0) += 1;
                }
            }
        }
        let mut ranked: Vec<(&Product, i64)> = hits
            .into_iter()
            .map(|(id, n)| (&self.products[id], ranker.score(&self.products[id], n)))
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.sku.cmp(&b.0.sku)));
        ranked
    }

    fn suggested_tags(&self, sku: &str) -> Vec<String> {
        let Some(product) = self.products.iter().find(|p| p.sku == sku) else {
            return Vec::new();
        };
        let mut tags = product.tags.clone();
        for word in product.title.split_whitespace() {
            let term = clean_term(word);
            if term.len() > 3 && self.postings.get(&term).map_or(0, |s| s.len()) > 1 {
                tags.push(term);
            }
        }
        tags.sort();
        tags.dedup();
        tags
    }

    fn discounted(&self, percent: u64, tag: &str) -> Vec<Product> {
        self.products
            .iter()
            .filter(|p| p.tags.iter().any(|t| t == tag))
            .cloned()
            .map(|mut p| {
                p.price_cents = p.price_cents * (100 - percent) / 100;
                p.tags.push("sale".to_string());
                p
            })
            .collect()
    }

    fn top_by<K, F>(&self, n: usize, key: F) -> Vec<&Product>
    where
        K: Ord,
        F: Fn(&Product) -> K,
    {
        let mut all: Vec<&Product> = self.products.iter().collect();
        all.sort_by_key(|p| std::cmp::Reverse(key(p)));
        all.truncate(n);
        all
    }
}

fn main() {
    let catalog = vec![
        Product::new("KB-01", "Mechanical Keyboard Brown Switches", &["keyboard", "peripherals"], 8999),
        Product::new("KB-02", "Compact Wireless Keyboard", &["keyboard", "wireless"], 4999),
        Product::new("MS-01", "Wireless Mouse", &["mouse", "wireless", "peripherals"], 2599),
        Product::new("HD-01", "Studio Headphones", &["audio", "wired"], 12999),
        Product::new("HD-02", "Wireless Earbuds", &["audio", "wireless"], 7999),
    ];
    let mut index = SearchIndex::new(catalog);
    index.add_alias("kbd", "keyboard");
    index.add_alias("Cordless", "wireless");

    for query in ["cordless kbd", "Wireless audio", "mouse!", "speaker"] {
        let results = index.search(query, &HitsThenPrice);
        let skus: Vec<&str> = results.iter().map(|(p, _)| p.sku.as_str()).collect();
        println!("{:<16} -> {:?}", query, skus);
    }
    println!("tags KB-02: {:?}", index.suggested_tags("KB-02"));
    for p in index.discounted(20, "wireless") {
        println!("sale {} {} {:?}", p.sku, p.price_cents, p.tags);
    }
    let priciest: Vec<&str> = index.top_by(2, |p| p.price_cents).iter().map(|p| p.title.as_str()).collect();
    println!("priciest: {:?}", priciest);
}
