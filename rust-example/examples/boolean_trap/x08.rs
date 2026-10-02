use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Reply {
    Body(String),
    Redirect(String),
    Busy,
}

struct Server {
    routes: HashMap<String, Vec<Reply>>,
}

impl Server {
    fn respond(&self, path: &str, attempt: usize) -> Option<&Reply> {
        let replies = self.routes.get(path)?;
        replies.get(attempt).or_else(|| replies.last())
    }
}

#[derive(Debug, Clone, Copy)]
struct RetryPolicy {
    attempts: usize,
    backoff_ms: u64,
}

#[derive(Debug)]
enum FetchError {
    NotFound(String),
    RedirectRefused(String),
    TooManyHops(usize),
    GaveUp { path: String, attempts: usize },
}

struct Client<'s> {
    server: &'s Server,
    follow_redirects: bool,
    max_hops: usize,
    retry: RetryPolicy,
}

struct ClientBuilder<'s> {
    client: Client<'s>,
}

impl<'s> ClientBuilder<'s> {
    fn new(server: &'s Server) -> Self {
        ClientBuilder {
            client: Client {
                server,
                follow_redirects: false,
                max_hops: 3,
                retry: RetryPolicy { attempts: 1, backoff_ms: 0 },
            },
        }
    }

    fn follow_redirects(mut self, enabled: bool) -> Self {
        self.client.follow_redirects = enabled;
        self
    }

    fn max_hops(mut self, hops: usize) -> Self {
        self.client.max_hops = hops;
        self
    }

    fn retry(mut self, policy: RetryPolicy) -> Self {
        self.client.retry = policy;
        self
    }

    fn build(self) -> Client<'s> {
        self.client
    }
}

impl<'s> Client<'s> {
    fn attempt(&self, path: &str) -> Result<&'s Reply, FetchError> {
        for attempt in 0..self.retry.attempts {
            match self.server.respond(path, attempt) {
                None => return Err(FetchError::NotFound(path.to_string())),
                Some(Reply::Busy) => {
                    thread::sleep(Duration::from_millis(self.retry.backoff_ms << attempt));
                }
                Some(reply) => return Ok(reply),
            }
        }
        Err(FetchError::GaveUp { path: path.to_string(), attempts: self.retry.attempts })
    }

    fn fetch(&self, path: &str) -> Result<&'s str, FetchError> {
        let mut current = path;
        for _ in 0..=self.max_hops {
            match self.attempt(current)? {
                Reply::Body(body) => return Ok(body),
                Reply::Redirect(next) if self.follow_redirects => current = next,
                Reply::Redirect(next) => return Err(FetchError::RedirectRefused(next.to_string())),
                Reply::Busy => unreachable!("busy replies are retried"),
            }
        }
        Err(FetchError::TooManyHops(self.max_hops))
    }
}

fn main() {
    let mut routes = HashMap::new();
    routes.insert("/home".to_string(), vec![Reply::Body("welcome".to_string())]);
    routes.insert("/old".to_string(), vec![Reply::Redirect("/home".to_string())]);
    routes.insert("/loop".to_string(), vec![Reply::Redirect("/loop".to_string())]);
    routes.insert(
        "/flaky".to_string(),
        vec![Reply::Busy, Reply::Busy, Reply::Body("finally".to_string())],
    );
    routes.insert("/down".to_string(), vec![Reply::Busy]);
    let server = Server { routes };

    let patient = ClientBuilder::new(&server)
        .follow_redirects(true)
        .max_hops(4)
        .retry(RetryPolicy { attempts: 4, backoff_ms: 1 })
        .build();
    let strict = ClientBuilder::new(&server).build();

    let paths = ["/home", "/old", "/loop", "/flaky", "/down", "/missing"];
    let (tx, rx) = mpsc::channel();
    thread::scope(|scope| {
        for (label, client) in [("patient", &patient), ("strict", &strict)] {
            for path in paths {
                let tx = tx.clone();
                scope.spawn(move || {
                    let outcome = client.fetch(path);
                    let _ = tx.send((label, path, outcome));
                });
            }
        }
    });
    drop(tx);

    let mut outcomes: Vec<_> = rx.into_iter().collect();
    outcomes.sort_by_key(|(label, path, _)| (*label, *path));
    for (label, path, outcome) in outcomes {
        match outcome {
            Ok(body) => println!("{label:<8} {path:<9} -> {body}"),
            Err(e) => println!("{label:<8} {path:<9} !! {e:?}"),
        }
    }
}
