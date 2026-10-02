use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Image,
    Audio,
    Video,
    Other,
}

const IMAGE: [&str; 4] = ["png", "jpg", "jpeg", "gif"];
const AUDIO: [&str; 3] = ["mp3", "flac", "ogg"];
const VIDEO: [&str; 3] = ["mp4", "mkv", "webm"];

fn classify(path: &PathBuf) -> Kind {
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => ext,
        None => return Kind::Other,
    };
    if IMAGE.iter().any(|x| x.eq_ignore_ascii_case(ext)) {
        Kind::Image
    } else if AUDIO.iter().any(|x| x.eq_ignore_ascii_case(ext)) {
        Kind::Audio
    } else if VIDEO.iter().any(|x| x.eq_ignore_ascii_case(ext)) {
        Kind::Video
    } else {
        Kind::Other
    }
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('.'))
}

fn tally(paths: &[PathBuf]) -> HashMap<Kind, usize> {
    let mut counts = HashMap::new();
    for path in paths {
        if is_hidden(path) {
            continue;
        }
        *counts.entry(classify(path)).or_insert(0) += 1;
    }
    counts
}

fn main() {
    let library = vec![
        PathBuf::from("/media/photos/beach.JPG"),
        PathBuf::from("/media/photos/.thumbs.png"),
        PathBuf::from("/media/music/track01.flac"),
        PathBuf::from("/media/music/track02.mp3"),
        PathBuf::from("/media/films/holiday.mkv"),
        PathBuf::from("/media/notes/todo"),
    ];

    for path in &library {
        println!("{:<32} {:?}", path.display(), classify(path));
    }

    let counts = tally(&library);
    for kind in [Kind::Image, Kind::Audio, Kind::Video, Kind::Other] {
        println!("{kind:?}: {}", counts.get(&kind).copied().unwrap_or(0));
    }
}
