use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
struct Job {
    name: String,
    payload: Vec<u32>,
}

fn label_for(prefix: &str, index: usize) -> String {
    if index == 0 {
        prefix.to_string()
    } else {
        format!("{}-{}", prefix, index)
    }
}

fn build_jobs(prefix: &str, batches: &[Vec<u32>]) -> Vec<Job> {
    batches
        .iter()
        .enumerate()
        .map(|(i, b)| Job { name: label_for(prefix, i), payload: b.clone() })
        .collect()
}

fn run(jobs: Vec<Job>) -> Vec<(String, u64)> {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();
    for job in jobs {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let total: u64 = job.payload.iter().map(|&x| x as u64 * x as u64).sum();
            tx.send((job.name, total)).unwrap();
        }));
    }
    drop(tx);
    for h in handles {
        h.join().unwrap();
    }
    let mut results: Vec<(String, u64)> = rx.into_iter().collect();
    results.sort();
    results
}

fn main() {
    let batches = vec![vec![1, 2, 3], vec![4, 5], vec![6, 7, 8, 9]];
    let prefix = String::from("worker");
    let jobs = build_jobs(&prefix, &batches);
    drop(prefix);
    println!("{:?}", jobs[0]);
    for (name, total) in run(jobs) {
        println!("{} => {}", name, total);
    }
}
