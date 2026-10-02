use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

struct Invoice {
    number: u32,
    client: String,
    due: Date,
    paid: bool,
    amount_cents: u64,
}

fn next_to_chase<'a>(invoices: &'a [Invoice], today: Date) -> Option<&'a Invoice> {
    let overdue: Vec<&Invoice> = invoices
        .iter()
        .filter(|inv| !inv.paid && inv.due < today)
        .collect();
    overdue.first().copied()
}

fn main() {
    let d = |year, month, day| Date { year, month, day };
    let invoices = vec![
        Invoice { number: 1001, client: "Acme".into(), due: d(2024, 3, 1), paid: true, amount_cents: 120000 },
        Invoice { number: 1002, client: "Globex".into(), due: d(2024, 3, 15), paid: false, amount_cents: 56000 },
        Invoice { number: 1003, client: "Initech".into(), due: d(2024, 4, 2), paid: false, amount_cents: 9900 },
        Invoice { number: 1004, client: "Umbrella".into(), due: d(2024, 6, 30), paid: false, amount_cents: 31000 },
    ];
    let today = d(2024, 5, 10);

    match next_to_chase(&invoices, today) {
        Some(inv) => println!(
            "chase #{} from {} (due {}, ${:.2})",
            inv.number,
            inv.client,
            inv.due,
            inv.amount_cents as f64 / 100.0
        ),
        None => println!("nothing overdue as of {today}"),
    }
}
