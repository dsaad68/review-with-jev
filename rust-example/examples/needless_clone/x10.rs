use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

#[derive(Clone, Debug)]
struct Job {
    id: u32,
    url: String,
    payload_kb: u32,
}

#[derive(Clone, Debug)]
enum Outcome {
    Uploaded { id: u32, host: String, ms: u32 },
    Rejected { id: u32, reason: String },
}

trait Policy: Send + Sync {
    fn check(&self, job: &Job) -> Result<(), String>;
}

struct SizeLimit {
    max_kb: u32,
}

impl Policy for SizeLimit {
    fn check(&self, job: &Job) -> Result<(), String> {
        if job.payload_kb > self.max_kb {
            Err(format!("{}kb exceeds {}kb", job.payload_kb, self.max_kb))
        } else {
            Ok(())
        }
    }
}

struct SchemeAllow {
    schemes: Vec<&'static str>,
}

impl Policy for SchemeAllow {
    fn check(&self, job: &Job) -> Result<(), String> {
        let scheme = job.url.split("://").next().unwrap_or("");
        if self.schemes.iter().any(|s| *s == scheme) {
            Ok(())
        } else {
            Err(format!("scheme '{scheme}' not allowed"))
        }
    }
}

fn host_of(url: &str) -> &str {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    rest.split('/').next().unwrap_or(rest)
}

fn process(job: Job, policies: &[Box<dyn Policy>]) -> Outcome {
    for p in policies {
        if let Err(reason) = p.check(&job) {
            return Outcome::Rejected { id: job.id, reason };
        }
    }
    let host = String::from(host_of(&job.url));
    let ms = 20 + job.payload_kb / 8 + (host.len() as u32 % 5) * 3;
    Outcome::Uploaded { id: job.id, host, ms }
}

struct Summary {
    per_host: HashMap<String, (u32, u32)>,
    rejected: Vec<(u32, String)>,
}

impl Summary {
    fn build(outcomes: Vec<Outcome>) -> Summary {
        let mut per_host = HashMap::new();
        let mut rejected = Vec::new();
        for o in outcomes {
            match o {
                Outcome::Uploaded { host, ms, .. } => {
                    let e = per_host.entry(host).or_insert((0, 0));
                    e.0 += 1;
                    e.1 += ms;
                }
                Outcome::Rejected { id, reason } => rejected.push((id, reason)),
            }
        }
        rejected.sort_by_key(|r| r.0);
        Summary { per_host, rejected }
    }

    fn print(&self) {
        let mut hosts: Vec<_> = self.per_host.iter().collect();
        hosts.sort_by(|a, b| a.0.cmp(b.0));
        for (host, (n, ms)) in hosts {
            println!("{host}: {n} uploads, avg {} ms", ms / n);
        }
        for (id, reason) in &self.rejected {
            println!("job {id} rejected: {reason}");
        }
    }
}

fn main() {
    let policies: Arc<Vec<Box<dyn Policy>>> = Arc::new(vec![
        Box::new(SizeLimit { max_kb: 4096 }),
        Box::new(SchemeAllow { schemes: vec!["https", "sftp"] }),
    ]);

    let jobs = vec![
        Job { id: 1, url: String::from("https://cdn.example.org/a.png"), payload_kb: 512 },
        Job { id: 2, url: String::from("http://legacy.example.org/b.bin"), payload_kb: 64 },
        Job { id: 3, url: String::from("sftp://vault.example.org/c.tar"), payload_kb: 3900 },
        Job { id: 4, url: String::from("https://cdn.example.org/d.mp4"), payload_kb: 9000 },
        Job { id: 5, url: String::from("https://cdn.example.org/e.css"), payload_kb: 12 },
        Job { id: 6, url: String::from("sftp://vault.example.org/f.sql"), payload_kb: 2048 },
    ];

    let workers = 3;
    let mut buckets: Vec<Vec<Job>> = (0..workers).map(|_| Vec::new()).collect();
    for (i, job) in jobs.into_iter().enumerate() {
        buckets[i % workers].push(job);
    }

    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();
    for bucket in buckets {
        let tx = tx.clone();
        let policies = Arc::clone(&policies);
        handles.push(thread::spawn(move || {
            for job in bucket {
                let outcome = process(job, &policies);
                if tx.send(outcome).is_err() {
                    break;
                }
            }
        }));
    }
    drop(tx);

    let received: Vec<Outcome> = rx.iter().collect();
    for h in handles {
        h.join().unwrap();
    }

    println!("{} outcomes from {workers} workers", received.len());
    let summary = Summary::build(received.clone());
    summary.print();
}
