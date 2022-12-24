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

enum Impossible {
    Blocked,
    OutOfBounds,
}

type ExploreResult = Result<Pos, Impossible>;

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

    fn generate_sand(&mut self) -> ExploreResult {
        self.try_point(&Pos {
            x: self.origin.x,
            y: self.origin.y + 1,
        })
        .and_then(|fall_point| {
            self.cells.insert(fall_point, Cell::Sand);
            Ok(fall_point)
        })
    }

    fn in_bounds(&self, pos: &Pos) -> bool {
        return pos.x <= self.bounds.east
            && pos.x >= self.bounds.west
            && pos.y >= self.bounds.north
            && pos.y <= self.bounds.south;
    }

    fn try_point(&self, pos: &Pos) -> ExploreResult {
        let current_cell = self.at(pos);
        if current_cell.is_some() {
            return Err(Impossible::Blocked);
        }
        if !self.in_bounds(pos) {
            return Err(Impossible::OutOfBounds);
        }
        let under = Pos {
            x: pos.x,
            y: pos.y + 1,
        };
        let bottom_left = Pos {
            x: pos.x - 1,
            y: pos.y + 1,
        };
        let bottom_right = Pos {
            x: pos.x + 1,
            y: pos.y + 1,
        };
        match self.try_point(&under) {
            Err(Impossible::OutOfBounds) => return Err(Impossible::OutOfBounds),
            Ok(res) => return Ok(res),
            _ => {}
        }
        match self.try_point(&bottom_left) {
            Err(Impossible::OutOfBounds) => return Err(Impossible::OutOfBounds),
            Ok(res) => return Ok(res),
            _ => {}
        }
        match self.try_point(&bottom_right) {
            Err(Impossible::OutOfBounds) => return Err(Impossible::OutOfBounds),
            Err(Impossible::Blocked) => return Ok(*pos),
            Ok(res) => return Ok(res),
        }
    }

    fn print_pattern(&self) {
        for y in self.bounds.north..self.bounds.south + 2 {
            let line: String = (self.bounds.west - 1..self.bounds.east + 2)
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
    let mut file = File::open("./src/input.txt")?;
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, layout) = scan(&content).unwrap();
    let mut arena = Arena::new(&layout);
    arena.print_pattern();
    let mut sand_added = 0;
    while let Ok(_) = arena.generate_sand() {
        sand_added += 1;
    }
    println!();
    println!();
    println!("RESULTS");
    println!();
    println!();
    arena.print_pattern();
    println!("Added {} units of sand until it is full", sand_added);
    Ok(())
}
