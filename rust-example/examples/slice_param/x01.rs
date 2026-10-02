fn enqueue(queue: &mut Vec<String>, name: &str, priority: u8) {
    if priority > 3 {
        queue.push(format!("urgent:{name}"));
    } else {
        queue.push(format!("normal:{name}"));
    }
}

fn total_weight(weights: &[u32]) -> u32 {
    weights.iter().sum()
}

fn main() {
    let mut queue = Vec::new();
    enqueue(&mut queue, "backup", 5);
    enqueue(&mut queue, "cleanup", 1);
    enqueue(&mut queue, "report", 4);

    let weights = vec![3, 7, 2];
    println!("{} jobs, total weight {}", queue.len(), total_weight(&weights));
    for job in &queue {
        println!("{job}");
    }
}
