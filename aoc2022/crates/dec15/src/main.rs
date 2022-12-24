use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::{character::complete::i32 as number, sequence::preceded, IResult};
use std::collections::HashMap;
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

    fn around(&self, radius: u32) -> Vec<Pos> {
        let radius = radius as i32;
        let mut points_around: Vec<Pos> = Vec::new();
        for d_x in 0..radius + 1 {
            for d_y in 0..radius + 1 - d_x {
                points_around.push(Pos {
                    x: self.x + d_x,
                    y: self.y + d_y,
                });
                points_around.push(Pos {
                    x: self.x - d_x,
                    y: self.y - d_y,
                });
                points_around.push(Pos {
                    x: self.x + d_x,
                    y: self.y - d_y,
                });
                points_around.push(Pos {
                    x: self.x - d_x,
                    y: self.y + d_y,
                });
            }
        }
        points_around
    }
}

enum Presence {
    Impossible,
    Beacon,
    Sensor,
}

struct Arena {
    layout: HashMap<Pos, Presence>,
}

impl Arena {
    fn new(input: Vec<(Pos, Pos)>) -> Self {
        let mut layout: HashMap<Pos, Presence> = HashMap::new();
        for (signal, beacon) in input.into_iter() {
            layout.insert(signal, Presence::Sensor);
            layout.insert(beacon, Presence::Beacon);
            let radius = signal.distance(&beacon);
            for pos in signal.around(radius).iter() {
                match layout.get(pos) {
                    None => {
                        layout.insert(*pos, Presence::Impossible);
                    }
                    Some(_) => {}
                }
            }
        }
        println!("GENERATED");
        Self { layout }
    }
}

fn pos(input: &str) -> IResult<&str, Pos> {
    let (input, x) = preceded(tag("x="), number)(input)?;
    let (input, y) = preceded(tag(", y="), number)(input)?;
    Ok((input, Pos { x, y }))
}

fn couple(input: &str) -> IResult<&str, (Pos, Pos)> {
    let (input, sensor) = preceded(tag("Sensor at "), pos)(input)?;
    let (input, beacon) = preceded(tag(": closest beacon is at "), pos)(input)?;
    Ok((input, (sensor, beacon)))
}

fn scan(input: &str) -> IResult<&str, Vec<(Pos, Pos)>> {
    separated_list1(line_ending, couple)(input)
}

fn main() -> std::io::Result<()> {
    let mut file = File::open("./src/input.txt")?;
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, data) = scan(&content).unwrap();
    let arena = Arena::new(data);
    let ruled_out = arena
        .layout
        .iter()
        .filter(|(pos, cell)| match cell {
            Presence::Beacon => false,
            _ => pos.y == 2000000,
        })
        .count();
    println!("{} cells ruled out", ruled_out);
    Ok(())
}
