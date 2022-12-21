use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Add;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
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

enum Next {
    More(Box<Knot>),
    Tail(Rc<RefCell<HashSet<Position>>>),
}

struct Knot {
    pos: Position,
    next: Next,
}

impl Knot {
    fn move_by(&mut self, offset: Position) {
        self.pos = self.pos + offset;
        self.bring_tail();
    }

    fn bring_tail(&mut self) {
        match &mut self.next {
            Next::Tail(recorder) => {
                recorder.borrow_mut().insert(self.pos);
            }
            Next::More(next_knot) => {
                let movement = match (self.pos.x - next_knot.pos.x, self.pos.y - next_knot.pos.y) {
                    (0, 0) => Movement::Neutral,
                    (1, 0) => Movement::Neutral,
                    (0, 1) => Movement::Neutral,
                    (-1, 0) => Movement::Neutral,
                    (0, -1) => Movement::Neutral,
                    (1, 1) => Movement::Neutral,
                    (-1, -1) => Movement::Neutral,
                    (-1, 1) => Movement::Neutral,
                    (1, -1) => Movement::Neutral,
                    (2, 2) => Movement::new(1, 1),
                    (-2, -2) => Movement::new(-1, -1),
                    (2, -2) => Movement::new(1, -1),
                    (-2, 2) => Movement::new(-1, 1),
                    (2, y) => Movement::new(1, y),
                    (-2, y) => Movement::new(-1, y),
                    (x, 2) => Movement::new(x, 1),
                    (x, -2) => Movement::new(x, -1),
                    (x, y) => panic!("Invalid distance found: x {} y {}", x, y),
                };
                match movement {
                    Movement::Neutral => {}
                    Movement::Offset(off) => next_knot.move_by(off),
                }
            }
        }
    }
}

struct Rope {
    head: Knot,
}

impl Rope {
    fn new(length: u32, recorder: Rc<RefCell<HashSet<Position>>>) -> Self {
        let initial_pos = Position { x: 0, y: 0 };
        recorder.borrow_mut().insert(initial_pos.clone());
        let tail = Knot {
            pos: initial_pos.clone(),
            next: Next::Tail(recorder),
        };
        let mut head = tail;
        for _ in 0..length - 1 {
            head = Knot {
                pos: initial_pos.clone(),
                next: Next::More(Box::new(head)),
            }
        }
        Self { head }
    }

    fn move_up(&mut self) {
        self.head.move_by(Position { x: 0, y: 1 });
    }

    fn move_down(&mut self) {
        self.head.move_by(Position { x: 0, y: -1 });
    }

    fn move_left(&mut self) {
        self.head.move_by(Position { x: -1, y: 0 });
    }

    fn move_right(&mut self) {
        self.head.move_by(Position { x: 1, y: 0 });
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let position_recorder = Rc::new(RefCell::new(HashSet::<Position>::new()));
    let mut rope = Rope::new(10, position_recorder.clone());
    for line in lines {
        let content = line.unwrap();
        let mut chars = content.chars();
        let command = chars.next().unwrap();
        let quantity = i32::from_str(&content[2..]).unwrap();
        for _ in 0..quantity {
            match command {
                'D' => {
                    rope.move_down();
                }
                'U' => {
                    rope.move_up();
                }
                'L' => {
                    rope.move_left();
                }
                'R' => {
                    rope.move_right();
                }
                c => panic!("Unexpected character {}", c),
            }
        }
    }
    println!(
        "The tail has gone through {} positions",
        position_recorder.borrow().len()
    );
    Ok(())
}
