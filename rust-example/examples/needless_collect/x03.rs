use std::collections::HashSet;

fn has_banned_host(urls: &[&str], banned: &HashSet<&str>) -> bool {
    let hosts: Vec<&str> = urls
        .iter()
        .filter_map(|u| u.strip_prefix("https://"))
        .map(|rest| rest.split('/').next().unwrap_or(rest))
        .filter(|h| banned.contains(h))
        .collect();
    !hosts.is_empty()
}

fn main() {
    let banned: HashSet<&str> = ["tracker.example", "ads.example"].into_iter().collect();
    let page = [
        "https://cdn.example/app.js",
        "https://ads.example/banner",
        "https://fonts.example/inter.css",
    ];
    if has_banned_host(&page, &banned) {
        println!("page blocked");
    } else {
        println!("page allowed");
    }
}
