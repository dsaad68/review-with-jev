use std::fmt;

#[derive(Debug, PartialEq)]
enum Token {
    Number(u64),
    Ident(String),
    Op(char),
    LParen,
    RParen,
}

#[derive(Debug)]
struct LexError {
    position: usize,
    found: char,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected '{}' at {}", self.found, self.position)
    }
}

impl std::error::Error for LexError {}

fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let value = text.parse().map_err(|_| LexError { position: start, found: c })?;
            tokens.push(Token::Number(value));
        } else if c.is_alphabetic() {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(Token::Ident(chars[start..i].iter().collect()));
        } else {
            let tok = match c {
                '+' | '-' | '*' | '/' => Token::Op(c),
                '(' => Token::LParen,
                ')' => Token::RParen,
                other => return Err(LexError { position: i, found: other }),
            };
            tokens.push(tok);
            i += 1;
        }
    }
    Ok(tokens)
}

fn main() {
    for src in ["rate * (base_12 + 340)", "7 / x2 - 1", "4 % 2"] {
        match lex(src) {
            Ok(tokens) => println!("{src:?} -> {tokens:?}"),
            Err(e) => println!("{src:?} -> error: {e}"),
        }
    }
}
