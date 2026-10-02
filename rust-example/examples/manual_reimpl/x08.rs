use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Format {
    Hardcover,
    Paperback,
    Audio,
}

#[derive(Debug, Clone)]
struct Volume {
    isbn: String,
    title: String,
    format: Format,
    loans: u32,
    damaged: bool,
}

trait Shelf {
    fn volumes(&self) -> &[Volume];

    fn by_format(&self) -> HashMap<Format, usize> {
        let mut counts = HashMap::new();
        for v in self.volumes() {
            *counts.entry(v.format).or_insert(0) += 1;
        }
        counts
    }

    fn most_loaned(&self) -> Option<&Volume> {
        self.volumes().iter().max_by_key(|v| v.loans)
    }
}

struct Branch {
    name: String,
    stock: Vec<Volume>,
}

impl Shelf for Branch {
    fn volumes(&self) -> &[Volume] {
        &self.stock
    }
}

impl Branch {
    fn new(name: &str) -> Self {
        Branch {
            name: name.to_string(),
            stock: Vec::new(),
        }
    }

    fn acquire(&mut self, isbn: &str, title: &str, format: Format) {
        self.stock.push(Volume {
            isbn: isbn.to_string(),
            title: title.to_string(),
            format,
            loans: 0,
            damaged: false,
        });
    }

    fn lend(&mut self, isbn: &str, times: u32) -> Result<(), LendError> {
        let v = self
            .stock
            .iter_mut()
            .find(|v| v.isbn == isbn)
            .ok_or(LendError::UnknownIsbn)?;
        if v.damaged {
            return Err(LendError::Damaged);
        }
        v.loans += times;
        Ok(())
    }

    fn mark_damaged(&mut self, isbn: &str) {
        if let Some(v) = self.stock.iter_mut().find(|v| v.isbn == isbn) {
            v.damaged = true;
        }
    }

    fn weed(&mut self) {
        let mut kept = Vec::with_capacity(self.stock.len());
        for v in self.stock.drain(..) {
            if !v.damaged {
                kept.push(v);
            }
        }
        self.stock = kept;
    }
}

#[derive(Debug, PartialEq)]
enum LendError {
    UnknownIsbn,
    Damaged,
}

impl fmt::Display for LendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LendError::UnknownIsbn => write!(f, "no such isbn"),
            LendError::Damaged => write!(f, "volume is damaged"),
        }
    }
}

fn main() {
    let mut branch = Branch::new("Riverside");
    branch.acquire("978-0141439518", "Pride and Prejudice", Format::Paperback);
    branch.acquire("978-0553380163", "A Brief History of Time", Format::Hardcover);
    branch.acquire("978-0007525546", "The Hobbit", Format::Audio);
    branch.acquire("978-0261103252", "The Silmarillion", Format::Hardcover);

    for (isbn, times) in [("978-0141439518", 7), ("978-0007525546", 12), ("978-0000000000", 1)] {
        if let Err(e) = branch.lend(isbn, times) {
            println!("lend {isbn}: {e}");
        }
    }
    branch.mark_damaged("978-0553380163");
    if let Err(e) = branch.lend("978-0553380163", 1) {
        println!("lend refused: {e}");
    }
    branch.weed();

    println!("{} holds {} volumes", branch.name, branch.volumes().len());
    let mut formats: Vec<_> = branch.by_format().into_iter().collect();
    formats.sort_by_key(|(_, n)| *n);
    println!("formats: {formats:?}");
    if let Some(v) = branch.most_loaned() {
        println!("most loaned: {} ({} loans)", v.title, v.loans);
    }
}
