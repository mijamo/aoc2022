use regex::{Match, Regex};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

struct SectionRange {
    start: i32,
    end: i32,
}

impl SectionRange {
    fn full_overlap(&self, other: &Self) -> bool {
        return (self.start >= other.start && self.end <= other.end)
            || (self.start <= other.start && self.end >= other.end);
    }

    fn overlap(&self, other: &Self) -> bool {
        return self.start <= other.end && self.end >= other.start;
    }
}

fn cap_to_i32(cap: Option<Match>) -> i32 {
    i32::from_str(cap.unwrap().as_str()).unwrap()
}

fn main() -> std::io::Result<()> {
    let range_def = Regex::new(r"^(\d+)-(\d+),(\d+)-(\d+)$").unwrap();
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let mut overlapping = 0;
    for line in lines {
        let line_content = line.unwrap();
        let captures = range_def.captures(&line_content).unwrap();
        let first_elve_range = SectionRange {
            start: cap_to_i32(captures.get(1)),
            end: cap_to_i32(captures.get(2)),
        };
        let second_elve_range = SectionRange {
            start: cap_to_i32(captures.get(3)),
            end: cap_to_i32(captures.get(4)),
        };
        if first_elve_range.overlap(&second_elve_range) {
            overlapping += 1;
        }
    }
    println!("There are {overlapping} assignments that are overlapping");
    Ok(())
}
