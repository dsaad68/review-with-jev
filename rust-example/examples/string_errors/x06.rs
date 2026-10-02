use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;

struct Score<'a> {
    student: &'a str,
    points: u32,
}

fn parse_score(line: &str) -> Option<Result<Score<'_>, ParseIntError>> {
    let (student, points) = line.split_once(':')?;
    Some(points.trim().parse().map(|points| Score { student: student.trim(), points }))
}

fn load_gradebook(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

fn letter(points: u32) -> char {
    match points {
        90.. => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        _ => 'F',
    }
}

fn main() -> Result<(), String> {
    let path = std::env::temp_dir().join("gradebook_week3.txt");
    fs::write(&path, "ana: 93\nbo: 78\ncarmen: 85\ndev: 61\n")
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;

    let text = load_gradebook(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;

    let mut by_letter: BTreeMap<char, Vec<&str>> = BTreeMap::new();
    for (n, line) in text.lines().enumerate() {
        let score = parse_score(line)
            .ok_or_else(|| format!("line {}: expected 'name: points'", n + 1))?
            .map_err(|e| format!("line {}: {e}", n + 1))?;
        by_letter.entry(letter(score.points)).or_default().push(score.student);
    }

    for (grade, students) in &by_letter {
        println!("{grade}: {}", students.join(", "));
    }
    fs::remove_file(&path).map_err(|e| format!("cannot clean up: {e}"))?;
    Ok(())
}
