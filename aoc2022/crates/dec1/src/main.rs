use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

struct ElveSupply {
    id: i16,
    calories: i32,
}

impl ElveSupply {
    fn new(id: i16) -> Self {
        Self {
            id: id,
            calories: 0,
        }
    }

    fn register_food(&mut self, value: i32) {
        self.calories += value
    }
}

struct ElveRegister {
    elves: Vec<ElveSupply>,
    counter: i16,
}

impl ElveRegister {
    fn new() -> Self {
        Self {
            elves: Vec::new(),
            counter: 0,
        }
    }

    fn move_to_new(&mut self) -> &mut ElveSupply {
        self.counter += 1;
        self.elves.push(ElveSupply::new(self.counter));
        return self.elves.last_mut().unwrap();
    }

    fn elves_by_calories(&self) -> Vec<&ElveSupply> {
        let mut new_vec: Vec<&ElveSupply> = self.elves.iter().collect();
        new_vec.sort_by(|a, b| b.calories.cmp(&a.calories));
        return new_vec;
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let mut elves_register = ElveRegister::new();
    let mut current_elve = elves_register.move_to_new();
    for l in lines {
        let line_content = l.unwrap();
        if line_content == String::from("") {
            current_elve = elves_register.move_to_new();
        } else {
            let value = i32::from_str(&line_content).unwrap();
            current_elve.register_food(value);
        }
    }
    let elves_carryings_most = elves_register.elves_by_calories();
    let top_three = &elves_carryings_most[0..3];
    let sum_of_calories = top_three.iter().fold(0, |acc, elve| elve.calories + acc);
    println!("Top three elves gather {} together", sum_of_calories);
    Ok(())
}
