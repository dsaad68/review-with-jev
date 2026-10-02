use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn write_report(path: &Path, lines: &[&str]) -> io::Result<usize> {
    let mut file = fs::File::create(path)?;
    let mut bytes = 0;
    for line in lines {
        writeln!(file, "{line}")?;
        bytes += line.len() + 1;
    }
    file.flush()?;
    Ok(bytes)
}

fn main() -> io::Result<()> {
    let path = std::env::temp_dir().join("nightly_build_report.txt");
    let written = write_report(&path, &["alpha passed", "beta passed", "gamma flaky"])?;
    let contents = fs::read_to_string(&path)?;
    println!(
        "wrote {written} bytes, read back {} lines",
        contents.lines().count()
    );
    let _ = fs::remove_file(&path);
    Ok(())
}
