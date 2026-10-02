use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Debug, PartialEq)]
enum VersionErrorKind {
    WrongPartCount(usize),
    NotNumeric { position: usize },
    LeadingZero { position: usize },
}

#[derive(Debug)]
struct VersionError {
    input: String,
    kind: VersionErrorKind,
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            VersionErrorKind::WrongPartCount(n) => write!(f, "{:?} has {n} parts, expected 3", self.input),
            VersionErrorKind::NotNumeric { position } => write!(f, "{:?}: part {position} is not a number", self.input),
            VersionErrorKind::LeadingZero { position } => write!(f, "{:?}: part {position} has a leading zero", self.input),
        }
    }
}

impl Error for VersionError {}

fn parse_version(input: &str) -> Result<Version, VersionError> {
    let fail = |kind| VersionError { input: input.to_string(), kind };
    let parts: Vec<&str> = input.trim().trim_start_matches('v').split('.').collect();
    if parts.len() != 3 {
        return Err(fail(VersionErrorKind::WrongPartCount(parts.len())));
    }
    let mut numbers = [0u32; 3];
    for (position, (part, slot)) in parts.iter().zip(numbers.iter_mut()).enumerate() {
        if part.len() > 1 && part.starts_with('0') {
            return Err(fail(VersionErrorKind::LeadingZero { position }));
        }
        *slot = part.parse().map_err(|_| fail(VersionErrorKind::NotNumeric { position }))?;
    }
    let [major, minor, patch] = numbers;
    Ok(Version { major, minor, patch })
}

fn main() {
    let installed = Version { major: 1, minor: 4, patch: 2 };
    for candidate in ["1.4.10", "v2.0.0", "1.4", "1.04.0", "1.x.0", "1.3.9"] {
        match parse_version(candidate) {
            Ok(v) => match v.cmp(&installed) {
                Ordering::Greater => println!("{candidate}: upgrade available"),
                Ordering::Equal => println!("{candidate}: up to date"),
                Ordering::Less => println!("{candidate}: older than installed"),
            },
            Err(e) if matches!(e.kind, VersionErrorKind::LeadingZero { .. }) => {
                println!("{candidate}: normalise first ({e})")
            }
            Err(e) => println!("{candidate}: rejected ({e})"),
        }
    }
}
