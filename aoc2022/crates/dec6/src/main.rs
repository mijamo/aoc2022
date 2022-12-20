use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let reader = BufReader::new(file);
    let content = reader.lines().next().unwrap().unwrap();
    let mut start_buffer = ['z', 'z', 'z', 'z'];
    for (i, c) in content.char_indices() {
        start_buffer = start_buffer[1..]
            .into_iter()
            .map(|v| *v)
            .chain([c])
            .collect::<Vec<char>>()
            .try_into()
            .unwrap();
        if HashSet::<char>::from_iter(start_buffer).len() == 4 {
            println!(
                "The first 4 characters are {} and can be found starting from position {}",
                start_buffer.into_iter().collect::<String>(),
                i + 1
            );
            break;
        }
    }
    Ok(())
}
