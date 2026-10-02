use std::collections::{BTreeSet, HashMap};
use std::sync::mpsc;
use std::thread;

trait Task: Sync {
    fn name(&self) -> &str;
    fn needs(&self) -> &[&str];
    fn run(&self, inputs: &[u64]) -> Result<u64, String>;
}

struct Step {
    name: &'static str,
    needs: Vec<&'static str>,
    weight: u64,
}

impl Task for Step {
    fn name(&self) -> &str {
        self.name
    }

    fn needs(&self) -> &[&str] {
        &self.needs
    }

    fn run(&self, inputs: &[u64]) -> Result<u64, String> {
        if self.weight == 0 {
            return Err(format!("step {} has no work configured", self.name));
        }
        inputs
            .iter()
            .try_fold(self.weight, |acc, &x| acc.checked_add(x))
            .ok_or_else(|| format!("step {} overflowed its budget", self.name))
    }
}

fn plan<'t, T: Task>(tasks: &'t [T]) -> Result<Vec<Vec<&'t T>>, String> {
    let mut by_name: HashMap<&str, &T> = HashMap::new();
    for task in tasks {
        if by_name.insert(task.name(), task).is_some() {
            return Err(format!("task {} is defined twice", task.name()));
        }
    }
    let mut pending: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    for task in tasks {
        for dep in task.needs() {
            if !by_name.contains_key(dep) {
                return Err(format!("task {} needs unknown task {dep}", task.name()));
            }
            dependents.entry(dep).or_default().push(task.name());
        }
        pending.insert(task.name(), task.needs().len());
    }

    let mut ready: BTreeSet<&str> = pending.iter().filter(|(_, &n)| n == 0).map(|(&k, _)| k).collect();
    let mut waves = Vec::new();
    let mut scheduled = 0;
    while !ready.is_empty() {
        let wave: Vec<&str> = std::mem::take(&mut ready).into_iter().collect();
        for name in &wave {
            for next in dependents.get(name).into_iter().flatten() {
                if let Some(count) = pending.get_mut(next) {
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(next);
                    }
                }
            }
        }
        scheduled += wave.len();
        waves.push(wave.iter().filter_map(|n| by_name.get(n).copied()).collect());
    }
    if scheduled < tasks.len() {
        let stuck: Vec<&str> = pending.iter().filter(|(_, &n)| n > 0).map(|(&k, _)| k).collect();
        return Err(format!("dependency cycle among {stuck:?}"));
    }
    Ok(waves)
}

fn execute<T: Task>(tasks: &[T]) -> Result<HashMap<String, u64>, String> {
    let waves = plan(tasks)?;
    let mut results: HashMap<String, u64> = HashMap::new();
    for wave in waves {
        let (tx, rx) = mpsc::channel();
        thread::scope(|s| {
            for task in wave {
                let inputs: Vec<u64> = task
                    .needs()
                    .iter()
                    .filter_map(|dep| results.get(*dep).copied())
                    .collect();
                let tx = tx.clone();
                s.spawn(move || tx.send((task.name().to_string(), task.run(&inputs))));
            }
        });
        drop(tx);
        for (name, outcome) in rx {
            let value = outcome.map_err(|e| format!("wave failed: {e}"))?;
            results.insert(name, value);
        }
    }
    Ok(results)
}

fn step(name: &'static str, needs: &[&'static str], weight: u64) -> Step {
    Step { name, needs: needs.to_vec(), weight }
}

fn main() {
    let pipelines = [
        (
            "release",
            vec![
                step("fetch", &[], 3),
                step("compile", &["fetch"], 10),
                step("lint", &["fetch"], 2),
                step("test", &["compile"], 7),
                step("package", &["test", "lint"], 1),
            ],
        ),
        ("typo", vec![step("fetch", &[], 1), step("build", &["fecth"], 4)]),
        ("loop", vec![step("a", &["c"], 1), step("b", &["a"], 1), step("c", &["b"], 1)]),
        ("dupe", vec![step("fetch", &[], 1), step("fetch", &[], 2)]),
        ("empty", vec![step("fetch", &[], 1), step("idle", &["fetch"], 0)]),
    ];
    for (label, tasks) in &pipelines {
        match execute(tasks) {
            Ok(results) => {
                let mut finished: Vec<_> = results.into_iter().collect();
                finished.sort();
                println!("{label}: {finished:?}");
            }
            Err(e) => println!("{label}: {e}"),
        }
    }
}
