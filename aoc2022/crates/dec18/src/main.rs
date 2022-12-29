use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::ops::{Add, Sub};

use nom::character::complete::{char, line_ending, u32 as number};
use nom::multi::separated_list1;
use nom::IResult;

struct ImpossiblePosition {}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Pos {
    x: u32,
    y: u32,
    z: u32,
}

impl Pos {
    fn connects(&self, other: &Self) -> bool {
        match (
            u32::abs_diff(self.x, other.x),
            u32::abs_diff(self.y, other.y),
            u32::abs_diff(self.z, other.z),
        ) {
            (1, 0, 0) | (0, 1, 0) | (0, 0, 1) => true,
            _ => false,
        }
    }

    fn touches(&self, other: &Self) -> bool {
        return u32::abs_diff(self.x, other.x) < 2
            && u32::abs_diff(self.y, other.y) < 2
            && u32::abs_diff(self.z, other.z) < 2;
    }

    fn connecting_cells(&self) -> Vec<Pos> {
        let mut positions = Vec::new();
        let diffs: [(u32, u32, u32); 3] = [(1, 0, 0), (0, 1, 0), (0, 0, 1)];
        for diff in diffs {
            positions.push(self + diff);
            match self.checked_sub(diff) {
                Ok(res) => positions.push(res),
                _ => {}
            };
        }
        return positions;
    }

    fn checked_sub(&self, rhs: (u32, u32, u32)) -> Result<Self, ImpossiblePosition> {
        match (
            self.x.checked_sub(rhs.0),
            self.y.checked_sub(rhs.1),
            self.z.checked_sub(rhs.2),
        ) {
            (Some(x), Some(y), Some(z)) => Ok(Self { x, y, z }),
            _ => Err(ImpossiblePosition {}),
        }
    }
}

impl<'a> Add<(u32, u32, u32)> for &'a Pos {
    type Output = Pos;

    fn add(self, rhs: (u32, u32, u32)) -> Self::Output {
        Pos {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
            z: self.z + rhs.2,
        }
    }
}

struct Bounds {
    max_x: u32,
    min_x: u32,
    min_y: u32,
    max_y: u32,
    min_z: u32,
    max_z: u32,
}

impl Bounds {
    fn contains(&self, pos: &Pos) -> bool {
        return pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.y >= self.min_y
            && pos.y <= self.max_y
            && pos.z >= self.min_z
            && pos.z <= self.max_z;
    }

    fn enlarge(&self) -> Self {
        Self {
            min_x: u32::checked_sub(self.min_x, 1).or(Some(0)).unwrap(),
            min_y: u32::checked_sub(self.min_y, 1).or(Some(0)).unwrap(),
            min_z: u32::checked_sub(self.min_z, 1).or(Some(0)).unwrap(),
            max_x: self.max_x + 1,
            max_y: self.max_y + 1,
            max_z: self.max_z + 1,
        }
    }
}

struct World {
    droplets: Vec<Pos>,
}

impl World {
    fn new(droplets: Vec<Pos>) -> Self {
        Self { droplets }
    }

    fn surface_area(&self) -> usize {
        let mut explorer = AreaExplorer::new(&self.droplets, self.bounds().enlarge());
        explorer.explore();
        let inner_cells = explorer.inner_cells();
        println!("{} inner cells detected", inner_cells.len());
        let count_droplets = self.droplets.len();
        let mut surface_area = count_droplets * 6;
        for a in 0..count_droplets {
            let lhs = self.droplets.get(a).unwrap();
            for b in a..count_droplets {
                let rhs = self.droplets.get(b).unwrap();
                if lhs.connects(rhs) {
                    surface_area -= 2;
                }
            }
            for inner in inner_cells.iter() {
                if lhs.connects(inner) {
                    surface_area -= 1;
                }
            }
        }
        return surface_area;
    }

    fn bounds(&self) -> Bounds {
        let first = self.droplets.get(0).unwrap();
        let mut bounds = Bounds {
            max_x: first.x,
            min_x: first.x,
            max_y: first.y,
            min_y: first.y,
            max_z: first.z,
            min_z: first.z,
        };
        self.droplets.iter().for_each(|pos| {
            if pos.x < bounds.min_x {
                bounds.min_x = pos.x
            }
            if pos.x > bounds.max_x {
                bounds.max_x = pos.x
            }
            if pos.y < bounds.min_y {
                bounds.min_y = pos.y
            }
            if pos.y > bounds.max_y {
                bounds.max_y = pos.y
            }
            if pos.z < bounds.min_z {
                bounds.min_z = pos.z
            }
            if pos.z > bounds.max_z {
                bounds.max_z = pos.z
            }
        });
        return bounds;
    }
}

struct AreaExplorer {
    bounds: Bounds,
    droplets: HashSet<Pos>,
    outside_cells: HashSet<Pos>,
    current_positions: Vec<Pos>,
}

impl AreaExplorer {
    fn new(droplets: &Vec<Pos>, bounds: Bounds) -> Self {
        let start = Pos {
            x: bounds.min_x,
            y: bounds.min_y,
            z: bounds.min_z,
        };
        Self {
            droplets: droplets.iter().map(|p| *p).collect(),
            bounds,
            outside_cells: HashSet::new(),
            current_positions: Vec::from([start]),
        }
    }

    fn explore(&mut self) {
        let mut next_positions = Vec::new();
        for cell in self.current_positions.iter() {
            for possibility in cell.connecting_cells() {
                if !self.bounds.contains(&possibility)
                    || self.outside_cells.contains(&possibility)
                    || self.droplets.contains(&possibility)
                {
                    continue;
                }
                self.outside_cells.insert(possibility.clone());
                next_positions.push(possibility);
            }
        }
        self.current_positions = next_positions;
        if self.current_positions.len() > 0 {
            self.explore()
        }
    }

    fn inner_cells(&self) -> Vec<Pos> {
        let mut inner_cells = Vec::new();
        for x in self.bounds.min_x..self.bounds.max_x + 1 {
            for y in self.bounds.min_y..self.bounds.max_y + 1 {
                for z in self.bounds.min_z..self.bounds.max_z + 1 {
                    let pos = Pos { x, y, z };
                    if !self.outside_cells.contains(&pos) && !self.droplets.contains(&pos) {
                        inner_cells.push(pos);
                    }
                }
            }
        }
        return inner_cells;
    }
}

fn droplet(input: &str) -> IResult<&str, Pos> {
    let (input, coords) = separated_list1(char(','), number)(input)?;
    Ok((
        input,
        Pos {
            x: coords[0],
            y: coords[1],
            z: coords[2],
        },
    ))
}

fn droplets(input: &str) -> IResult<&str, Vec<Pos>> {
    separated_list1(line_ending, droplet)(input)
}

fn main() {
    let mut file = File::open("./src/input.txt").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, droplets) = droplets(&content).unwrap();
    let world = World::new(droplets);
    println!("Total surface area is {}", world.surface_area());
}
