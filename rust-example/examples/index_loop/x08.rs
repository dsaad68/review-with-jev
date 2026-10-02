use std::collections::HashMap;
use std::fmt;

trait EditCost {
    fn substitute(&self, a: char, b: char) -> u32;
    fn insert(&self, c: char) -> u32;
    fn delete(&self, c: char) -> u32;
}

struct Uniform;

impl EditCost for Uniform {
    fn substitute(&self, a: char, b: char) -> u32 {
        if a == b {
            0
        } else {
            1
        }
    }
    fn insert(&self, _c: char) -> u32 {
        1
    }
    fn delete(&self, _c: char) -> u32 {
        1
    }
}

struct Keyboard {
    neighbours: HashMap<char, &'static str>,
}

impl Keyboard {
    fn qwerty() -> Self {
        let mut neighbours = HashMap::new();
        neighbours.insert('a', "qwsz");
        neighbours.insert('s', "awedxz");
        neighbours.insert('e', "wsdr");
        neighbours.insert('r', "edft");
        neighbours.insert('t', "rfgy");
        neighbours.insert('o', "iklp");
        neighbours.insert('i', "ujko");
        neighbours.insert('n', "bhjm");
        Keyboard { neighbours }
    }
}

impl EditCost for Keyboard {
    fn substitute(&self, a: char, b: char) -> u32 {
        if a == b {
            return 0;
        }
        match self.neighbours.get(&a) {
            Some(near) if near.contains(b) => 1,
            _ => 2,
        }
    }
    fn insert(&self, _c: char) -> u32 {
        2
    }
    fn delete(&self, _c: char) -> u32 {
        2
    }
}

struct Alignment<'a> {
    source: &'a str,
    target: &'a str,
    cost: u32,
}

impl fmt::Display for Alignment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {} = {}", self.source, self.target, self.cost)
    }
}

fn distance<C: EditCost>(model: &C, source: &str, target: &str) -> u32 {
    let a: Vec<char> = source.chars().collect();
    let b: Vec<char> = target.chars().collect();
    let mut table = vec![vec![0u32; b.len() + 1]; a.len() + 1];
    for i in 1..=a.len() {
        table[i][0] = table[i - 1][0] + model.delete(a[i - 1]);
    }
    for j in 1..=b.len() {
        table[0][j] = table[0][j - 1] + model.insert(b[j - 1]);
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let replace = table[i - 1][j - 1] + model.substitute(a[i - 1], b[j - 1]);
            let remove = table[i - 1][j] + model.delete(a[i - 1]);
            let add = table[i][j - 1] + model.insert(b[j - 1]);
            table[i][j] = replace.min(remove).min(add);
        }
    }
    table[a.len()][b.len()]
}

fn best_match<'a, C: EditCost>(
    model: &C,
    word: &'a str,
    dictionary: &[&'a str],
) -> Option<Alignment<'a>> {
    dictionary
        .iter()
        .map(|&candidate| Alignment {
            source: word,
            target: candidate,
            cost: distance(model, word, candidate),
        })
        .min_by_key(|al| al.cost)
}

fn main() {
    let dictionary = ["rate", "sate", "note", "tone", "stone", "rot", "riot"];
    let typos = ["rste", "nto", "stine", "roit"];
    let uniform = Uniform;
    let keyboard = Keyboard::qwerty();
    for typo in typos {
        if let Some(al) = best_match(&uniform, typo, &dictionary) {
            println!("uniform  {al}");
        }
        if let Some(al) = best_match(&keyboard, typo, &dictionary) {
            println!("keyboard {al}");
        }
    }
}
