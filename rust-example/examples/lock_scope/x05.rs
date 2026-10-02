use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Team {
    Red,
    Blue,
}

struct Lobby {
    rosters: Mutex<HashMap<Team, Vec<String>>>,
    history: Mutex<Vec<(String, Team)>>,
}

impl Lobby {
    fn new() -> Self {
        let rosters = HashMap::from([(Team::Red, Vec::new()), (Team::Blue, Vec::new())]);
        Lobby { rosters: Mutex::new(rosters), history: Mutex::new(Vec::new()) }
    }

    fn join(&self, player: String, team: Team) -> usize {
        let size = {
            let mut rosters = self.rosters.lock().unwrap();
            let roster = rosters.entry(team).or_default();
            roster.push(player.clone());
            roster.len()
        };
        self.history.lock().unwrap().push((player, team));
        size
    }

    fn rebalance(&self) -> Option<String> {
        let mut rosters = self.rosters.lock().unwrap();
        let [Some(red), Some(blue)] = rosters.get_disjoint_mut([&Team::Red, &Team::Blue]) else {
            return None;
        };
        let (from, to, team) = if red.len() > blue.len() + 1 {
            (red, blue, Team::Blue)
        } else if blue.len() > red.len() + 1 {
            (blue, red, Team::Red)
        } else {
            return None;
        };
        let moved = from.pop()?;
        to.push(moved.clone());
        drop(rosters);
        self.history.lock().unwrap().push((moved.clone(), team));
        Some(moved)
    }
}

fn main() {
    let lobby = Arc::new(Lobby::new());
    let mut arrivals = Vec::new();
    for n in 0..6 {
        let lobby = Arc::clone(&lobby);
        arrivals.push(thread::spawn(move || {
            let size = lobby.join(format!("player{n}"), Team::Red);
            thread::sleep(Duration::from_millis(1));
            size
        }));
    }
    for arrival in arrivals {
        arrival.join().unwrap();
    }
    while let Some(name) = lobby.rebalance() {
        println!("moved {name}");
    }
    println!("history has {} entries", lobby.history.lock().unwrap().len());
}
