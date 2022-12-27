use std::fs::File;
use std::io::Read;
use std::ops::{Add, Range, Sub};

struct Collisions {
    left: bool,
    right: bool,
    bottom: bool,
}

impl Collisions {
    fn new() -> Self {
        Self {
            left: false,
            right: false,
            bottom: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Pos {
    x: u32,
    y: u32,
}

impl Add<(u32, u32)> for Pos {
    type Output = Self;

    fn add(self, rhs: (u32, u32)) -> Self::Output {
        Self {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

impl Sub<(u32, u32)> for Pos {
    type Output = Self;

    fn sub(self, rhs: (u32, u32)) -> Self::Output {
        Self {
            x: self.x - rhs.0,
            y: self.y - rhs.1,
        }
    }
}

#[derive(Clone, Copy)]
enum RockType {
    HLine,
    Plus,
    Corner,
    VLine,
    Block,
}

struct Rock {
    kind: RockType,
    bottom_left: Pos,
    moving: Option<Collisions>,
}

impl Rock {
    fn top(&self) -> u32 {
        self.bottom_left.y
            + match self.kind {
                RockType::HLine => 0,
                RockType::Plus => 2,
                RockType::Corner => 2,
                RockType::VLine => 3,
                RockType::Block => 1,
            }
    }

    fn pixel_positions(&self) -> Vec<Pos> {
        let mut positions = Vec::new();
        match self.kind {
            RockType::Block => {
                positions.push(self.bottom_left);
                positions.push(self.bottom_left + (1, 0));
                positions.push(self.bottom_left + (1, 1));
                positions.push(self.bottom_left + (0, 1));
            }
            RockType::Corner => {
                positions.push(self.bottom_left + (2, 0));
                positions.push(self.bottom_left + (1, 0));
                positions.push(self.bottom_left);
                positions.push(self.bottom_left + (2, 1));
                positions.push(self.bottom_left + (2, 2));
            }
            RockType::HLine => {
                positions.push(self.bottom_left);
                positions.push(self.bottom_left + (1, 0));
                positions.push(self.bottom_left + (2, 0));
                positions.push(self.bottom_left + (3, 0));
            }
            RockType::Plus => {
                positions.push(self.bottom_left + (1, 0));
                positions.push(self.bottom_left + (0, 1));
                positions.push(self.bottom_left + (1, 1));
                positions.push(self.bottom_left + (2, 1));
                positions.push(self.bottom_left + (1, 2));
            }
            RockType::VLine => {
                positions.push(self.bottom_left);
                positions.push(self.bottom_left + (0, 1));
                positions.push(self.bottom_left + (0, 2));
                positions.push(self.bottom_left + (0, 3));
            }
        }
        return positions;
    }

    fn rasterize(&self, pixels: &mut PixelTable) {
        let pixel_type = match self.moving {
            Some(_) => Pixel::Active,
            None => Pixel::Fix,
        };
        self.pixel_positions()
            .into_iter()
            .for_each(|pos| pixels.set(pos, pixel_type));
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pixel {
    Empty,
    Fix,
    Active,
}

struct PixelTable {
    range: Range<u32>,
    content: Vec<Pixel>,
}

impl PixelTable {
    fn new(lines: Range<u32>) -> Self {
        Self {
            content: (0..(lines.len() * 7)).map(|_| Pixel::Empty).collect(),
            range: lines.clone(),
        }
    }

    fn index(&self, at: Pos) -> Option<usize> {
        if self.range.contains(&at.y) {
            Some((at.y as usize - self.range.start as usize) * 7 + (at.x as usize))
        } else {
            None
        }
    }

    fn set(&mut self, at: Pos, pixel: Pixel) {
        match self.index(at) {
            Some(index) => {
                self.content[index] = pixel;
            }
            None => {}
        }
    }

    fn at(&self, at: Pos) -> Option<Pixel> {
        self.index(at).and_then(|index| Some(self.content[index]))
    }
}

enum Effect {
    Wind,
    Fall,
    Generate,
}

struct World {
    rocks: Vec<Rock>,
    wind_tick: u32,
    rock_tick: usize,
    pattern: Vec<Direction>,
    next_effect: Effect,
}

impl World {
    fn new(wind_pattern: Vec<Direction>) -> Self {
        Self {
            rocks: Vec::new(),
            wind_tick: 0,
            rock_tick: 0,
            pattern: wind_pattern,
            next_effect: Effect::Generate,
        }
    }

    fn turn(&mut self) {
        match &self.next_effect {
            Effect::Wind => {
                self.wind_effect();
                self.detect_collisions();
            }
            Effect::Fall => {
                self.fall_rock();
                self.detect_collisions();
            }
            Effect::Generate => {
                self.generate_rock();
            }
        }
    }

    fn generate_rock(&mut self) {
        let rock_type = [
            RockType::HLine,
            RockType::Plus,
            RockType::Corner,
            RockType::VLine,
            RockType::Block,
        ][self.rock_tick % 5];
        let top = self
            .rocks
            .iter()
            .map(|r| r.top())
            .max()
            .and_then(|top| Some(top + 1))
            .or(Some(0))
            .unwrap()
            + 3;
        self.rocks.push(Rock {
            kind: rock_type,
            bottom_left: Pos { y: top, x: 2 },
            moving: Some(Collisions::new()),
        });
        self.rock_tick += 1;
        self.next_effect = Effect::Wind;
    }

    fn fall_rock(&mut self) {
        self.next_effect = Effect::Wind;
        self.rocks
            .iter_mut()
            .filter(|r| r.moving.is_some())
            .for_each(|rock| match &rock.moving {
                Some(c) if c.bottom => {
                    self.next_effect = Effect::Generate;
                    rock.moving = None;
                }
                _ => {
                    rock.bottom_left = rock.bottom_left - (0, 1);
                }
            });
    }

    fn wind_effect(&mut self) {
        let wind = self
            .pattern
            .get(self.wind_tick as usize % self.pattern.len())
            .unwrap()
            .clone();
        self.rocks
            .iter_mut()
            .filter(|r| r.moving.is_some())
            .for_each(|rock| {
                match (wind, &rock.moving) {
                    (Direction::Left, Some(c)) if !c.left => {
                        rock.bottom_left = rock.bottom_left - (1, 0)
                    }
                    (Direction::Right, Some(c)) if !c.right => {
                        rock.bottom_left = rock.bottom_left + (1, 0)
                    }
                    _ => {}
                };
            });
        self.wind_tick += 1;
        self.next_effect = Effect::Fall;
    }

    fn rasterize(&self, lines: Range<u32>) -> PixelTable {
        let mut pixels = PixelTable::new(lines.clone());
        self.rocks
            .iter()
            .filter(|rock| rock.bottom_left.y > lines.start || rock.bottom_left.y + 4 < lines.end)
            .for_each(|rock| {
                rock.rasterize(&mut pixels);
            });
        pixels
    }

    fn detect_collisions(&mut self) -> PixelTable {
        let mut top_line = 0;
        let mut bottom_line: Option<u32> = None;
        for rock in &self.rocks {
            if !rock.moving.is_some() {
                continue;
            }
            if rock.bottom_left.y + 3 > top_line {
                top_line = rock.bottom_left.y + 3;
            }
            let bottom_for_rock = if rock.bottom_left.y < 1 {
                0
            } else {
                rock.bottom_left.y - 1
            };
            match bottom_line {
                None => bottom_line = Some(bottom_for_rock),
                Some(v) if v > bottom_for_rock => bottom_line = Some(bottom_for_rock),
                _ => {}
            }
        }
        let lines = bottom_line.or(Some(0)).unwrap()..top_line + 1;
        let pixels = self.rasterize(lines);
        self.rocks
            .iter_mut()
            .filter(|r| r.moving.is_some())
            .for_each(|rock| {
                let mut collisions = Collisions::new();
                rock.pixel_positions().into_iter().for_each(|pos| {
                    if !collisions.left
                        && (pos.x == 0 || pixels.at(pos - (1, 0)).unwrap() == Pixel::Fix)
                    {
                        collisions.left = true;
                    }
                    if !collisions.bottom
                        && (pos.y == 0 || pixels.at(pos - (0, 1)).unwrap() == Pixel::Fix)
                    {
                        collisions.bottom = true;
                    }
                    if !collisions.right
                        && (pos.x == 6 || pixels.at(pos + (1, 0)).unwrap() == Pixel::Fix)
                    {
                        collisions.right = true;
                    }
                });
                rock.moving = Some(collisions);
            });
        return pixels;
    }

    fn print(&self) {
        let top = self
            .rocks
            .iter()
            .map(|r| r.top())
            .max()
            .or(Some(0))
            .unwrap()
            + 1;
        let pixels = self.rasterize(0..top);
        for y in 0..top {
            let line: String = (0..7)
                .map(|x| match pixels.content[(top - y - 1) as usize * 7 + x] {
                    Pixel::Active => '@',
                    Pixel::Empty => '.',
                    Pixel::Fix => '#',
                })
                .collect();
            println!("{}", line);
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut file = File::open("./src/input.txt")?;
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let wind_pattern: Vec<Direction> = content
        .chars()
        .take_while(|c| match c {
            '>' | '<' => true,
            _ => false,
        })
        .map(|c| match c {
            '>' => Direction::Right,
            '<' => Direction::Left,
            other => panic!("Unexpected character {}", other),
        })
        .collect();
    let mut world = World::new(wind_pattern);
    while world.rocks.len() < 2023 {
        world.turn();
    }
    let tallest = world
        .rocks
        .iter()
        .filter(|r| r.moving.is_none())
        .map(|r| r.top())
        .max()
        .or(Some(0))
        .unwrap()
        + 1;
    println!("Tallest rock stands {} high", tallest);
    println!("There are {} rocks in the world", world.rocks.len());
    //world.print();
    Ok(())
}
