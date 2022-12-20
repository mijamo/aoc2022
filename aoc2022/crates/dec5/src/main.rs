use regex::{Match, Regex};
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

struct Stack<'a> {
    crates: RefCell<Vec<&'a char>>,
}

impl<'a> Stack<'a> {
    fn new() -> Self {
        Self {
            crates: RefCell::new(Vec::new()),
        }
    }
    fn pop(&self) -> Option<&'a char> {
        self.crates.borrow_mut().pop()
    }
    fn push(&self, value: &'a char) {
        self.crates.borrow_mut().push(value)
    }
    fn top(&self) -> Option<&'a char> {
        self.crates.borrow().last().and_then(|c| Some(*c))
    }
}

struct Ship<'a> {
    stacks: [Stack<'a>; 9],
}

impl<'a> Ship<'a> {
    fn new() -> Self {
        Self {
            stacks: [(); 9].map(|_| Stack::new()),
        }
    }

    fn move_crates(&self, from: usize, to: usize, quantity: usize) {
        let from_stack = self.stacks.get(from - 1).unwrap();
        let to_stack = self.stacks.get(to - 1).unwrap();
        let mut from_crates = from_stack.crates.borrow_mut();
        let length = from_crates.len();
        to_stack
            .crates
            .borrow_mut()
            .extend(from_crates.drain(length - quantity..length))
    }
}

#[derive(Debug)]
struct ParseError {}

fn parse_initial_layout(input: &str) -> Result<[Option<char>; 9], ParseError> {
    let mut chars = input.chars();
    chars.next();
    return chars
        .step_by(4)
        .take(9)
        .map(|c| match c {
            ' ' => None,
            letter => Some(letter),
        })
        .collect::<Vec<Option<char>>>()
        .try_into()
        .or(Err(ParseError {}));
}

fn cap_to_usize(cap: Option<Match>) -> usize {
    usize::from_str(cap.unwrap().as_str()).unwrap()
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let mut lines = BufReader::new(file).lines();
    let ship = Ship::new();
    let move_regex = Regex::new(r"^move (\d+) from (\d) to (\d)$").unwrap();
    let mut initial_layout: Vec<[Option<char>; 9]> = Vec::new();
    loop {
        let content = lines.next().unwrap().unwrap();
        match parse_initial_layout(&content) {
            Ok(chars) => initial_layout.push(chars),
            Err(_) => break,
        }
    }
    initial_layout.iter().rev().for_each(|line| {
        line.iter()
            .zip(ship.stacks.iter())
            .for_each(|(char, stack)| match char {
                Some(c) => stack.push(c),
                _ => {}
            })
    });
    for line in lines {
        let content = line.unwrap();
        if content.len() < 4 {
            break;
        }
        let capt = move_regex.captures(&content).unwrap();
        let move_quantity = cap_to_usize(capt.get(1));
        let move_from = cap_to_usize(capt.get(2));
        let move_to = cap_to_usize(capt.get(3));
        ship.move_crates(move_from, move_to, move_quantity);
    }
    let crates_at_top: String = ship.stacks.map(|s| s.top().unwrap()).into_iter().collect();
    println!("The boxes are the top are {}", crates_at_top);
    Ok(())
}
