use std::collections::{HashSet, VecDeque};

enum ParseError {
    Empty,
    Ragged(usize),
}

struct Maze {
    cells: Vec<Vec<bool>>,
    width: usize,
    height: usize,
}

impl Maze {
    fn parse(text: &str) -> Result<Maze, ParseError> {
        let cells: Vec<Vec<bool>> = text
            .lines()
            .map(|line| line.chars().map(|c| c != '#').collect())
            .collect();
        let width = cells.first().map(|row| row.len()).ok_or(ParseError::Empty)?;
        if let Some(i) = cells.iter().position(|row| row.len() != width) {
            return Err(ParseError::Ragged(i));
        }
        let height = cells.len();
        Ok(Maze { cells, width, height })
    }

    fn open(&self, x: i64, y: i64) -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < self.width
            && (y as usize) < self.height
            && self.cells[y as usize][x as usize]
    }

    fn shortest(&self, start: (i64, i64), goal: (i64, i64)) -> Option<usize> {
        const STEPS: [(i64, i64); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let mut seen = HashSet::new();
        let mut frontier = VecDeque::new();
        seen.insert(start);
        frontier.push_back((start, 0));
        while let Some(((x, y), dist)) = frontier.pop_front() {
            if (x, y) == goal {
                return Some(dist);
            }
            for (dx, dy) in STEPS.iter() {
                let next = (x + dx, y + dy);
                if self.open(next.0, next.1) && seen.insert(next) {
                    frontier.push_back((next, dist + 1));
                }
            }
        }
        None
    }
}

fn main() {
    let text = "\
..#.......
..#.####..
....#..#..
###.#..#.#
....#.....
.####.###.
..........";
    match Maze::parse(text) {
        Ok(maze) => {
            let goal = (maze.width as i64 - 1, maze.height as i64 - 1);
            match maze.shortest((0, 0), goal) {
                Some(d) => println!("{}x{} maze, shortest path {}", maze.width, maze.height, d),
                None => println!("no path"),
            }
        }
        Err(ParseError::Empty) => println!("empty maze"),
        Err(ParseError::Ragged(row)) => println!("row {} has the wrong width", row),
    }
}
