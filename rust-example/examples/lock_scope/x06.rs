use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

struct Job {
    id: u32,
    pages: u32,
}

#[derive(Default)]
struct Spooler {
    queue: Mutex<VecDeque<Job>>,
}

impl Spooler {
    fn submit(&self, job: Job) {
        self.queue.lock().unwrap().push_back(job);
    }

    fn dispatch_all(&self, printer: &Sender<Job>, acks: &Receiver<u32>) -> u32 {
        let mut queue = self.queue.lock().unwrap();
        let mut printed = 0;
        while let Some(job) = queue.pop_front() {
            let id = job.id;
            if printer.send(job).is_err() {
                break;
            }
            match acks.recv() {
                Ok(done) if done == id => printed += 1,
                _ => break,
            }
        }
        printed
    }
}

fn main() {
    let spooler = Arc::new(Spooler::default());
    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let (ack_tx, ack_rx) = mpsc::channel();
    let printer = thread::spawn(move || {
        let mut total_pages = 0;
        for job in job_rx {
            total_pages += job.pages;
            if ack_tx.send(job.id).is_err() {
                break;
            }
        }
        total_pages
    });
    let submitter = {
        let spooler = Arc::clone(&spooler);
        thread::spawn(move || {
            for id in 0..8 {
                spooler.submit(Job { id, pages: id % 3 + 1 });
            }
        })
    };
    submitter.join().unwrap();
    let printed = spooler.dispatch_all(&job_tx, &ack_rx);
    drop(job_tx);
    println!("printed {printed} jobs, {} pages", printer.join().unwrap());
}
