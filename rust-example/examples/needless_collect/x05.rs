#[derive(Debug, Clone, Copy, PartialEq)]
enum Category {
    Food,
    Travel,
    Office,
}

struct Expense {
    category: Category,
    cents: u64,
    reimbursable: bool,
}

struct Report {
    total: u64,
    largest: u64,
    items: usize,
}

fn summarize(expenses: &[Expense], category: Category) -> Report {
    let matching: Vec<&Expense> = expenses
        .iter()
        .filter(|e| e.category == category && e.reimbursable)
        .collect();

    let mut report = Report { total: 0, largest: 0, items: 0 };
    for e in matching {
        report.total += e.cents;
        report.largest = report.largest.max(e.cents);
        report.items += 1;
    }
    report
}

fn dollars(cents: u64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

fn main() {
    let expenses = vec![
        Expense { category: Category::Food, cents: 1850, reimbursable: true },
        Expense { category: Category::Travel, cents: 42000, reimbursable: true },
        Expense { category: Category::Food, cents: 990, reimbursable: false },
        Expense { category: Category::Office, cents: 4599, reimbursable: true },
        Expense { category: Category::Food, cents: 3125, reimbursable: true },
        Expense { category: Category::Travel, cents: 8800, reimbursable: false },
    ];

    for category in [Category::Food, Category::Travel, Category::Office] {
        let r = summarize(&expenses, category);
        println!(
            "{:?}: {} items, total {}, largest {}",
            category,
            r.items,
            dollars(r.total),
            dollars(r.largest)
        );
    }
}
