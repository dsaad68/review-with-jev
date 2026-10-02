use std::collections::BTreeMap;
use std::fmt::{self, Write as _};
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::sync::mpsc;
use std::thread;

struct Signup<'a> {
    name: &'a str,
    email: &'a str,
    age: u8,
}

#[derive(Debug)]
enum Problem {
    MissingField(&'static str),
    BadAge(ParseIntError),
    TooYoung(u8),
    BadEmail(String),
}

impl Problem {
    fn kind(&self) -> &'static str {
        match self {
            Problem::MissingField(_) => "missing field",
            Problem::BadAge(_) => "bad age",
            Problem::TooYoung(_) => "too young",
            Problem::BadEmail(_) => "bad email",
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::MissingField(name) => write!(f, "missing {name}"),
            Problem::BadAge(e) => write!(f, "age is not a number ({e})"),
            Problem::TooYoung(age) => write!(f, "age {age} is below the minimum"),
            Problem::BadEmail(email) => write!(f, "{email:?} is not an email address"),
        }
    }
}

trait Rule: Send {
    fn check(&self, signup: &Signup<'_>) -> Result<(), Problem>;
}

struct MinimumAge(u8);
struct EmailShape;

impl Rule for MinimumAge {
    fn check(&self, signup: &Signup<'_>) -> Result<(), Problem> {
        if signup.age < self.0 {
            return Err(Problem::TooYoung(signup.age));
        }
        Ok(())
    }
}

impl Rule for EmailShape {
    fn check(&self, signup: &Signup<'_>) -> Result<(), Problem> {
        match signup.email.split_once('@') {
            Some((user, domain)) if !user.is_empty() && domain.contains('.') => Ok(()),
            _ => Err(Problem::BadEmail(signup.email.to_string())),
        }
    }
}

fn parse_signup(line: &str) -> Result<Signup<'_>, Problem> {
    let mut fields = line.split(',').map(str::trim);
    let name = fields.next().filter(|s| !s.is_empty()).ok_or(Problem::MissingField("name"))?;
    let email = fields.next().ok_or(Problem::MissingField("email"))?;
    let age_raw = fields.next().ok_or(Problem::MissingField("age"))?;
    let age = age_raw.parse().map_err(Problem::BadAge)?;
    Ok(Signup { name, email, age })
}

fn validate(line: &str, rules: &[Box<dyn Rule>]) -> Result<String, Problem> {
    let signup = parse_signup(line)?;
    for rule in rules {
        rule.check(&signup)?;
    }
    Ok(format!("{} <{}>", signup.name, signup.email))
}

enum Job {
    Line(usize, String),
    Stop,
}

fn main() -> io::Result<()> {
    let input = "\
ada, ada@example.org, 36
bob, bob-at-example, 41
, ghost@example.org, 30
cy, cy@example.net, 15
dee, dee@example.com, forty
eve, eve@example.com";

    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let (result_tx, result_rx) = mpsc::channel();

    let worker = thread::spawn(move || {
        let rules: Vec<Box<dyn Rule>> = vec![Box::new(MinimumAge(18)), Box::new(EmailShape)];
        for job in job_rx {
            match job {
                Job::Line(n, line) => {
                    if result_tx.send((n, validate(&line, &rules))).is_err() {
                        break;
                    }
                }
                Job::Stop => break,
            }
        }
    });

    for (n, line) in input.lines().enumerate() {
        job_tx
            .send(Job::Line(n + 1, line.to_string()))
            .expect("validation worker exited early");
    }
    let _ = job_tx.send(Job::Stop);
    worker.join().expect("validation worker panicked");

    let mut accepted = Vec::new();
    let mut by_kind: BTreeMap<&'static str, usize> = BTreeMap::new();
    for (n, outcome) in result_rx {
        match outcome {
            Ok(who) => accepted.push(who),
            Err(problem) => {
                eprintln!("line {n}: {problem}");
                *by_kind.entry(problem.kind()).or_insert(0) += 1;
            }
        }
    }

    let mut summary = String::new();
    let _ = writeln!(summary, "accepted {}", accepted.len());
    for (kind, count) in &by_kind {
        let _ = writeln!(summary, "rejected ({kind}) {count}");
    }
    for who in &accepted {
        let _ = writeln!(summary, "  {who}");
    }

    let path = std::env::temp_dir().join("signup_validation_summary.txt");
    fs::write(&path, &summary)?;
    print!("{}", fs::read_to_string(&path)?);
    let _ = fs::remove_file(&path);
    Ok(())
}
