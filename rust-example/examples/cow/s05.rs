use std::thread;

fn job_name(raw: &str) -> String {
    if raw.trim().is_empty() {
        "unnamed-job".to_string()
    } else {
        raw.trim().to_string()
    }
}

fn main() {
    let inputs: Vec<String> = vec!["backup".into(), "  ".into(), " reindex ".into()];
    let mut handles = Vec::new();
    for (i, raw) in inputs.iter().enumerate() {
        let name = job_name(raw);
        handles.push(thread::spawn(move || {
            format!("worker {} finished {}", i, name)
        }));
    }
    for handle in handles {
        println!("{}", handle.join().unwrap());
    }
}
