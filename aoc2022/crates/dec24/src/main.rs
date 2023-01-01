use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Add;

struct Bounds {
    north: i32,
    south: i32,
    east: i32,
    west: i32,
}

impl Bounds {
    fn contains(&self, pos: &Pos) -> bool {
        pos.x >= self.west && pos.x <= self.east && pos.y >= self.north && pos.y <= self.south
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Pos {
    x: i32,
    y: i32,
}

impl Add<(i32, i32)> for Pos {
    type Output = Self;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        Self {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

impl Pos {
    fn distance(&self, other: &Pos) -> u32 {
        i32::abs_diff(self.x, other.x) + i32::abs_diff(self.y, other.y)
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Blizzard {
    pos: Pos,
    direction: Direction,
}

struct World {
    blizzards: Vec<Blizzard>,
    scenarios: Vec<Pos>,
    bounds: Bounds,
    destination: Pos,
    origin: Pos,
    turns: u32,
}

impl World {
    fn new(blizzards: Vec<Blizzard>, bounds: Bounds) -> Self {
        let destination = Pos {
            y: bounds.south + 1,
            x: bounds.east,
        };
        let origin = Pos { x: 1, y: 0 };
        Self {
            blizzards,
            bounds,
            scenarios: Vec::from([origin]),
            origin,
            destination,
            turns: 0,
        }
    }

    fn print(&self) {
        for y in self.bounds.north..self.bounds.south + 1 {
            let line: String = (self.bounds.west..self.bounds.east + 1)
                .map(|x| {
                    let on_cell: Vec<&Blizzard> = self
                        .blizzards
                        .iter()
                        .filter(|b| b.pos == Pos { x, y })
                        .collect();
                    match on_cell.len() {
                        0 => ".".to_string(),
                        1 => match on_cell[0].direction {
                            Direction::Right => ">".to_string(),
                            Direction::Left => "<".to_string(),
                            Direction::Up => "^".to_string(),
                            Direction::Down => "v".to_string(),
                        },
                        nb => nb.to_string(),
                    }
                })
                .collect();
            println!("{}", line);
        }
    }

    fn next_blizzards(&self) -> Vec<Blizzard> {
        self.blizzards
            .iter()
            .map(|b| {
                let pos = match b.direction {
                    Direction::Up if b.pos.y > self.bounds.north => b.pos + (0, -1),
                    Direction::Up => Pos {
                        x: b.pos.x,
                        y: self.bounds.south,
                    },
                    Direction::Down if b.pos.y < self.bounds.south => b.pos + (0, 1),
                    Direction::Down => Pos {
                        x: b.pos.x,
                        y: self.bounds.north,
                    },
                    Direction::Right if b.pos.x < self.bounds.east => b.pos + (1, 0),
                    Direction::Right => Pos {
                        x: self.bounds.west,
                        y: b.pos.y,
                    },
                    Direction::Left if b.pos.x > self.bounds.west => b.pos + (-1, 0),
                    Direction::Left => Pos {
                        x: self.bounds.east,
                        y: b.pos.y,
                    },
                };
                Blizzard {
                    direction: b.direction,
                    pos,
                }
            })
            .collect()
    }

    fn taken_cells(&self) -> HashSet<Pos> {
        self.blizzards.iter().map(|b| b.pos).collect()
    }

    fn turn(&mut self) -> Option<u32> {
        self.blizzards = self.next_blizzards();
        let taken_cells = self.taken_cells();
        let mut next_pos = HashSet::new();
        for current in self.scenarios.iter() {
            for offset in [(1, 0), (-1, 0), (0, 1), (0, -1), (0, 0)] {
                let pos = *current + offset;
                if taken_cells.contains(&pos)
                    || (!self.bounds.contains(&pos)
                        && pos != self.destination
                        && pos != self.origin)
                {
                    continue;
                }
                next_pos.insert(pos);
            }
        }
        let mut next_pos: Vec<Pos> = next_pos.into_iter().collect();
        let min_distance = next_pos
            .iter()
            .map(|p| p.distance(&self.destination))
            .min()
            .or(Some(100))
            .unwrap();
        next_pos = next_pos
            .into_iter()
            .filter(|p| p.distance(&self.destination) <= min_distance + 18)
            .collect();
        self.scenarios = next_pos;
        self.turns += 1;
        if self.scenarios.len() == 0 {
            panic!("No more valid scenario !");
        }
        println!(
            "Turn {}, {} positions still valid",
            self.turns,
            self.scenarios.len()
        );
        if min_distance == 0 {
            Some(self.turns)
        } else {
            None
        }
    }

    fn find_path(&mut self, from: Pos, to: Pos) -> u32 {
        self.destination = to;
        self.origin = from;
        self.turns = 0;
        self.scenarios = Vec::from([self.origin]);
        loop {
            match self.turn() {
                Some(res) => return res,
                None => continue,
            }
        }
    }
}

fn main() {
    let file = File::open("./src/input.txt").unwrap();
    let lines = BufReader::new(file).lines();
    let mut blizzards = Vec::new();
    let mut max_y = 0;
    let mut max_x: i32 = 0;
    for (y, line) in lines.enumerate() {
        max_y = y as i32;
        let content = line.unwrap();
        for (x, c) in content.char_indices() {
            if x as i32 > max_x {
                max_x = x as i32;
            }
            let pos = Pos {
                x: x as i32,
                y: y as i32,
            };
            match c {
                '<' => blizzards.push(Blizzard {
                    pos,
                    direction: Direction::Left,
                }),
                '>' => blizzards.push(Blizzard {
                    pos,
                    direction: Direction::Right,
                }),
                'v' => blizzards.push(Blizzard {
                    pos,
                    direction: Direction::Down,
                }),
                '^' => blizzards.push(Blizzard {
                    pos,
                    direction: Direction::Up,
                }),
                _ => {}
            }
        }
    }
    let mut world = World::new(
        blizzards,
        Bounds {
            north: 1,
            south: max_y - 1,
            west: 1,
            east: max_x - 1,
        },
    );
    let start = Pos { x: 1, y: 0 };
    let end = Pos {
        y: max_y,
        x: max_x - 1,
    };
    let first_trip = world.find_path(start, end);
    let back_trip = world.find_path(end, start);
    let last_trip = world.find_path(start, end);
    println!(
        "It took {} turns to do the first trip, then {} back, finally {} to reach the end, for a total of {}",
        first_trip, back_trip, last_trip, first_trip + back_trip + last_trip
    );
}
