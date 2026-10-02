use std::path::{Path, PathBuf};

fn resolve_path(base: &Path, target: &Path) -> PathBuf {
    if target.is_absolute() {
        target.to_path_buf()
    } else {
        base.join(target)
    }
}

fn main() {
    let base = Path::new("/home/alice/project");
    let targets = ["src/main.rs", "/etc/hosts", "Cargo.toml"];
    for t in targets {
        let resolved = resolve_path(base, Path::new(t));
        println!("{} -> {}", t, resolved.display());
    }
}
