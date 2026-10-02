use std::collections::HashMap;
use std::fmt;

struct Request<'a> {
    method: &'a str,
    path: &'a str,
    headers: HashMap<String, &'a str>,
    body: &'a [u8],
}

fn parse_request(raw: &[u8]) -> Request<'_> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("header terminator");
    let head = std::str::from_utf8(&raw[..split]).unwrap();
    let mut lines = head.split("\r\n");
    let mut start = lines.next().unwrap().split(' ');
    let method = start.next().unwrap();
    let path = start.next().unwrap();
    let headers = lines
        .filter_map(|l| l.split_once(": "))
        .map(|(k, v)| (k.to_ascii_lowercase(), v))
        .collect();
    Request { method, path, headers, body: &raw[split + 4..] }
}

fn declared_length(req: &Request) -> usize {
    req.headers["content-length"].parse().unwrap()
}

enum Response {
    Healthy,
    Created(usize),
    BadRequest { declared: usize, got: usize },
    NotFound,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Healthy => write!(f, "200 OK"),
            Response::Created(n) => write!(f, "201 Created ({n} bytes)"),
            Response::BadRequest { declared, got } => {
                write!(f, "400 Bad Request (declared {declared}, got {got})")
            }
            Response::NotFound => write!(f, "404 Not Found"),
        }
    }
}

fn route(req: &Request) -> Response {
    match (req.method, req.path) {
        ("GET", "/health") => Response::Healthy,
        ("POST", "/upload") => {
            let declared = declared_length(req);
            let got = req.body.len();
            if declared == got {
                Response::Created(got)
            } else {
                Response::BadRequest { declared, got }
            }
        }
        _ => Response::NotFound,
    }
}

fn main() {
    let samples: [&[u8]; 3] = [
        b"GET /health HTTP/1.1\r\nHost: example\r\n\r\n",
        b"POST /upload HTTP/1.1\r\nHost: example\r\nContent-Length: 5\r\n\r\nhello",
        b"POST /upload HTTP/1.1\r\nContent-Length: 9\r\n\r\nshort",
    ];
    for raw in samples {
        let req = parse_request(raw);
        println!("{} {} -> {}", req.method, req.path, route(&req));
    }
}
