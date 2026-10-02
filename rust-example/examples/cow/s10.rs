fn file_extension(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext,
        _ => "",
    }
}

fn main() {
    let files = ["report.pdf", "archive.tar.gz", ".bashrc", "README"];
    for file in files {
        let ext = file_extension(file);
        if ext.is_empty() {
            println!("{}: no extension", file);
        } else {
            println!("{}: {}", file, ext);
        }
    }
}
