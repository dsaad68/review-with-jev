use std::collections::{BTreeMap, HashMap};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
struct Profile {
    name: String,
    out_dir: String,
    extension: String,
    flags: Vec<String>,
}

#[derive(Debug)]
struct Job {
    id: usize,
    source: String,
    profile: Arc<Profile>,
}

#[derive(Debug)]
struct Outcome { job_id: usize, worker: usize, output: String, command: String }

trait Stage: Send + Sync {
    fn label(&self) -> &str;
    fn process(&self, job: &Job) -> Result<String, String>;
}

struct Minify;
struct Fingerprint { salt: u64 }

impl Stage for Minify {
    fn label(&self) -> &str {
        "minify"
    }
    fn process(&self, job: &Job) -> Result<String, String> {
        if job.source.ends_with(".min.js") {
            return Err(format!("{} is already minified", job.source));
        }
        Ok(format!("min({})", job.source))
    }
}

impl Stage for Fingerprint {
    fn label(&self) -> &str {
        "fingerprint"
    }
    fn process(&self, job: &Job) -> Result<String, String> {
        let hash = job.source.bytes().fold(self.salt, |h, b| h.rotate_left(5) ^ b as u64);
        Ok(format!("{:08x}", hash & 0xffff_ffff))
    }
}

fn file_stem(path: &str) -> &str {
    let base = path.rsplit('/').next().unwrap_or(path);
    base.split('.').next().unwrap_or(base)
}

fn output_path(source: &str, profile: &Profile, digest: &str) -> String {
    format!("{}/{}-{}.{}", profile.out_dir, file_stem(source), digest, profile.extension)
}

struct Planner {
    profiles: HashMap<String, Profile>,
}

impl Planner {
    fn profile_for(&self, name: &str, verbose: bool) -> Option<Profile> {
        let mut profile = self.profiles.get(name)?.clone();
        if verbose {
            profile.flags.push("--verbose".to_string());
        }
        profile.flags.sort();
        Some(profile)
    }

    fn plan(&self, manifest: Vec<(String, String)>, verbose: bool) -> Vec<Job> {
        let mut resolved: HashMap<String, Arc<Profile>> = HashMap::new();
        let mut jobs = Vec::new();
        for (id, (source, profile_name)) in manifest.into_iter().enumerate() {
            if !resolved.contains_key(&profile_name) {
                match self.profile_for(&profile_name, verbose) {
                    Some(p) => {
                        resolved.insert(profile_name.clone(), Arc::new(p));
                    }
                    None => continue,
                }
            }
            let profile = Arc::clone(&resolved[&profile_name]);
            jobs.push(Job { id, source, profile });
        }
        jobs
    }
}

fn run_pool(jobs: Vec<Job>, stages: Arc<Vec<Box<dyn Stage>>>, workers: usize) -> (Vec<Outcome>, Vec<String>) {
    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let job_rx = Arc::new(Mutex::new(job_rx));
    let (out_tx, out_rx) = mpsc::channel::<Result<Outcome, String>>();
    let mut handles = Vec::new();
    for worker in 0..workers {
        let rx = Arc::clone(&job_rx);
        let tx = out_tx.clone();
        let stages = Arc::clone(&stages);
        handles.push(thread::spawn(move || loop {
            let job = match rx.lock().unwrap().recv() {
                Ok(job) => job,
                Err(_) => break,
            };
            let mut digest = String::new();
            let mut failed = None;
            for stage in stages.iter() {
                match stage.process(&job) {
                    Ok(part) if stage.label() == "fingerprint" => digest = part,
                    Ok(_) => {}
                    Err(e) => {
                        failed = Some(format!("job {} [{}]: {}", job.id, stage.label(), e));
                        break;
                    }
                }
            }
            let msg = match failed {
                Some(err) => Err(err),
                None => Ok(Outcome {
                    job_id: job.id,
                    worker,
                    output: output_path(&job.source, &job.profile, &digest),
                    command: format!("{} {}", job.profile.name, job.profile.flags.join(" ")),
                }),
            };
            if tx.send(msg).is_err() {
                break;
            }
        }));
    }
    drop(out_tx);
    for job in jobs {
        job_tx.send(job).expect("pool closed");
    }
    drop(job_tx);
    let (mut done, mut errors) = (Vec::new(), Vec::new());
    for msg in out_rx {
        match msg {
            Ok(o) => done.push(o),
            Err(e) => errors.push(e),
        }
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
    done.sort_by_key(|o| o.job_id);
    errors.sort();
    (done, errors)
}

fn main() {
    let mut profiles = HashMap::new();
    profiles.insert("js".to_string(), Profile { name: "esbuild".into(), out_dir: "dist/js".into(), extension: "js".into(), flags: vec!["--bundle".into()] });
    profiles.insert("css".to_string(), Profile { name: "lightningcss".into(), out_dir: "dist/css".into(), extension: "css".into(), flags: vec![] });
    let planner = Planner { profiles };
    let manifest: Vec<(String, String)> = ["src/app.js:js", "src/vendor.min.js:js", "styles/site.css:css", "styles/print.css:css", "img/logo.png:img"]
        .iter()
        .filter_map(|line| line.split_once(':').map(|(a, b)| (a.to_string(), b.to_string())))
        .collect();
    let jobs = planner.plan(manifest, true);
    let stages: Arc<Vec<Box<dyn Stage>>> = Arc::new(vec![Box::new(Minify), Box::new(Fingerprint { salt: 7 })]);
    let (done, errors) = run_pool(jobs, stages, 3);
    let mut by_dir: BTreeMap<&str, Vec<&Outcome>> = BTreeMap::new();
    for o in &done {
        let dir = o.output.rsplit_once('/').map_or("", |(d, _)| d);
        by_dir.entry(dir).or_default().push(o);
    }
    for (dir, outs) in by_dir {
        let lines: Vec<String> = outs.iter().map(|o| format!("#{} {} (w{}) `{}`", o.job_id, o.output, o.worker, o.command)).collect();
        println!("{}: {:?}", dir, lines);
    }
    for e in errors {
        println!("error: {}", e);
    }
}
