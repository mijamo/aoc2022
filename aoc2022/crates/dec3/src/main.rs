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
    let lines = BufReader::new(file).lines();
    let mut total_priority = 0;
    for line in lines {
        let line_content = line.unwrap();
        if line_content.len() > 0 {
            let first_compartment = &line_content[0..line_content.len() / 2];
            let second_compartment = &line_content[line_content.len() / 2..line_content.len()];
            let letters_in_first = HashSet::<char>::from_iter(first_compartment.chars());
            let letters_in_second = HashSet::<char>::from_iter(second_compartment.chars());
            let letter_in_both = letters_in_first
                .intersection(&letters_in_second)
                .next()
                .unwrap();
            total_priority += priority(letter_in_both);
        }
    }
    println!("The total priority of all rucksacks is {}", total_priority);
    Ok(())
}
