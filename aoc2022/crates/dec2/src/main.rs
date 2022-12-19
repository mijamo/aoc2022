use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(PartialEq, Eq)]
enum Choice {
    Rock,
    Paper,
    Scissors,
}

impl PartialOrd for Choice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Rock, Self::Rock) => Some(Ordering::Equal),
            (Self::Rock, Self::Scissors) => Some(Ordering::Greater),
            (Self::Rock, Self::Paper) => Some(Ordering::Less),
            (Self::Paper, Self::Paper) => Some(Ordering::Equal),
            (Self::Paper, Self::Rock) => Some(Ordering::Greater),
            (Self::Paper, Self::Scissors) => Some(Ordering::Less),
            (Self::Scissors, Self::Rock) => Some(Ordering::Less),
            (Self::Scissors, Self::Paper) => Some(Ordering::Greater),
            (Self::Scissors, Self::Scissors) => Some(Ordering::Equal),
        }
    }
}

#[derive(Debug)]
struct InvalidInputError {}

impl Choice {
    fn from_first_column(input: &str) -> Result<Self, InvalidInputError> {
        match input {
            "A" => Ok(Self::Rock),
            "B" => Ok(Self::Paper),
            "C" => Ok(Self::Scissors),
            _ => Err(InvalidInputError {}),
        }
    }

    fn from_second_column(input: &str) -> Result<Self, InvalidInputError> {
        match input {
            "X" => Ok(Self::Rock),
            "Y" => Ok(Self::Paper),
            "Z" => Ok(Self::Scissors),
            _ => Err(InvalidInputError {}),
        }
    }

    const fn value(&self) -> i32 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }
}

fn score_round(self_choice: &Choice, opponant_choice: &Choice) -> i32 {
    let outcome_score = match self_choice.partial_cmp(opponant_choice).unwrap() {
        Ordering::Less => 0,
        Ordering::Equal => 3,
        Ordering::Greater => 6,
    };
    let choice_score = self_choice.value();
    return outcome_score + choice_score;
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let mut total_score = 0;
    for line in lines {
        let line_content = line.unwrap();
        if line_content.len() == 3 {
            let opponant_choice = Choice::from_first_column(&line_content[0..1]).unwrap();
            let self_choice = Choice::from_second_column(&line_content[2..3]).unwrap();
            total_score += score_round(&self_choice, &opponant_choice);
        }
    }
    println!("Total score if everything goes right: {}", total_score);
    Ok(())
}
