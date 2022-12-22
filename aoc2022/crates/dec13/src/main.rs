use nom::branch::alt;
use nom::character::complete::{char, digit1, line_ending, multispace0, multispace1};
use nom::combinator::map_res;
use nom::complete::tag;
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::{delimited, tuple};
use nom::IResult;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

enum PacketData {
    List(Vec<PacketData>),
    Value(u16),
}

enum CmpRes {
    Yes,
    No,
    Maybe,
}

impl PacketData {
    fn comes_before(&self, other: &Self) -> CmpRes {
        match (self, other) {
            (Self::Value(lhs), Self::Value(rhs)) => match lhs.cmp(rhs) {
                Ordering::Equal => CmpRes::Maybe,
                Ordering::Greater => CmpRes::No,
                Ordering::Less => CmpRes::Yes,
            },
            (Self::List(_), Self::Value(rhs)) => {
                self.comes_before(&Self::List(Vec::from([Self::Value(*rhs)])))
            }
            (Self::Value(lhs), Self::List(_)) => {
                Self::List(Vec::from([Self::Value(*lhs)])).comes_before(other)
            }
            (Self::List(lhs), Self::List(rhs)) => {
                let mut idx = 0;
                loop {
                    let left_item = lhs.get(idx);
                    let right_item = rhs.get(idx);
                    match (left_item, right_item) {
                        (None, Some(_)) => return CmpRes::Yes,
                        (Some(_), None) => return CmpRes::No,
                        (None, None) => return CmpRes::Maybe,
                        (Some(lhs_v), Some(rhs_v)) => match lhs_v.comes_before(rhs_v) {
                            CmpRes::Maybe => {}
                            other => return other,
                        },
                    }
                    idx += 1;
                }
            }
        }
    }
}

fn number(input: &str) -> IResult<&str, PacketData> {
    let (input, number) = map_res(digit1, str::parse)(input)?;
    Ok((input, PacketData::Value(number)))
}

fn array(input: &str) -> IResult<&str, PacketData> {
    let (input, content) = delimited(
        char('['),
        separated_list0(char(','), alt((array, number))),
        char(']'),
    )(input)?;
    Ok((input, PacketData::List(content)))
}

fn pair(input: &str) -> IResult<&str, (PacketData, PacketData)> {
    let (input, first) = array(input)?;
    let (input, _) = line_ending(input)?;
    let (input, second) = array(input)?;
    Ok((input, (first, second)))
}

fn signal(input: &str) -> IResult<&str, Vec<(PacketData, PacketData)>> {
    separated_list0(many1(line_ending), pair)(input)
}

fn main() -> std::io::Result<()> {
    let mut file = File::open("./src/input.txt")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let (_, signal) = signal(&content).unwrap();
    println!("Parsed {} pairs", signal.len());
    let index_sum: usize = signal
        .iter()
        .enumerate()
        .filter_map(|(i, (first, second))| match first.comes_before(second) {
            CmpRes::Yes => Some(i + 1),
            CmpRes::No => None,
            CmpRes::Maybe => panic!("Unexpected comparison result"),
        })
        .sum();
    println!("Indexes of packets in right order sum to {}", index_sum);
    Ok(())
}
