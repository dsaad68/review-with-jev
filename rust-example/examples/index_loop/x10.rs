use std::cmp::Ordering;
use std::fmt::Debug;

trait Ranked {
    fn rank(&self) -> u64;
}

#[derive(Debug, Clone)]
struct Job {
    name: &'static str,
    deadline: u64,
}

impl Ranked for Job {
    fn rank(&self) -> u64 {
        self.deadline
    }
}

#[derive(Debug, Clone)]
struct Packet {
    seq: u32,
    ttl: u8,
}

impl Ranked for Packet {
    fn rank(&self) -> u64 {
        u64::from(self.ttl) * 1000 + u64::from(self.seq)
    }
}

struct MinHeap<T: Ranked> {
    slots: Vec<T>,
}

impl<T: Ranked + Debug> MinHeap<T> {
    fn new() -> Self {
        MinHeap { slots: Vec::new() }
    }

    fn len(&self) -> usize {
        self.slots.len()
    }

    fn push(&mut self, item: T) {
        self.slots.push(item);
        let mut child = self.slots.len() - 1;
        while child > 0 {
            let parent = (child - 1) / 2;
            if self.slots[child].rank() < self.slots[parent].rank() {
                self.slots.swap(child, parent);
                child = parent;
            } else {
                break;
            }
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.slots.is_empty() {
            return None;
        }
        let last = self.slots.len() - 1;
        self.slots.swap(0, last);
        let top = self.slots.pop();
        self.sift_down(0);
        top
    }

    fn sift_down(&mut self, mut node: usize) {
        let n = self.slots.len();
        loop {
            let left = 2 * node + 1;
            let right = left + 1;
            let mut smallest = node;
            if left < n && self.compare(left, smallest) == Ordering::Less {
                smallest = left;
            }
            if right < n && self.compare(right, smallest) == Ordering::Less {
                smallest = right;
            }
            if smallest == node {
                return;
            }
            self.slots.swap(node, smallest);
            node = smallest;
        }
    }

    fn compare(&self, a: usize, b: usize) -> Ordering {
        self.slots[a].rank().cmp(&self.slots[b].rank())
    }

    fn heapify(items: Vec<T>) -> Self {
        let mut heap = MinHeap { slots: items };
        for node in (0..heap.slots.len() / 2).rev() {
            heap.sift_down(node);
        }
        heap
    }

    fn drain_sorted(mut self) -> Vec<T> {
        let mut out = Vec::with_capacity(self.len());
        while let Some(item) = self.pop() {
            out.push(item);
        }
        out
    }
}

fn main() {
    let mut jobs = MinHeap::new();
    for (name, deadline) in [("backup", 900), ("report", 300), ("deploy", 120), ("audit", 450)] {
        jobs.push(Job { name, deadline });
    }
    jobs.push(Job { name: "hotfix", deadline: 60 });
    println!("{} jobs queued", jobs.len());
    while let Some(job) = jobs.pop() {
        println!("run {} (due {})", job.name, job.deadline);
    }

    let packets: Vec<Packet> = (0u32..10)
        .map(|seq| Packet { seq, ttl: (seq.wrapping_mul(7) % 5) as u8 })
        .collect();
    let ordered = MinHeap::heapify(packets).drain_sorted();
    for p in ordered.iter().take(4) {
        println!("{:?} rank {}", p, p.rank());
    }
}
