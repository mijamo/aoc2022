use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1, line_ending};
use nom::combinator::map_res;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct Pos {
    x: usize,
    y: usize,
}

#[derive(Debug)]
struct Bounds {
    north: usize,
    south: usize,
    west: usize,
    east: usize,
}

impl Bounds {
    fn union(&self, other: &Self) -> Self {
        Self {
            north: usize::max(other.north, self.north),
            south: usize::max(other.south, self.south),
            west: usize::max(other.west, self.west),
            east: usize::max(other.east, self.east),
        }
    }

    fn extends_to(&mut self, pos: &Pos) {
        match pos.x {
            v if v < self.west => self.west = v,
            v if v > self.east => self.east = v,
            _ => {}
        }
        match pos.y {
            v if v < self.north => self.north = v,
            v if v > self.south => self.south = v,
            _ => {}
        }
    }
}

enum Cell {
    Origin,
    Sand,
    Rock,
}

#[derive(Debug)]
struct FullCaveError {}

struct Arena {
    cells: HashMap<Pos, Cell>,
    origin: Pos,
    bounds: Bounds,
}

enum ExploreResult {
    Possible(Pos),
    Blocked,
    OutOfBounds,
}

impl Arena {
    fn new(input: &Vec<Vec<Pos>>) -> Self {
        let mut cells: HashMap<Pos, Cell> = HashMap::new();
        let mut bounds = Bounds {
            north: 0,
            south: 0,
            east: 500,
            west: 500,
        };
        let origin = Pos { x: 500, y: 0 };
        cells.insert(origin, Cell::Origin);
        for line in input.iter() {
            let mut points_in_line = line.iter();
            let mut first_point = points_in_line.next().unwrap();
            bounds.extends_to(&first_point);
            while let Some(next_point) = points_in_line.next() {
                bounds.extends_to(next_point);
                let mut x_range = [first_point.x, next_point.x];
                x_range.sort();
                let mut y_range = [first_point.y, next_point.y];
                y_range.sort();
                println!("X line: {:?}, Y line: {:?}", x_range, y_range);
                for x in x_range[0]..x_range[1] + 1 {
                    for y in y_range[0]..y_range[1] + 1 {
                        cells.insert(Pos { x, y }, Cell::Rock);
                    }
                }
                first_point = next_point;
            }
        }
        Self {
            cells,
            origin,
            bounds,
        }
    }

    fn at(&self, pos: &Pos) -> Option<&Cell> {
        self.cells.get(pos)
    }

    fn generate_sand(&mut self) -> Result<(), FullCaveError> {
        match self.try_under(&self.origin) {
            Some(fall_point) => {
                println!("Sand at {:?}", fall_point);
                self.cells.insert(fall_point, Cell::Sand);
                Ok(())
            }
            None => Err(FullCaveError {}),
        }
    }

    fn in_bounds(&self, pos: &Pos) -> bool {
        return pos.x <= self.bounds.east
            && pos.x >= self.bounds.west
            && pos.y >= self.bounds.north
            && pos.y <= self.bounds.south;
    }

    fn fall_to_solid(&self, pos: &Pos) -> Option<Pos> {
        let mut current_pos = *pos;
        let mut next_pos = Pos {
            x: pos.x,
            y: pos.y + 1,
        };
        let mut current_cell = self.at(&current_pos);
        let mut next_cell = self.at(&next_pos);
        loop {
            println!("Next: {:?}", next_pos);
            if !self.in_bounds(&next_pos) {
                return None;
            }
            match next_cell {
                Some(_) => match current_cell {
                    Some(_) => return None,
                    None => return Some(current_pos),
                },
                None => {}
            };
            current_pos.y += 1;
            next_pos.y += 1;
            current_cell = self.at(&current_pos);
            next_cell = self.at(&next_pos);
        }
    }

    fn try_under(&self, pos: &Pos) -> Option<Pos> {
        self.fall_to_solid(&pos)
            .and_then(|fall_point| match self.try_around(&fall_point) {
                None => Some(fall_point),
                Some(side_point) => Some(side_point),
            })
    }

    fn try_around(&self, pos: &Pos) -> Option<Pos> {
        let bottom_left = Pos {
            x: pos.x - 1,
            y: pos.y + 1,
        };
        let bottom_right = Pos {
            x: pos.x + 1,
            y: pos.y + 1,
        };
        self.try_under(&bottom_left)
            .or_else(|| self.try_under(&bottom_right))
    }

    fn print_pattern(&self) {
        for y in self.bounds.north..self.bounds.south + 1 {
            let line: String = (self.bounds.west..self.bounds.east + 1)
                .map(|x| match self.at(&Pos { x, y }) {
                    Some(Cell::Origin) => '*',
                    Some(Cell::Rock) => '#',
                    Some(Cell::Sand) => 'o',
                    None => '.',
                })
                .collect();
            println!("{}", line);
        }
    }
}

fn number(input: &str) -> IResult<&str, usize> {
    map_res(digit1, str::parse)(input)
}

fn point(input: &str) -> IResult<&str, Pos> {
    let (input, coord) = separated_pair(number, char(','), number)(input)?;
    Ok((
        input,
        Pos {
            x: coord.0,
            y: coord.1,
        },
    ))
}

fn line(input: &str) -> IResult<&str, Vec<Pos>> {
    separated_list1(tag(" -> "), point)(input)
}

fn scan(input: &str) -> IResult<&str, Vec<Vec<Pos>>> {
    separated_list1(line_ending, line)(input)
}

fn main() -> std::io::Result<()> {
    let mut file = File::open("./src/test.txt")?;
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, layout) = scan(&content).unwrap();
    let mut arena = Arena::new(&layout);
    println!("{} cells parsed", arena.cells.len());
    arena.print_pattern();
    println!("Bounds for arena: {:#?}", arena.bounds);
    let mut sand_added = 0;
    while let Ok(_) = arena.generate_sand() {
        sand_added += 1;
    }
    arena.print_pattern();
    println!("Added {} units of sand until it is full", sand_added);
    Ok(())
}
