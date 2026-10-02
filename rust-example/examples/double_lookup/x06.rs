use std::collections::{HashMap, HashSet, VecDeque};

struct RoadMap {
    links: HashMap<&'static str, Vec<&'static str>>,
}

impl RoadMap {
    fn from_roads(roads: &[(&'static str, &'static str)]) -> Self {
        let mut links: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
        for &(a, b) in roads {
            links.entry(a).or_default().push(b);
            links.entry(b).or_default().push(a);
        }
        for neighbours in links.values_mut() {
            neighbours.sort_unstable();
        }
        RoadMap { links }
    }

    fn hops(&self, start: &'static str, goal: &'static str) -> Option<usize> {
        let mut seen: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<(&str, usize)> = VecDeque::new();
        seen.insert(start);
        queue.push_back((start, 0));

        while let Some((town, dist)) = queue.pop_front() {
            if town == goal {
                return Some(dist);
            }
            let Some(next) = self.links.get(town) else { continue };
            for &n in next {
                if seen.insert(n) {
                    queue.push_back((n, dist + 1));
                }
            }
        }
        None
    }
}

fn main() {
    let roads = [
        ("Alder", "Birch"),
        ("Birch", "Cedar"),
        ("Cedar", "Dogwood"),
        ("Alder", "Elm"),
        ("Elm", "Dogwood"),
        ("Fir", "Gum"),
    ];
    let map = RoadMap::from_roads(&roads);

    let queries = [("Alder", "Dogwood"), ("Birch", "Elm"), ("Alder", "Gum"), ("Fir", "Gum")];
    for (from, to) in queries {
        match map.hops(from, to) {
            Some(h) => println!("{from} -> {to}: {h} hops"),
            None => println!("{from} -> {to}: unreachable"),
        }
    }
}
