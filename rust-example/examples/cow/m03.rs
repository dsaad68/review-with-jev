use std::path::Path;

struct Workspace {
    root: String,
}

fn normalize(path: &str) -> String {
    if !path.contains("//") && !path.contains("/./") && !path.ends_with('/') {
        return path.to_owned();
    }
    let mut parts: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => continue,
            s => parts.push(s),
        }
    }
    let joined = parts.join("/");
    if path.starts_with('/') {
        format!("/{}", joined)
    } else {
        joined
    }
}

impl Workspace {
    fn new(root: &str) -> Self {
        Workspace { root: normalize(root) }
    }

    fn absolute(&self, rel: &str) -> String {
        let cleaned = normalize(rel);
        if cleaned.starts_with('/') {
            cleaned
        } else {
            format!("{}/{}", self.root, cleaned)
        }
    }

    fn extension_of(&self, rel: &str) -> Option<String> {
        let full = self.absolute(rel);
        Path::new(&full)
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
    }
}

fn main() {
    let ws = Workspace::new("/home/user/project/");
    let inputs = ["src/main.rs", "src//lib.rs", "./docs/./guide.md", "/etc/hosts"];
    for p in inputs.iter() {
        println!("{:<22} -> {}", p, ws.absolute(p));
    }
    for p in inputs.iter() {
        println!("{} ext {:?}", normalize(p), ws.extension_of(p));
    }
    let total: usize = inputs.iter().map(|p| normalize(p).len()).sum();
    println!("total normalized length: {}", total);
}
