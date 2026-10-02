use std::fmt;

#[derive(Debug)]
enum Op { Push(i64), Pop, Add, Mul, Dup }

#[derive(Debug)]
enum ParseError {
    UnknownOp(String),
    BadNumber(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnknownOp(op) => write!(f, "unknown operation `{op}`"),
            ParseError::BadNumber(n) => write!(f, "bad number `{n}`"),
        }
    }
}

#[derive(Debug)]
enum ExecError { Underflow, Overflow }

fn parse(token: &str) -> Result<Op, ParseError> {
    match token {
        "pop" => Ok(Op::Pop),
        "add" => Ok(Op::Add),
        "mul" => Ok(Op::Mul),
        "dup" => Ok(Op::Dup),
        other if other.chars().all(|c| c.is_ascii_alphabetic()) => {
            Err(ParseError::UnknownOp(other.to_string()))
        }
        other => other
            .parse::<i64>()
            .map(Op::Push)
            .map_err(|_| ParseError::BadNumber(other.to_string())),
    }
}

fn run(program: &[Op]) -> Result<Vec<i64>, ExecError> {
    let mut stack = Vec::new();
    for op in program {
        match op {
            Op::Push(n) => stack.push(*n),
            Op::Pop => {
                stack.pop().ok_or(ExecError::Underflow)?;
            }
            Op::Dup => {
                let top = *stack.last().ok_or(ExecError::Underflow)?;
                stack.push(top);
            }
            Op::Add | Op::Mul => {
                let b = stack.pop().ok_or(ExecError::Underflow)?;
                let a = stack.pop().ok_or(ExecError::Underflow)?;
                let result = match op {
                    Op::Add => a.checked_add(b),
                    _ => a.checked_mul(b),
                };
                stack.push(result.ok_or(ExecError::Overflow)?);
            }
        }
    }
    Ok(stack)
}

fn main() {
    for source in ["3 4 add dup mul", "1 pop pop", "2 x7 add", "5 swap"] {
        let parsed: Result<Vec<Op>, ParseError> = source.split_whitespace().map(parse).collect();
        match parsed {
            Ok(program) => match run(&program) {
                Ok(stack) => println!("{source:?} => {stack:?}"),
                Err(e) => println!("{source:?} failed: {e:?}"),
            },
            Err(e) => println!("{source:?} rejected: {e}"),
        }
    }
}
