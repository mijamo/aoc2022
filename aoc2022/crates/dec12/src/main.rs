use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

type Altitude = u32;
type Coord = i16;

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
struct Position {
    x: Coord,
    y: Coord,
}

enum Cell {
    Origin,
    Destination,
    Other(Altitude),
}

impl Cell {
    fn altitude(&self) -> Altitude {
        match self {
            Self::Origin => 1,
            Self::Destination => 26,
            Self::Other(v) => *v,
        }
    }
}

struct Grid {
    lines: Vec<Vec<Cell>>,
}

impl Grid {
    fn altitude(&self, pos: &Position) -> Altitude {
        self.lines[usize::try_from(pos.y).unwrap()][usize::try_from(pos.x).unwrap()].altitude()
    }

    fn in_bound(&self, pos: &Position) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && pos.x < self.lines[0].len().try_into().unwrap()
            && pos.y < self.lines.len().try_into().unwrap()
    }

    fn origin(&self) -> Option<Position> {
        for (y, line) in self.lines.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                match cell {
                    Cell::Origin => {
                        return Some(Position {
                            x: x.try_into().unwrap(),
                            y: y.try_into().unwrap(),
                        })
                    }
                    _ => continue,
                }
            }
        }
        return None;
    }

    fn destination(&self) -> Option<Position> {
        for (y, line) in self.lines.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                match cell {
                    Cell::Destination => {
                        return Some(Position {
                            x: x.try_into().unwrap(),
                            y: y.try_into().unwrap(),
                        })
                    }
                    _ => continue,
                }
            }
        }
        return None;
    }
}

type Path = Vec<Position>;

struct PathExplorer<'a> {
    grid: &'a Grid,
    touched_positions: HashSet<Position>,
    current_paths: Vec<Path>,
    destination: Position,
}

impl<'a> PathExplorer<'a> {
    fn new(grid: &'a Grid, from: Position, to: Position) -> Self {
        Self {
            grid,
            touched_positions: HashSet::new(),
            current_paths: Vec::from([Vec::from([from])]),
            destination: to,
        }
    }

    fn possible_next_moves(&self, from: &Position) -> Vec<Position> {
        let mut valid_moves: Vec<Position> = Vec::new();
        let current_altitude = self.grid.altitude(from);
        for [x, y] in [[-1, 0], [1, 0], [0, 1], [0, -1]] {
            let candidate = Position {
                x: from.x + x,
                y: from.y + y,
            };
            if !self.touched_positions.contains(&candidate)
                && self.grid.in_bound(&candidate)
                && self.grid.altitude(&candidate) <= current_altitude + 1
            {
                valid_moves.push(candidate)
            }
        }
        return valid_moves;
    }

    fn explore(&mut self) -> Path {
        loop {
            let mut next_paths = Vec::<Path>::new();
            if self.current_paths.is_empty() {
                break;
            }
            for existing in &self.current_paths {
                let last_position = existing.last().unwrap();
                for next_move in &self.possible_next_moves(last_position) {
                    let mut new_path = existing.clone();
                    new_path.push(*next_move);
                    if next_move == &self.destination {
                        return new_path;
                    }
                    self.touched_positions.insert(*next_move);
                    next_paths.push(new_path);
                }
            }
            self.current_paths = next_paths;
        }
        panic!("Did not find a path")
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let grid_lines: Vec<Vec<Cell>> = lines
        .map(|l| {
            l.unwrap()
                .chars()
                .map(|char| match char {
                    'S' => Cell::Origin,
                    'E' => Cell::Destination,
                    other => Cell::Other(other as Altitude - 96),
                })
                .collect()
        })
        .collect();
    let grid = Grid { lines: grid_lines };
    let mut explorer =
        PathExplorer::new(&grid, grid.origin().unwrap(), grid.destination().unwrap());
    let final_path = explorer.explore();
    println!("The final path took {} steps", final_path.len() - 1);
    Ok(())
}
