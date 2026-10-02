use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug)]
struct Employee {
    id: u32,
    name: String,
    department: String,
    salary: u32,
}

struct Directory {
    by_name: HashMap<String, Employee>,
}

impl Directory {
    fn new(list: Vec<Employee>) -> Self {
        let by_name = list
            .into_iter()
            .map(|e| (e.name.to_lowercase(), e))
            .collect();
        Directory { by_name }
    }

    fn find(&self, name: String) -> Option<&Employee> {
        let key = name.to_lowercase();
        self.by_name.get(&key)
    }

    fn row(&self, e: &Employee) -> String {
        let mut line = String::new();
        write!(line, "{:>4} | {:<10} | {:<12} | {:>7}", e.id, e.name, e.department, e.salary).unwrap();
        line
    }

    fn report(&self, names: &[&str]) -> Vec<String> {
        let mut rows = Vec::new();
        for n in names {
            if let Some(e) = self.find(n.to_string()) {
                rows.push(self.row(e));
            }
        }
        rows
    }
}

fn main() {
    let dir = Directory::new(vec![
        Employee { id: 1, name: "Ada".into(), department: "Research".into(), salary: 120000 },
        Employee { id: 2, name: "Grace".into(), department: "Compilers".into(), salary: 115000 },
        Employee { id: 3, name: "Linus".into(), department: "Kernel".into(), salary: 110000 },
    ]);
    for line in dir.report(&["grace", "ADA", "bob", "Linus"]) {
        println!("{}", line);
    }
    if let Some(e) = dir.find(String::from("ada")) {
        println!("{:?}", e);
    }
}
