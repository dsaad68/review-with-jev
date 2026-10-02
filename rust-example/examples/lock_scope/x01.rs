use std::fs::{self, File};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;

fn flush_entries(entries: &Mutex<Vec<String>>, out: &mut File) -> io::Result<usize> {
    let mut pending = entries.lock().unwrap();
    for line in pending.iter() {
        writeln!(out, "{line}")?;
    }
    out.sync_all()?;
    let written = pending.len();
    pending.clear();
    Ok(written)
}

fn main() -> io::Result<()> {
    let entries = Arc::new(Mutex::new(Vec::new()));
    let shared = Arc::clone(&entries);
    let producer = thread::spawn(move || {
        for n in 0..20 {
            shared.lock().unwrap().push(format!("event {n}"));
        }
    });
    producer.join().unwrap();
    let path = std::env::temp_dir().join("journal_x01.txt");
    let mut out = File::create(&path)?;
    println!("wrote {} lines", flush_entries(&entries, &mut out)?);
    fs::remove_file(path)
}
