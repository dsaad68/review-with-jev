#[derive(Debug)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
    timeout_secs: u64,
    tls: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 4,
            timeout_secs: 30,
            tls: false,
        }
    }
}

fn load_config(text: &str) -> ServerConfig {
    let mut cfg = ServerConfig::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "host" => cfg.host = value.to_string(),
            "port" => cfg.port = value.parse().unwrap_or_default(),
            "workers" => {
                if let Ok(n) = value.parse() {
                    cfg.workers = n;
                }
            }
            "timeout_secs" => cfg.timeout_secs = value.parse().unwrap_or(30),
            "tls" => cfg.tls = value == "on",
            _ => {}
        }
    }
    cfg
}

fn bind_address(cfg: &ServerConfig) -> String {
    let scheme = if cfg.tls { "https" } else { "http" };
    format!("{scheme}://{}:{}", cfg.host, cfg.port)
}

fn main() {
    let text = "\
# production settings
host = 0.0.0.0
port = 80800
workers = four
timeout_secs = 2m
tls = on
max_body 10mb
";
    let cfg = load_config(text);
    println!("{cfg:?}");
    println!("serving on {}", bind_address(&cfg));
    println!(
        "{} workers, requests time out after {}s",
        cfg.workers, cfg.timeout_secs
    );
}
