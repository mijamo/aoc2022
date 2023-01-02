use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{fmt::Display, str::FromStr};

#[derive(PartialEq, Eq)]
enum Digit {
    DoubleMinus,
    Minus,
    Zero,
    One,
    Two,
}

impl Digit {
    fn value(&self) -> i64 {
        match self {
            Self::DoubleMinus => -2,
            Self::Minus => -1,
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
        }
    }
}

impl Display for Digit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::DoubleMinus => "=",
            Self::Minus => "-",
            Self::Zero => "0",
            Self::One => "1",
            Self::Two => "2",
        })
    }
}

#[derive(Debug)]
struct InvalidDigit {}

impl TryFrom<char> for Digit {
    type Error = InvalidDigit;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '=' => Ok(Self::DoubleMinus),
            '-' => Ok(Self::Minus),
            '0' => Ok(Self::Zero),
            '1' => Ok(Self::One),
            '2' => Ok(Self::Two),
            _ => Err(InvalidDigit {}),
        }
    }
}

impl TryFrom<i64> for Digit {
    type Error = InvalidDigit;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            -2 => Ok(Self::DoubleMinus),
            -1 => Ok(Self::Minus),
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(InvalidDigit {}),
        }
    }
}

struct Snafu {
    digits: Vec<Digit>,
}

impl FromStr for Snafu {
    type Err = InvalidDigit;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let digits: Vec<Digit> = s
            .chars()
            .map(|c| c.try_into())
            .collect::<Result<Vec<Digit>, InvalidDigit>>()?;
        Ok(Self { digits })
    }
}

impl From<i64> for Snafu {
    fn from(value: i64) -> Self {
        let mut highest_digit = 0;
        loop {
            if 3 * 5_i64.pow(highest_digit) > value {
                break;
            }
            highest_digit += 1;
        }
        let mut digits = Vec::new();
        let mut remains = value;
        let mut i = highest_digit;
        let mut digit_unit = 5_i64.pow(i);
        loop {
            let quotient = remains.div_euclid(digit_unit);
            let remainder = remains.rem_euclid(digit_unit);
            if i == 0 {
                digits.push(quotient.try_into().unwrap());
                break;
            }
            i -= 1;
            let next_digit_unit = 5_i64.pow(i);
            if remainder > next_digit_unit * 2 {
                remains = remainder - digit_unit;
                digits.push((quotient + 1).try_into().unwrap());
            } else {
                remains = remainder;
                digits.push(quotient.try_into().unwrap());
            }
            digit_unit = next_digit_unit;
        }
        Self { digits }
    }
}

impl Display for Snafu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.digits
            .iter()
            .map(|d| f.write_str(&d.to_string()))
            .collect()
    }
}

impl Snafu {
    fn value(&self) -> i64 {
        let length = self.digits.len() as u32;
        self.digits.iter().enumerate().fold(0, |acc, (i, d)| {
            acc + d.value() * 5_i64.pow(length - 1 - i as u32)
        })
    }
}

fn main() {
    let file = File::open("./src/input.txt").unwrap();
    let lines = BufReader::new(file).lines();
    let numbers: Vec<Snafu> = lines
        .map(|l| Snafu::from_str(&l.unwrap()).unwrap())
        .collect();
    let final_value: i64 = numbers.iter().map(|n| n.value()).sum();
    println!(
        "The total fuel requires is {}, or written in SNAFU {}",
        final_value,
        Snafu::from(final_value).to_string()
    );
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::Snafu;

    #[test]
    fn test_parse_nb() {
        assert_eq!(Snafu::from_str("1=").unwrap().value(), 3);
        assert_eq!(Snafu::from_str("12").unwrap().value(), 7);
        assert_eq!(Snafu::from_str("1=11-2").unwrap().value(), 2022);
    }

    #[test]
    fn test_serialize_nb() {
        assert_eq!(Snafu::from(3).to_string(), "1=");
        assert_eq!(Snafu::from(7).to_string(), "12");
        assert_eq!(Snafu::from(2022).to_string(), "1=11-2");
    }
}
