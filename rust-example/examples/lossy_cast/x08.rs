use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

trait Encode {
    fn encode(&self, out: &mut Vec<u8>);
}

struct Entry {
    name: String,
    modified: i64,
    data: Vec<u8>,
}

impl Encode for Entry {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&(self.name.len() as u16).to_le_bytes());
        out.extend_from_slice(self.name.as_bytes());
        out.extend_from_slice(&(self.modified as u32).to_le_bytes());
        out.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.data);
    }
}

struct Archive<T: Encode> {
    items: Vec<T>,
}

impl<T: Encode> Encode for Archive<T> {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(b"PKA1");
        out.extend_from_slice(&(self.items.len() as u16).to_le_bytes());
        for item in &self.items {
            item.encode(out);
        }
    }
}

#[derive(Debug)]
enum ReadError {
    BadMagic,
    Truncated(&'static str),
    BadName,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::BadMagic => write!(f, "not an archive"),
            ReadError::Truncated(what) => write!(f, "archive ends inside {what}"),
            ReadError::BadName => write!(f, "entry name is not utf-8"),
        }
    }
}

impl Error for ReadError {}

struct Reader<'a> {
    buf: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize, what: &'static str) -> Result<&'a [u8], ReadError> {
        if self.buf.len() < n {
            return Err(ReadError::Truncated(what));
        }
        let (head, tail) = self.buf.split_at(n);
        self.buf = tail;
        Ok(head)
    }

    fn u16(&mut self, what: &'static str) -> Result<u16, ReadError> {
        let b = self.take(2, what)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&mut self, what: &'static str) -> Result<u32, ReadError> {
        let b = self.take(4, what)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

#[derive(Debug)]
struct EntryView<'a> {
    name: &'a str,
    modified: u32,
    data: &'a [u8],
}

fn parse(buf: &[u8]) -> Result<Vec<EntryView<'_>>, ReadError> {
    let mut r = Reader { buf };
    if r.take(4, "magic")? != b"PKA1" {
        return Err(ReadError::BadMagic);
    }
    let count = r.u16("entry count")?;
    let mut entries = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        let name_len = usize::from(r.u16("name length")?);
        let name = std::str::from_utf8(r.take(name_len, "name")?).map_err(|_| ReadError::BadName)?;
        let modified = r.u32("timestamp")?;
        let data_len = r.u32("data length")? as usize;
        let data = r.take(data_len, "data")?;
        entries.push(EntryView { name, modified, data });
    }
    Ok(entries)
}

fn write_archive<T: Encode>(path: &Path, archive: &T) -> std::io::Result<usize> {
    let mut buf = Vec::new();
    archive.encode(&mut buf);
    fs::write(path, &buf)?;
    Ok(buf.len())
}

fn main() -> Result<(), Box<dyn Error>> {
    let archive = Archive {
        items: vec![
            Entry {
                name: "notes/todo.txt".to_string(),
                modified: 1_712_000_000,
                data: b"buy milk\nfix fence\n".to_vec(),
            },
            Entry {
                name: "img/logo.bin".to_string(),
                modified: 1_712_086_400,
                data: (0u8..64).collect(),
            },
            Entry {
                name: "README".to_string(),
                modified: 1_700_000_000,
                data: b"sample archive".to_vec(),
            },
        ],
    };
    let path = std::env::temp_dir().join("pka_demo_archive.bin");
    let written = write_archive(&path, &archive)?;
    println!("wrote {written} bytes to {}", path.display());
    let bytes = fs::read(&path)?;
    for entry in parse(&bytes)? {
        println!(
            "{:<16} modified {} size {}",
            entry.name,
            entry.modified,
            entry.data.len()
        );
    }
    match parse(&bytes[..20]) {
        Ok(_) => println!("unexpected parse of truncated data"),
        Err(e) => println!("truncated archive: {e}"),
    }
    fs::remove_file(&path)?;
    Ok(())
}
