use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::{character::complete::i32 as number, sequence::preceded, IResult};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::ops::{Mul, Range};
use std::rc::Rc;

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

struct MultiRange {
    elements: Vec<Rc<Range<i32>>>,
}

impl MultiRange {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    fn add(&mut self, range: Range<i32>) {
        let mut new_range: Range<i32> = range;
        let mut new_elements: Vec<Rc<Range<i32>>> = Vec::new();
        for current in self.elements.iter() {
            match (
                new_range.start.cmp(&current.end),
                new_range.end.cmp(&current.start),
            ) {
                (Ordering::Less, Ordering::Greater) => {
                    new_range = Range {
                        start: i32::min(current.start, new_range.start),
                        end: i32::max(current.end, new_range.end),
                    };
                }
                _ => {
                    new_elements.push(current.clone());
                }
            }
        }
        new_elements.push(Rc::new(new_range));
        self.elements = new_elements;
    }

    fn len(&self) -> usize {
        self.elements
            .iter()
            .map(|r| r.end - r.start)
            .sum::<i32>()
            .try_into()
            .unwrap()
    }

    fn contains(&self, value: &i32) -> bool {
        self.elements.iter().any(|r| r.contains(value))
    }
}

impl Arena {
    fn new(layout: Vec<Pair>) -> Self {
        Self { layout }
    }

    fn rule_out_at(&self, line: i32) -> usize {
        let mut ruled_out = MultiRange::new();
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
            let max_delta = (radius - d_y) as i32;
            let min_x = pair.sensor.x - max_delta;
            let max_x = pair.sensor.x + max_delta + 1;
            ruled_out.add(min_x..max_x);
        }
        ruled_out.len()
            - beacons_on_line
                .iter()
                .filter(|p| ruled_out.contains(p))
                .count()
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
