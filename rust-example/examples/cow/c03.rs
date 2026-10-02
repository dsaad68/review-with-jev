use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Method { Get, Post, Delete }

#[derive(Debug)]
struct Request<'a> {
    method: Method,
    path: &'a str,
}

#[derive(Debug, Default)]
struct Params<'a> {
    pairs: Vec<(&'a str, &'a str)>,
}

impl<'a> Params<'a> {
    fn get(&self, key: &str) -> Option<&'a str> {
        self.pairs.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
}

struct Response { status: u16, body: String }

trait Handler: Send + Sync {
    fn call(&self, req: &Request<'_>, params: &Params<'_>) -> Response;
}

impl<F> Handler for F
where
    F: Fn(&Request<'_>, &Params<'_>) -> Response + Send + Sync,
{
    fn call(&self, req: &Request<'_>, params: &Params<'_>) -> Response {
        self(req, params)
    }
}

#[derive(Debug)]
enum Segment {
    Literal(String),
    Capture(String),
    Rest(String),
}

struct Route {
    method: Method,
    pattern: String,
    segments: Vec<Segment>,
    handler: Box<dyn Handler>,
}

fn compile(pattern: &str) -> Vec<Segment> {
    pattern
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| match s.as_bytes()[0] {
            b':' => Segment::Capture(s[1..].to_string()),
            b'*' => Segment::Rest(s[1..].to_string()),
            _ => Segment::Literal(s.to_string()),
        })
        .collect()
}

fn normalize_path(path: &str) -> String {
    let needs_work = path.contains("//")
        || path.contains("/./")
        || path.contains("/../")
        || (path.len() > 1 && path.ends_with('/'));
    if !needs_work {
        return path.to_owned();
    }
    let mut parts: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => continue,
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    format!("/{}", parts.join("/"))
}

fn match_segments<'p, 's>(segments: &'s [Segment], path: &'p str) -> Option<Params<'p>>
where
    's: 'p,
{
    let mut params = Params::default();
    let mut rest = path.trim_start_matches('/');
    for seg in segments {
        if let Segment::Rest(name) = seg {
            params.pairs.push((name.as_str(), rest));
            return Some(params);
        }
        let (head, tail) = rest.split_once('/').unwrap_or((rest, ""));
        if head.is_empty() {
            return None;
        }
        match seg {
            Segment::Literal(lit) if lit == head => {}
            Segment::Capture(name) => params.pairs.push((name.as_str(), head)),
            _ => return None,
        }
        rest = tail;
    }
    if rest.is_empty() { Some(params) } else { None }
}

#[derive(Default)]
struct Router {
    routes: Vec<Route>,
    misses: HashMap<Method, usize>,
}

impl Router {
    fn add<H: Handler + 'static>(&mut self, method: Method, pattern: &str, handler: H) -> &mut Self {
        self.routes.push(Route {
            method,
            pattern: pattern.to_string(),
            segments: compile(pattern),
            handler: Box::new(handler),
        });
        self
    }

    fn dispatch(&mut self, req: &Request<'_>) -> Response {
        let clean = normalize_path(req.path);
        let hit = self
            .routes
            .iter()
            .filter(|r| r.method == req.method)
            .find_map(|r| match_segments(&r.segments, &clean).map(|p| (r, p)));
        match hit {
            Some((route, params)) => {
                let mut resp = route.handler.call(req, &params);
                resp.body.push_str(&format!(" [via {}]", route.pattern));
                resp
            }
            None => {
                *self.misses.entry(req.method).or_insert(0) += 1;
                Response { status: 404, body: format!("no route for {}", clean) }
            }
        }
    }
}

fn main() {
    let mut router = Router::default();
    router
        .add(Method::Get, "/users/:id", |_: &Request<'_>, p: &Params<'_>| Response {
            status: 200,
            body: format!("user {}", p.get("id").unwrap_or("?")),
        })
        .add(Method::Post, "/users", |_: &Request<'_>, _: &Params<'_>| Response {
            status: 201,
            body: "created".into(),
        })
        .add(Method::Get, "/static/*file", |r: &Request<'_>, p: &Params<'_>| Response {
            status: 200,
            body: format!("{:?} file {}", r.method, p.get("file").unwrap_or("")),
        });

    let requests = [
        Request { method: Method::Get, path: "/users/42" },
        Request { method: Method::Get, path: "/users//42/" },
        Request { method: Method::Post, path: "/users" },
        Request { method: Method::Delete, path: "/users/7" },
        Request { method: Method::Get, path: "/static/css/./site.css" },
        Request { method: Method::Get, path: "/admin/../users/9" },
        Request { method: Method::Post, path: "/users/1" },
    ];
    for req in &requests {
        let resp = router.dispatch(req);
        println!("{:<6} {:<28} -> {} {}", format!("{:?}", req.method), req.path, resp.status, resp.body);
    }
    println!("misses: {:?}", router.misses);
}
