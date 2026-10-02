use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Word,
    Assign,
    Pipe,
    Quote,
}

#[derive(Debug, Clone, Copy)]
struct Token<'src> {
    kind: Kind,
    text: &'src str,
}

#[derive(Debug, PartialEq)]
enum ParseError {
    Empty,
    MissingTarget,
    MissingValue,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ParseError::Empty => "empty command",
            ParseError::MissingTarget => "nothing to assign to",
            ParseError::MissingValue => "assignment has no value",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for ParseError {}

struct Line<'src> {
    tokens: Vec<Token<'src>>,
}

impl<'src> Line<'src> {
    fn tokenize(src: &'src str) -> Self {
        let tokens = src
            .split_whitespace()
            .map(|text| {
                let kind = match text {
                    "=" => Kind::Assign,
                    "|" => Kind::Pipe,
                    t if t.starts_with('"') => Kind::Quote,
                    _ => Kind::Word,
                };
                Token { kind, text }
            })
            .collect();
        Line { tokens }
    }

    fn assign_at(&self) -> Option<usize> {
        let mut at = None;
        for (i, t) in self.tokens.iter().enumerate() {
            if t.kind == Kind::Assign {
                at = Some(i);
                break;
            }
        }
        at
    }

    fn stages(&self) -> Vec<&[Token<'src>]> {
        self.tokens.split(|t| t.kind == Kind::Pipe).collect()
    }
}

#[derive(Debug)]
enum Command<'src> {
    Set { name: &'src str, value: Vec<&'src str> },
    Run { stages: Vec<Vec<&'src str>> },
}

fn parse<'src>(line: &Line<'src>) -> Result<Command<'src>, ParseError> {
    if line.tokens.is_empty() {
        return Err(ParseError::Empty);
    }
    if let Some(at) = line.assign_at() {
        let name = match at.checked_sub(1).and_then(|p| line.tokens.get(p)) {
            Some(t) => t.text,
            None => return Err(ParseError::MissingTarget),
        };
        let value: Vec<&str> = line.tokens[at + 1..].iter().map(|t| t.text).collect();
        if value.is_empty() {
            return Err(ParseError::MissingValue);
        }
        return Ok(Command::Set { name, value });
    }
    let stages = line
        .stages()
        .into_iter()
        .map(|stage| stage.iter().map(|t| t.text).collect())
        .collect();
    Ok(Command::Run { stages })
}

fn main() {
    let inputs = [
        "greeting = \"hello world\"",
        "cat notes.txt | grep todo | wc -l",
        "= 5",
        "path =",
        "",
    ];
    for src in inputs {
        let line = Line::tokenize(src);
        match parse(&line) {
            Ok(Command::Set { name, value }) => println!("set {name} to {}", value.join(" ")),
            Ok(Command::Run { stages }) => println!("run {} stage(s): {:?}", stages.len(), stages),
            Err(e) => println!("{src:?}: {e}"),
        }
    }
}
