use std::fs::File;
use std::io::Read;

use nom::character::complete::{char, line_ending, u32 as number};
use nom::multi::separated_list1;
use nom::IResult;

struct Pos {
    x: u32,
    y: u32,
    z: u32,
}

impl Pos {
    fn touches(&self, other: &Self) -> bool {
        match (
            u32::abs_diff(self.x, other.x),
            u32::abs_diff(self.y, other.y),
            u32::abs_diff(self.z, other.z),
        ) {
            (1, 0, 0) | (0, 1, 0) | (0, 0, 1) => true,
            _ => false,
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
        let count_droplets = self.droplets.len();
        let mut surface_area = count_droplets * 6;
        for a in 0..count_droplets {
            let lhs = self.droplets.get(a).unwrap();
            for b in a..count_droplets {
                let rhs = self.droplets.get(b).unwrap();
                if lhs.touches(rhs) {
                    surface_area -= 2;
                }
            }
        }
        surface_area
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
