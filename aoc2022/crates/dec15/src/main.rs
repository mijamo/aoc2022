use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::{character::complete::i32 as number, sequence::preceded, IResult};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn distance(&self, other: &Pos) -> u32 {
        i32::abs_diff(self.x, other.x) + i32::abs_diff(self.y, other.y)
    }
}

struct Pair {
    sensor: Pos,
    beacon: Pos,
}

impl Pair {
    fn radius(&self) -> u32 {
        self.sensor.distance(&self.beacon)
    }
}

struct Arena {
    layout: Vec<Pair>,
}

impl Arena {
    fn new(layout: Vec<Pair>) -> Self {
        Self { layout }
    }

    fn rule_out_at(&self, line: i32) -> usize {
        let mut ruled_out: HashSet<i32> = HashSet::new();
        let mut beacons_on_line: HashSet<i32> = HashSet::new();
        for pair in self.layout.iter() {
            let d_y = i32::abs_diff(pair.sensor.y, line);
            let radius = pair.radius();
            if pair.beacon.y == line {
                beacons_on_line.insert(pair.beacon.x);
            }
            if d_y > radius {
                continue;
            }
            for d_x in 0..radius + 1 - d_y {
                ruled_out.insert(pair.sensor.x + d_x as i32);
                ruled_out.insert(pair.sensor.x - d_x as i32);
            }
        }
        ruled_out.difference(&beacons_on_line).count()
    }
}

fn pos(input: &str) -> IResult<&str, Pos> {
    let (input, x) = preceded(tag("x="), number)(input)?;
    let (input, y) = preceded(tag(", y="), number)(input)?;
    Ok((input, Pos { x, y }))
}

fn pair(input: &str) -> IResult<&str, Pair> {
    let (input, sensor) = preceded(tag("Sensor at "), pos)(input)?;
    let (input, beacon) = preceded(tag(": closest beacon is at "), pos)(input)?;
    Ok((input, Pair { sensor, beacon }))
}

fn scan(input: &str) -> IResult<&str, Vec<Pair>> {
    separated_list1(line_ending, pair)(input)
}

fn main() -> std::io::Result<()> {
    let mut file = File::open("./src/input.txt")?;
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, data) = scan(&content).unwrap();
    println!("{} pairs parsed", data.len());
    let arena = Arena::new(data);
    let ruled_out = arena.rule_out_at(2000000);
    println!("{} cells ruled out", ruled_out);
    Ok(())
}
