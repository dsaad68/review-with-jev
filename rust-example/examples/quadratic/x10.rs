use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

const RESERVED: [&str; 4] = ["core", "std", "alloc", "proc_macro"];

#[derive(Debug)]
enum BuildError<'a> {
    Reserved(&'a str),
    MissingDependency { package: &'a str, needs: &'a str },
    Cycle(Vec<&'a str>),
}

struct Graph<N> {
    deps: HashMap<N, Vec<N>>,
}

impl<N: Copy + Eq + Hash + Ord + Debug> Graph<N> {
    fn new() -> Self {
        Graph { deps: HashMap::new() }
    }

    fn add(&mut self, node: N, needs: &[N]) {
        self.deps.entry(node).or_default().extend_from_slice(needs);
    }

    fn missing(&self) -> Option<(N, N)> {
        self.deps
            .iter()
            .flat_map(|(n, ds)| ds.iter().map(move |d| (*n, *d)))
            .find(|(_, d)| !self.deps.contains_key(d))
    }

    fn order(&self) -> Result<Vec<N>, Vec<N>> {
        let mut indegree: HashMap<N, usize> = self.deps.keys().map(|n| (*n, 0)).collect();
        let mut dependents: HashMap<N, Vec<N>> = HashMap::new();
        for (node, ds) in &self.deps {
            for d in ds {
                *indegree.entry(*node).or_insert(0) += 1;
                dependents.entry(*d).or_default().push(*node);
            }
        }
        let mut ready: BTreeSet<N> = indegree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(n, _)| *n)
            .collect();
        let mut out = Vec::with_capacity(indegree.len());
        while let Some(n) = ready.pop_first() {
            out.push(n);
            if let Some(users) = dependents.get(&n) {
                for u in users {
                    if let Some(deg) = indegree.get_mut(u) {
                        *deg -= 1;
                        if *deg == 0 {
                            ready.insert(*u);
                        }
                    }
                }
            }
        }
        if out.len() == indegree.len() {
            Ok(out)
        } else {
            let placed: HashSet<N> = out.iter().copied().collect();
            let mut stuck: Vec<N> = indegree.keys().copied().filter(|n| !placed.contains(n)).collect();
            stuck.sort();
            Err(stuck)
        }
    }

    fn chain_to(&self, from: N, target: N) -> Option<Vec<N>> {
        let mut parent: HashMap<N, N> = HashMap::new();
        let mut stack = vec![from];
        let mut seen = HashSet::from([from]);
        while let Some(n) = stack.pop() {
            if n == target {
                let mut path = vec![n];
                let mut cur = n;
                while let Some(p) = parent.get(&cur) {
                    path.push(*p);
                    cur = *p;
                }
                path.reverse();
                return Some(path);
            }
            for d in self.deps.get(&n).into_iter().flatten() {
                if seen.insert(*d) {
                    parent.insert(*d, n);
                    stack.push(*d);
                }
            }
        }
        None
    }
}

fn plan<'a>(manifest: &[(&'a str, &[&'a str])]) -> Result<(Vec<&'a str>, Graph<&'a str>), BuildError<'a>> {
    let mut g = Graph::new();
    for (name, needs) in manifest {
        if RESERVED.contains(name) {
            return Err(BuildError::Reserved(name));
        }
        g.add(*name, needs);
    }
    if let Some((package, needs)) = g.missing() {
        return Err(BuildError::MissingDependency { package, needs });
    }
    let order = g.order().map_err(BuildError::Cycle)?;
    Ok((order, g))
}

fn report(label: &str, manifest: &[(&str, &[&str])]) {
    println!("-- {}", label);
    match plan(manifest) {
        Ok((order, g)) => {
            for (i, pkg) in order.iter().enumerate() {
                println!("{:>2}. {}", i + 1, pkg);
            }
            if let (Some(first), Some(last)) = (order.first(), order.last()) {
                match g.chain_to(*last, *first) {
                    Some(chain) => println!("chain: {}", chain.join(" -> ")),
                    None => println!("{} does not depend on {}", last, first),
                }
            }
        }
        Err(e) => println!("error: {:?}", e),
    }
}

fn main() {
    let good: Vec<(&str, &[&str])> = vec![
        ("app", &["http", "db", "log"]),
        ("http", &["tls", "log", "bytes"]),
        ("db", &["pool", "log"]),
        ("pool", &["log"]),
        ("tls", &["bytes"]),
        ("bytes", &[]),
        ("log", &[]),
    ];
    let cyclic: Vec<(&str, &[&str])> = vec![("a", &["b"]), ("b", &["c"]), ("c", &["a"]), ("d", &[])];
    let missing: Vec<(&str, &[&str])> = vec![("web", &["json"]), ("cli", &[])];
    let reserved: Vec<(&str, &[&str])> = vec![("mine", &[]), ("std", &[])];
    report("good", &good);
    report("cyclic", &cyclic);
    report("missing", &missing);
    report("reserved", &reserved);
}
