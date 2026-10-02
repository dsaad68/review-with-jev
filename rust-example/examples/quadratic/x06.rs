#[derive(Debug)]
struct PrintJob {
    id: u32,
    pages: u32,
    attempts: u8,
}

struct Printer {
    paper: u32,
    refill: u32,
    jammed_every: u32,
    printed: u32,
}

impl Printer {
    fn try_print(&mut self, job: &PrintJob) -> Result<(), String> {
        if self.paper < job.pages {
            self.paper += self.refill;
            return Err(format!("job {} waiting for paper", job.id));
        }
        self.printed += 1;
        if self.printed % self.jammed_every == 0 {
            return Err(format!("job {} jammed", job.id));
        }
        self.paper -= job.pages;
        Ok(())
    }
}

fn run_spool(mut jobs: Vec<PrintJob>, printer: &mut Printer, max_attempts: u8) -> (Vec<u32>, Vec<u32>) {
    let mut done = Vec::new();
    let mut dropped = Vec::new();
    while !jobs.is_empty() {
        let mut job = jobs.remove(0);
        match printer.try_print(&job) {
            Ok(()) => done.push(job.id),
            Err(reason) => {
                job.attempts += 1;
                if job.attempts >= max_attempts {
                    println!("giving up: {}", reason);
                    dropped.push(job.id);
                } else {
                    println!("retry later: {}", reason);
                    jobs.push(job);
                }
            }
        }
    }
    (done, dropped)
}

fn main() {
    let jobs: Vec<PrintJob> = (1..=15)
        .map(|id| PrintJob {
            id,
            pages: (id * 7) % 23 + 1,
            attempts: 0,
        })
        .collect();
    let mut printer = Printer {
        paper: 40,
        refill: 25,
        jammed_every: 6,
        printed: 0,
    };
    let (done, dropped) = run_spool(jobs, &mut printer, 3);
    println!("printed {:?}", done);
    println!("dropped {:?}", dropped);
    println!("paper left {}", printer.paper);
}
