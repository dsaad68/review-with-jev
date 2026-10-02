#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Priority {
    Urgent,
    Normal,
    Deferred,
}

#[derive(Debug)]
struct Ticket {
    id: u32,
    priority: Priority,
}

fn triage(tickets: &mut [Ticket]) {
    if tickets.is_empty() {
        return;
    }
    let mut low = 0;
    let mut mid = 0;
    let mut high = tickets.len() - 1;
    while mid <= high {
        match tickets[mid].priority {
            Priority::Urgent => {
                tickets.swap(low, mid);
                low += 1;
                mid += 1;
            }
            Priority::Normal => mid += 1,
            Priority::Deferred => {
                tickets.swap(mid, high);
                if high == 0 {
                    break;
                }
                high -= 1;
            }
        }
    }
}

fn main() {
    let raw = [
        (101, Priority::Deferred),
        (102, Priority::Urgent),
        (103, Priority::Normal),
        (104, Priority::Deferred),
        (105, Priority::Urgent),
        (106, Priority::Normal),
        (107, Priority::Urgent),
    ];
    let mut tickets: Vec<Ticket> = raw
        .iter()
        .map(|&(id, priority)| Ticket { id, priority })
        .collect();
    triage(&mut tickets);
    for t in &tickets {
        println!("#{} {:?}", t.id, t.priority);
    }
}
