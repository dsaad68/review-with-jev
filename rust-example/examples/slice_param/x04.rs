use std::fmt::Write;
use std::path::{Path, PathBuf};

struct Artifact {
    location: PathBuf,
    bytes: u64,
}

impl Artifact {
    fn new(location: PathBuf, bytes: u64) -> Self {
        Artifact { location, bytes }
    }
}

fn extend_path(target: &mut PathBuf, segments: &[&str]) {
    for segment in segments {
        target.push(segment);
    }
}

fn with_extension(base: &Path, stem: &str, ext: &str) -> PathBuf {
    base.join(stem).with_extension(ext)
}

fn append_summary(report: &mut String, artifact: &Artifact) {
    let kind = artifact
        .location
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("none");
    let _ = writeln!(
        report,
        "{} [{}] {} bytes",
        artifact.location.display(),
        kind,
        artifact.bytes
    );
}

fn total_size(artifacts: &[Artifact]) -> u64 {
    artifacts.iter().map(|a| a.bytes).sum()
}

fn main() {
    let mut root = PathBuf::from("/var/build");
    extend_path(&mut root, &["release", "x86_64"]);

    let artifacts = vec![
        Artifact::new(with_extension(&root, "server", "bin"), 48_213),
        Artifact::new(with_extension(&root, "client", "wasm"), 12_004),
        Artifact::new(root.join("README"), 2_310),
    ];

    let mut report = String::new();
    report.push_str("build artifacts\n");
    for artifact in &artifacts {
        append_summary(&mut report, artifact);
    }
    let _ = writeln!(report, "total {} bytes", total_size(&artifacts));
    print!("{report}");
}
