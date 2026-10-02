use std::fmt::Write as _;
use std::sync::mpsc;
use std::thread;

enum Progress {
    Started(usize),
    Finished(usize, u64),
}

#[derive(Debug)]
struct ChecksumError {
    chunk: usize,
    reason: &'static str,
}

fn checksum(chunk: usize, data: &[u8]) -> Result<u64, ChecksumError> {
    if data.is_empty() {
        return Err(ChecksumError { chunk, reason: "empty chunk" });
    }
    Ok(data
        .iter()
        .fold(17u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(u64::from(b))))
}

fn main() {
    let chunks: Vec<Vec<u8>> = vec![b"alpha".to_vec(), b"bravo".to_vec(), Vec::new(), b"delta".to_vec()];
    let (tx, rx) = mpsc::channel();

    let monitor = thread::spawn(move || {
        let mut seen = 0;
        for event in rx {
            seen += 1;
            match event {
                Progress::Started(i) => println!("chunk {i} started"),
                Progress::Finished(i, sum) => println!("chunk {i} done: {sum:x}"),
            }
            if seen >= 4 {
                break;
            }
        }
        seen
    });

    let handles: Vec<_> = chunks
        .into_iter()
        .enumerate()
        .map(|(i, data)| {
            let tx = tx.clone();
            thread::spawn(move || {
                let _ = tx.send(Progress::Started(i));
                let result = checksum(i, &data);
                if let Ok(sum) = &result {
                    let _ = tx.send(Progress::Finished(i, *sum));
                }
                result
            })
        })
        .collect();
    drop(tx);

    let mut report = String::new();
    let mut failures = 0;
    for handle in handles {
        match handle.join().expect("worker panicked") {
            Ok(sum) => {
                let _ = writeln!(report, "ok {sum:016x}");
            }
            Err(e) => {
                failures += 1;
                let _ = writeln!(report, "chunk {} failed: {}", e.chunk, e.reason);
            }
        }
    }
    let events = monitor.join().expect("monitor panicked");
    print!("{report}");
    println!("{events} progress events observed, {failures} chunk failures");
}
