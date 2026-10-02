use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Reading {
    sensor: u16,
    value: i32,
}

fn dedup_preserving<T: Clone + Eq + Hash>(items: &[T]) -> Vec<T> {
    let mut seen = HashSet::with_capacity(items.len());
    if items.iter().all(|x| seen.insert(x)) {
        return items.to_vec();
    }
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|x| seen.insert(*x))
        .cloned()
        .collect()
}

fn summarize(readings: &[Reading]) -> (usize, i32, i32) {
    let unique = dedup_preserving(readings);
    let min = unique.iter().map(|r| r.value).min().unwrap_or(0);
    let max = unique.iter().map(|r| r.value).max().unwrap_or(0);
    (unique.len(), min, max)
}

fn sensors(readings: &[Reading]) -> Vec<u16> {
    let ids: Vec<u16> = readings.iter().map(|r| r.sensor).collect();
    dedup_preserving(&ids)
}

fn main() {
    let clean = vec![
        Reading { sensor: 1, value: 20 },
        Reading { sensor: 2, value: 22 },
        Reading { sensor: 3, value: 19 },
    ];
    let noisy = vec![
        Reading { sensor: 1, value: 20 },
        Reading { sensor: 1, value: 20 },
        Reading { sensor: 2, value: 25 },
        Reading { sensor: 2, value: 25 },
        Reading { sensor: 4, value: -3 },
    ];
    println!("clean: {:?}", summarize(&clean));
    println!("noisy: {:?}", summarize(&noisy));
    println!("sensors: {:?}", sensors(&noisy));
    let words = ["a", "b", "a", "c"];
    println!("words: {:?}", dedup_preserving(&words));
}
