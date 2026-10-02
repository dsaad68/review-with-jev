use std::fmt;

#[derive(Debug, PartialEq)]
enum Move {
    Forward(u32),
    Turn(char),
    Repeat(u32, Box<Move>),
}

#[derive(Debug)]
enum ParseError {
    Empty,
    UnknownOp(char),
    BadNumber(String),
    BadDirection(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty instruction"),
            ParseError::UnknownOp(c) => write!(f, "unknown op '{c}'"),
            ParseError::BadNumber(s) => write!(f, "bad number '{s}'"),
            ParseError::BadDirection(s) => write!(f, "bad direction '{s}'"),
        }
    }
}

fn number(s: &str) -> Result<u32, ParseError> {
    s.parse().map_err(|_| ParseError::BadNumber(s.to_string()))
}

fn parse_move(input: &str) -> Result<Move, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    let op = input.chars().next().unwrap();
    let rest = &input[op.len_utf8()..];
    match op {
        'F' => Ok(Move::Forward(number(rest)?)),
        'T' => match rest {
            "L" | "R" => Ok(Move::Turn(rest.chars().next().unwrap())),
            other => Err(ParseError::BadDirection(other.to_string())),
        },
        'x' => {
            let (count, inner) = rest.split_once(' ').ok_or(ParseError::Empty)?;
            Ok(Move::Repeat(number(count)?, Box::new(parse_move(inner)?)))
        }
        other => Err(ParseError::UnknownOp(other)),
    }
}

fn distance(m: &Move) -> u32 {
    match m {
        Move::Forward(n) => *n,
        Move::Turn(_) => 0,
        Move::Repeat(times, inner) => times * distance(inner),
    }
}

fn main() {
    let program = ["F10", "TL", "x3 F4", "TQ", "", "Z1", "Fabc", "x2 x2 F1"];
    let mut total = 0;
    for line in program {
        match parse_move(line) {
            Ok(m) => {
                total += distance(&m);
                println!("{line:<10} => {m:?}");
            }
            Err(e) => println!("{line:<10} => error: {e}"),
        }
    }
    println!("total distance {total}");
}
