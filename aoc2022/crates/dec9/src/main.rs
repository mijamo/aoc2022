use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}

enum Movement {
    Neutral,
    Offset(Position),
}

impl Movement {
    fn new(x: i32, y: i32) -> Self {
        Self::Offset(Position { x, y })
    }
}

struct Rope {
    head: Position,
    tail: Position,
}

impl Rope {
    fn new() -> Self {
        Self {
            head: Position { x: 0, y: 0 },
            tail: Position { x: 0, y: 0 },
        }
    }

    fn move_up(&mut self) {
        self.head.y += 1;
        self.bring_tail();
    }

    fn move_down(&mut self) {
        self.head.y -= 1;
        self.bring_tail();
    }

    fn move_left(&mut self) {
        self.head.x -= 1;
        self.bring_tail();
    }

    fn move_right(&mut self) {
        self.head.x += 1;
        self.bring_tail();
    }

    fn bring_tail(&mut self) {
        let movement = match (self.head.x - self.tail.x, self.head.y - self.tail.y) {
            (0, 0) => Movement::Neutral,
            (1, 0) => Movement::Neutral,
            (0, 1) => Movement::Neutral,
            (-1, 0) => Movement::Neutral,
            (0, -1) => Movement::Neutral,
            (1, 1) => Movement::Neutral,
            (-1, -1) => Movement::Neutral,
            (-1, 1) => Movement::Neutral,
            (1, -1) => Movement::Neutral,
            (2, y) => Movement::new(1, y),
            (-2, y) => Movement::new(-1, y),
            (x, 2) => Movement::new(x, 1),
            (x, -2) => Movement::new(x, -1),
            (x, y) => panic!("Invalid distance found: x {} y {}", x, y),
        };
        match movement {
            Movement::Neutral => {}
            Movement::Offset(off) => {
                self.tail.x += off.x;
                self.tail.y += off.y;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let mut rope = Rope::new();
    let mut position_recorder = HashSet::<Position>::new();
    position_recorder.insert(rope.tail);
    for line in lines {
        let content = line.unwrap();
        let mut chars = content.chars();
        let command = chars.next().unwrap();
        let quantity = i32::from_str(&content[2..]).unwrap();
        for _ in 0..quantity {
            match command {
                'D' => {
                    rope.move_down();
                    position_recorder.insert(rope.tail);
                }
                'U' => {
                    rope.move_up();
                    position_recorder.insert(rope.tail);
                }
                'L' => {
                    rope.move_left();
                    position_recorder.insert(rope.tail);
                }
                'R' => {
                    rope.move_right();
                    position_recorder.insert(rope.tail);
                }
                c => panic!("Unexpected character {}", c),
            }
        }
    }
    println!(
        "The tail has gone through {} positions",
        position_recorder.len()
    );
    Ok(())
}
