use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{collections::VecDeque, ops::Add};

#[derive(Clone, Copy, PartialEq, Debug)]
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

#[derive(Debug)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug)]
struct Bounds {
    north: i32,
    south: i32,
    east: i32,
    west: i32,
}

impl Bounds {
    fn contains(&self, pos: &Pos) -> bool {
        pos.x >= self.west && pos.x <= self.east && pos.y <= self.south && pos.y >= self.north
    }
}

struct Game {
    elves: Vec<Pos>,
    tick: u32,
}

impl Game {
    fn new(elves: Vec<Pos>) -> Self {
        Self { elves, tick: 0 }
    }

    fn bounds(&self) -> Bounds {
        let first_elf = self.elves[0];
        let mut bounds = Bounds {
            north: first_elf.y,
            south: first_elf.y,
            west: first_elf.x,
            east: first_elf.x,
        };
        for elf in self.elves.iter() {
            if elf.x < bounds.west {
                bounds.west = elf.x
            }
            if elf.x > bounds.east {
                bounds.east = elf.x
            }
            if elf.y < bounds.north {
                bounds.north = elf.y
            }
            if elf.y > bounds.south {
                bounds.south = elf.y
            }
        }
        return bounds;
    }

    fn empty_in_bounds(&self) -> i32 {
        let bounds = self.bounds();
        let size = (bounds.south - bounds.north + 1) * (bounds.east - bounds.west + 1);
        size - self.elves.len() as i32
    }

    fn is_elve_around(&self, pos: Pos) -> bool {
        self.elves.iter().any(|other| {
            i32::abs_diff(pos.x, other.x) <= 1
                && i32::abs_diff(pos.y, other.y) <= 1
                && *other != pos
        })
    }

    fn is_elve_north(&self, pos: Pos) -> bool {
        self.elves
            .iter()
            .any(|other| i32::abs_diff(pos.x, other.x) <= 1 && other.y - pos.y == -1)
    }

    fn is_elve_south(&self, pos: Pos) -> bool {
        self.elves
            .iter()
            .any(|other| i32::abs_diff(pos.x, other.x) <= 1 && other.y - pos.y == 1)
    }

    fn is_elve_west(&self, pos: Pos) -> bool {
        self.elves
            .iter()
            .any(|other| i32::abs_diff(pos.y, other.y) <= 1 && other.x - pos.x == -1)
    }

    fn is_elve_east(&self, pos: Pos) -> bool {
        self.elves
            .iter()
            .any(|other| i32::abs_diff(pos.y, other.y) <= 1 && other.x - pos.x == 1)
    }

    fn make_decision(&mut self) -> Vec<Option<Pos>> {
        let mut possibilities = VecDeque::from([
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]);
        possibilities.rotate_left((self.tick % 4) as usize);
        self.elves
            .iter()
            .map(|pos| {
                let mut destination: Option<Pos> = None;
                if !self.is_elve_around(*pos) {
                    return None;
                }
                'pos: for possibility in possibilities.iter() {
                    let can_do = match possibility {
                        Direction::Up => !self.is_elve_north(*pos),
                        Direction::Down => !self.is_elve_south(*pos),
                        Direction::Left => !self.is_elve_west(*pos),
                        Direction::Right => !self.is_elve_east(*pos),
                    };

                    if can_do {
                        destination = Some(match possibility {
                            Direction::Up => *pos + (0, -1),
                            Direction::Down => *pos + (0, 1),
                            Direction::Left => *pos + (-1, 0),
                            Direction::Right => *pos + (1, 0),
                        });
                        break 'pos;
                    }
                }
                destination
            })
            .collect()
    }

    fn make_movements(&mut self, mut movements: Vec<Option<Pos>>) {
        let length = movements.len();
        for i in 0..length {
            let first = movements[i];
            match first {
                None => {}
                Some(_) if i < length - 1 => {
                    for j in (i + 1)..length {
                        if movements[j] == first {
                            movements[j] = None;
                            movements[i] = None;
                        }
                    }
                }
                _ => {}
            }
        }
        movements
            .iter()
            .enumerate()
            .for_each(|(i, movement)| match movement {
                Some(pos) => self.elves[i] = *pos,
                None => {}
            })
    }

    fn round(&mut self) {
        let movements = self.make_decision();
        self.make_movements(movements);
        self.tick += 1;
    }

    fn print(&self) {
        let bounds = self.bounds();
        for y in bounds.north..bounds.south + 1 {
            let line: String = (bounds.west..bounds.east + 1)
                .map(|x| {
                    if self.elves.iter().any(|p| *p == Pos { x, y }) {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect();
            println!("{}", line);
        }
    }
}

fn main() {
    let file = File::open("./src/input.txt").unwrap();
    let lines = BufReader::new(file).lines();
    let mut elves = Vec::new();
    for (y, line) in lines.enumerate() {
        let content = line.unwrap();
        for (x, c) in content.char_indices() {
            match c {
                '#' => elves.push(Pos {
                    x: x as i32,
                    y: y as i32,
                }),
                _ => {}
            }
        }
    }
    println!("{} elves on the map", elves.len());
    let mut game = Game::new(elves);
    game.print();
    let rounds = 10;
    for i in 0..rounds {
        game.round();
        println!();
        println!("ROUND {}", i + 1);
        println!("Bounds: {:?}", game.bounds());
        game.print();
    }
    println!(
        "{} empty ground tiles after {} rounds",
        game.empty_in_bounds(),
        rounds
    );
}
