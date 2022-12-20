use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn priority(input: &char) -> i32 {
    let char_value = *input as i32;
    if char_value > 96 {
        return char_value - 96;
    }
    return char_value - 38;
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let mut lines = BufReader::new(file).lines();
    let mut total_priority = 0;
    loop {
        let line = lines.next();
        let line_content = match line {
            None => break,
            Some(Err(_)) => break,
            Some(Ok(content)) => content,
        };
        if &line_content == "" {
            break;
        };
        let group = [
            line_content,
            lines.next().unwrap().unwrap(),
            lines.next().unwrap().unwrap(),
        ];
        let letters = group.map(|g| g.chars().collect::<HashSet<char>>());
        let letter_in_common = letters
            .iter()
            .fold(letters[0].clone(), |acc, set| {
                acc.intersection(set).map(|v| *v).collect::<HashSet<char>>()
            })
            .iter()
            .next()
            .unwrap()
            .clone();
        total_priority += priority(&letter_in_common);
    }
    println!("The total priority of all rucksacks is {}", total_priority);
    Ok(())
}
