use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let reader = BufReader::new(file);
    let content = reader.lines().next().unwrap().unwrap();
    let mut start_buffer: [char; 14] = ['z'].repeat(14).try_into().unwrap();
    for (i, c) in content.char_indices() {
        start_buffer = start_buffer[1..]
            .into_iter()
            .map(|v| *v)
            .chain([c])
            .collect::<Vec<char>>()
            .try_into()
            .unwrap();
        if HashSet::<char>::from_iter(start_buffer).len() == 14 {
            println!(
                "The first 14 characters are {} and can be found finishing at position {}",
                start_buffer.into_iter().collect::<String>(),
                i + 1
            );
            break;
        }
    }
    Ok(())
}
