use std::collections::BTreeMap;

type Tree = BTreeMap<String, u32>;

#[derive(Default, Debug)]
struct Report {
    copied: Vec<String>,
    deleted: Vec<String>,
    unchanged: usize,
}

fn sync(source: &Tree, target: &mut Tree, delete_extra: bool, dry_run: bool) -> Report {
    let mut report = Report::default();
    let mut pending = Vec::new();
    for (path, &version) in source {
        if target.get(path) == Some(&version) {
            report.unchanged += 1;
        } else {
            report.copied.push(path.clone());
            if !dry_run {
                pending.push((path.clone(), version));
            }
        }
    }
    target.extend(pending);

    if delete_extra {
        let extra: Vec<String> =
            target.keys().filter(|k| !source.contains_key(*k)).cloned().collect();
        for path in extra {
            if !dry_run {
                target.remove(&path);
            }
            report.deleted.push(path);
        }
    }
    report
}

fn tree(entries: &[(&str, u32)]) -> Tree {
    entries.iter().map(|&(p, v)| (p.to_string(), v)).collect()
}

fn main() {
    let laptop = tree(&[("notes.md", 4), ("photos/cat.jpg", 1), ("src/main.rs", 9)]);
    let mut backup = tree(&[("notes.md", 3), ("photos/cat.jpg", 1), ("old.log", 2)]);

    let preview = sync(&laptop, &mut backup, true, true);
    println!("preview: {preview:?}");
    println!("backup untouched: {backup:?}");

    let first = sync(&laptop, &mut backup, false, false);
    println!("first pass: {first:?}");

    let second = sync(&laptop, &mut backup, true, false);
    println!("second pass: {second:?}");
    println!("backup now: {backup:?}");
}
