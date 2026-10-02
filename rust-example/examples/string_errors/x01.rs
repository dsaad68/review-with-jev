use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum DurationError {
    Empty,
    Number(ParseIntError),
    Unit(char),
}

impl fmt::Display for DurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DurationError::Empty => write!(f, "empty duration"),
            DurationError::Number(e) => write!(f, "bad amount: {e}"),
            DurationError::Unit(c) => write!(f, "unknown unit {c:?}"),
        }
    }
}

fn parse_duration_secs(input: &str) -> Result<u64, DurationError> {
    let input = input.trim();
    let unit = input.chars().last().ok_or(DurationError::Empty)?;
    let amount: u64 = input[..input.len() - unit.len_utf8()].parse().map_err(DurationError::Number)?;
    match unit {
        's' => Ok(amount),
        'm' => Ok(amount * 60),
        'h' => Ok(amount * 3600),
        other => Err(DurationError::Unit(other)),
    }
}

fn main() {
    for raw in ["90s", "5m", "", "2d", "h"] {
        match parse_duration_secs(raw) {
            Ok(secs) => println!("{raw:?} = {secs}s"),
            Err(DurationError::Unit(c)) => println!("{raw:?}: unit {c} not supported yet"),
            Err(e) => println!("{raw:?}: {e}"),
        }
    }
}
