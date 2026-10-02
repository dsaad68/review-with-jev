struct MailingList {
    subscribers: Vec<String>,
}

impl MailingList {
    fn from_raw(mut addresses: Vec<String>) -> Self {
        for address in addresses.iter_mut() {
            address.make_ascii_lowercase();
        }
        addresses.sort();
        addresses.dedup();
        MailingList { subscribers: addresses }
    }

    fn subscribe(&mut self, address: String) {
        if let Err(pos) = self.subscribers.binary_search(&address) {
            self.subscribers.insert(pos, address);
        }
    }
}

fn drop_domain(list: &mut Vec<String>, domain: &str) {
    let suffix = format!("@{domain}");
    list.retain(|a| !a.ends_with(&suffix));
}

fn count_matching<S: AsRef<str>>(items: &[S], needle: &str) -> usize {
    items.iter().filter(|s| s.as_ref().contains(needle)).count()
}

fn longest(list: &[String]) -> Option<&str> {
    list.iter().map(String::as_str).max_by_key(|a| a.len())
}

fn main() {
    let raw = vec![
        String::from("Alice@Example.com"),
        String::from("bob@spam.biz"),
        String::from("alice@example.com"),
        String::from("carol.long.name@example.org"),
        String::from("dave@spam.biz"),
    ];

    let mut list = MailingList::from_raw(raw);
    list.subscribe(String::from("erin@example.com"));
    list.subscribe(String::from("bob@spam.biz"));
    println!("before: {}", list.subscribers.join(", "));

    drop_domain(&mut list.subscribers, "spam.biz");
    println!("after: {}", list.subscribers.join(", "));

    println!("example.com: {}", count_matching(&list.subscribers, "example.com"));
    println!("keywords hit: {}", count_matching(&["example", "org"], "org"));

    if let Some(addr) = longest(&list.subscribers) {
        println!("longest address: {addr}");
    }
}
