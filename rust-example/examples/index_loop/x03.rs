struct Leaderboard {
    players: Vec<String>,
    points: Vec<u32>,
}

impl Leaderboard {
    fn print(&self) {
        for i in 0..self.players.len() {
            println!("{:<10} {:>5}", self.players[i], self.points[i]);
        }
    }
}

fn main() {
    let board = Leaderboard {
        players: vec![
            "ana".to_string(),
            "bram".to_string(),
            "chen".to_string(),
            "dagny".to_string(),
        ],
        points: vec![1420, 1385, 990, 875],
    };
    board.print();
}
