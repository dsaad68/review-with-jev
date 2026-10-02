struct Job {
    id: u32,
    due_at: i64,
}

fn seconds_until(job: &Job, now: i64) -> u64 {
    (job.due_at - now) as u64
}

fn main() {
    let now = 1_700_000_000;
    let jobs = vec![
        Job { id: 1, due_at: now + 30 },
        Job { id: 2, due_at: now + 3600 },
        Job { id: 3, due_at: now + 5 },
    ];
    let mut waits: Vec<(u32, u64)> = jobs
        .iter()
        .map(|j| (j.id, seconds_until(j, now)))
        .collect();
    waits.sort_by_key(|&(_, w)| w);
    for (id, wait) in &waits {
        println!("job {id} runs in {wait}s");
    }
    if let Some((id, _)) = waits.first() {
        println!("next up: job {id}");
    }
}
