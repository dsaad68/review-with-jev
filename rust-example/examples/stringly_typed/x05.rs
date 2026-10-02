use std::collections::HashMap;

struct Request {
    method: String,
    path: String,
    body: String,
}

fn parse_request(line: &str) -> Option<Request> {
    let mut parts = line.splitn(3, ' ');
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let body = parts.next().unwrap_or("").to_string();
    Some(Request { method, path, body })
}

struct Store {
    notes: HashMap<String, String>,
}

impl Store {
    fn handle(&mut self, req: &Request) -> u16 {
        if req.method == "GET" {
            match self.notes.get(&req.path) {
                Some(text) => {
                    println!("  body: {text}");
                    200
                }
                None => 404,
            }
        } else if req.method == "PUT" {
            match self.notes.insert(req.path.clone(), req.body.clone()) {
                None => 201,
                Some(_) => 204,
            }
        } else if req.method == "DELETE" {
            match self.notes.remove(&req.path) {
                Some(_) => 204,
                None => 404,
            }
        } else {
            405
        }
    }
}

fn main() {
    let mut store = Store { notes: HashMap::new() };
    let script = [
        "PUT /todo buy milk",
        "GET /todo",
        "PUT /todo buy oat milk",
        "PATCH /todo x",
        "get /todo",
        "DELETE /todo",
        "GET /todo",
        "",
    ];
    for line in script {
        match parse_request(line) {
            Some(req) => {
                let status = store.handle(&req);
                println!("{} {} -> {}", req.method, req.path, status);
            }
            None => println!("malformed request line {line:?}"),
        }
    }
}
