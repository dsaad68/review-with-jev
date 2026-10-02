use std::collections::VecDeque;

struct Document {
    owner: &'static str,
    title: &'static str,
    pages: u32,
}

struct Spooler {
    queue: VecDeque<Document>,
    printed_pages: u32,
}

impl Spooler {
    fn new() -> Self {
        Spooler { queue: VecDeque::new(), printed_pages: 0 }
    }

    fn enqueue(&mut self, doc: Document, jump: bool) {
        if jump {
            self.queue.push_front(doc);
        } else {
            self.queue.push_back(doc);
        }
    }

    fn cancel(&mut self, owner: &str) -> usize {
        let before = self.queue.len();
        self.queue.retain(|d| d.owner != owner);
        before - self.queue.len()
    }

    fn run(&mut self, page_budget: u32) -> Vec<&'static str> {
        let mut printed = Vec::new();
        while let Some(doc) = self.queue.front() {
            if self.printed_pages + doc.pages > page_budget {
                break;
            }
            self.printed_pages += doc.pages;
            printed.push(doc.title);
            self.queue.pop_front();
        }
        printed
    }
}

fn main() {
    let mut spooler = Spooler::new();
    spooler.enqueue(Document { owner: "maya", title: "quarterly.pdf", pages: 12 }, false);
    spooler.enqueue(Document { owner: "liam", title: "poster.png", pages: 1 }, false);
    spooler.enqueue(Document { owner: "maya", title: "draft.docx", pages: 30 }, false);
    spooler.enqueue(Document { owner: "ceo", title: "board-memo.pdf", pages: 3 }, true);
    spooler.enqueue(Document { owner: "liam", title: "tickets.pdf", pages: 2 }, true);

    println!("first batch: {:?}", spooler.run(20));
    println!("cancelled {} job(s) for maya", spooler.cancel("maya"));
    println!("second batch: {:?}", spooler.run(40));
    println!("pages printed: {}, waiting: {}", spooler.printed_pages, spooler.queue.len());
}
