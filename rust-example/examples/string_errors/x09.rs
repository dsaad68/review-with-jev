use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token<'a> {
    Num(i64),
    Ident(&'a str),
    Op(char),
    Open,
    Close,
}

#[derive(Debug, PartialEq)]
enum ErrorKind<'a> {
    UnexpectedChar(char),
    UnexpectedEnd,
    UnexpectedToken(Token<'a>),
    NumberTooLarge,
    UnknownVariable(&'a str),
    DivisionByZero,
    Overflow,
}

#[derive(Debug)]
struct ExprError<'a> {
    pos: usize,
    kind: ErrorKind<'a>,
}

impl fmt::Display for ExprError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "at {}: {:?}", self.pos, self.kind)
    }
}

impl std::error::Error for ExprError<'_> {}

fn tokenize(src: &str) -> Result<Vec<(usize, Token<'_>)>, ExprError<'_>> {
    let mut tokens = Vec::new();
    let mut chars = src.char_indices().peekable();
    while let Some(&(pos, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c.is_ascii_alphanumeric() {
            let mut end = pos;
            while let Some(&(i, d)) = chars.peek().filter(|(_, d)| d.is_ascii_alphanumeric() || *d == '_') {
                end = i + d.len_utf8();
                chars.next();
            }
            let text = &src[pos..end];
            let token = if c.is_ascii_digit() {
                Token::Num(text.parse().map_err(|_| ExprError { pos, kind: ErrorKind::NumberTooLarge })?)
            } else {
                Token::Ident(text)
            };
            tokens.push((pos, token));
        } else {
            let token = match c {
                '+' | '-' | '*' | '/' => Token::Op(c),
                '(' => Token::Open,
                ')' => Token::Close,
                other => return Err(ExprError { pos, kind: ErrorKind::UnexpectedChar(other) }),
            };
            tokens.push((pos, token));
            chars.next();
        }
    }
    Ok(tokens)
}

struct Evaluator<'a, 'v> {
    tokens: &'v [(usize, Token<'a>)],
    next: usize,
    end: usize,
    vars: &'v HashMap<&'v str, i64>,
}

impl<'a, 'v> Evaluator<'a, 'v> {
    fn peek(&self) -> Option<Token<'a>> {
        self.tokens.get(self.next).map(|&(_, t)| t)
    }

    fn pos(&self) -> usize {
        self.tokens.get(self.next).map_or(self.end, |&(p, _)| p)
    }

    fn fail<T>(&self, kind: ErrorKind<'a>) -> Result<T, ExprError<'a>> {
        Err(ExprError { pos: self.pos(), kind })
    }

    fn expr(&mut self) -> Result<i64, ExprError<'a>> {
        let mut value = self.term()?;
        while let Some(Token::Op(op @ ('+' | '-'))) = self.peek() {
            self.next += 1;
            let rhs = self.term()?;
            let combined = if op == '+' { value.checked_add(rhs) } else { value.checked_sub(rhs) };
            value = combined.map_or_else(|| self.fail(ErrorKind::Overflow), Ok)?;
        }
        Ok(value)
    }

    fn term(&mut self) -> Result<i64, ExprError<'a>> {
        let mut value = self.atom()?;
        while let Some(Token::Op(op @ ('*' | '/'))) = self.peek() {
            let at = self.pos();
            self.next += 1;
            let rhs = self.atom()?;
            value = match op {
                '/' if rhs == 0 => return Err(ExprError { pos: at, kind: ErrorKind::DivisionByZero }),
                '/' => value / rhs,
                _ => value.checked_mul(rhs).map_or_else(|| self.fail(ErrorKind::Overflow), Ok)?,
            };
        }
        Ok(value)
    }

    fn atom(&mut self) -> Result<i64, ExprError<'a>> {
        let Some(token) = self.peek() else { return self.fail(ErrorKind::UnexpectedEnd) };
        let at = self.pos();
        self.next += 1;
        match token {
            Token::Num(n) => Ok(n),
            Token::Ident(name) => {
                self.vars.get(name).copied().ok_or(ExprError { pos: at, kind: ErrorKind::UnknownVariable(name) })
            }
            Token::Open => {
                let inner = self.expr()?;
                match self.peek() {
                    Some(Token::Close) => {
                        self.next += 1;
                        Ok(inner)
                    }
                    Some(other) => self.fail(ErrorKind::UnexpectedToken(other)),
                    None => self.fail(ErrorKind::UnexpectedEnd),
                }
            }
            other => Err(ExprError { pos: at, kind: ErrorKind::UnexpectedToken(other) }),
        }
    }
}

fn evaluate<'a>(src: &'a str, vars: &HashMap<&str, i64>) -> Result<i64, ExprError<'a>> {
    let tokens = tokenize(src)?;
    let mut eval = Evaluator { tokens: &tokens, next: 0, end: src.len(), vars };
    let value = eval.expr()?;
    match eval.peek() {
        None => Ok(value),
        Some(extra) => eval.fail(ErrorKind::UnexpectedToken(extra)),
    }
}

fn main() {
    let vars = HashMap::from([("width", 12), ("height", 5), ("margin", 2)]);
    let inputs = ["width * height", "(width - margin * 2) * (height - margin)", "width / (height - 5)"];
    let more = ["depth + 1", "width * (height + 1", "3 $ 4", "99999999999999999999"];
    for src in inputs.into_iter().chain(more) {
        match evaluate(src, &vars) {
            Ok(v) => println!("{src} = {v}"),
            Err(ExprError { kind: ErrorKind::UnknownVariable(name), .. }) => {
                println!("{src}: define {name} first")
            }
            Err(e) => println!("{src}: {e}"),
        }
    }
}
