use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::{
    character::complete::{alpha1, i64 as number, one_of, space1},
    IResult,
};
use std::collections::HashMap;
use std::{fs::File, io::Read};

enum Operation {
    Add,
    Multiply,
    Substract,
    Divide,
}

impl Operation {
    fn apply(&self, lhs: &i64, rhs: &i64) -> i64 {
        match self {
            Self::Add => lhs + rhs,
            Self::Divide => lhs / rhs,
            Self::Multiply => lhs * rhs,
            Self::Substract => lhs - rhs,
        }
    }
}

struct OperationMonkey<'a> {
    rhs: &'a str,
    lhs: &'a str,
    operation: Operation,
}

enum MonkeyType<'a> {
    Operation(OperationMonkey<'a>),
    Value(i64),
}

struct Monkey<'a> {
    id: &'a str,
    kind: MonkeyType<'a>,
}

struct Troop<'a> {
    values: HashMap<&'a str, i64>,
    monkeys: Vec<Monkey<'a>>,
}

impl<'a> Troop<'a> {
    fn new(monkeys: Vec<Monkey<'a>>) -> Self {
        let values = HashMap::from_iter(monkeys.iter().filter_map(|m| match m.kind {
            MonkeyType::Value(v) => Some((m.id, v)),
            _ => None,
        }));
        Self { monkeys, values }
    }

    fn move_up(&mut self) {
        self.monkeys.iter().for_each(|monkey| {
            if self.values.contains_key(monkey.id) {
                return;
            }
            match &monkey.kind {
                MonkeyType::Operation(op) => {
                    match (self.values.get(op.lhs), self.values.get(op.rhs)) {
                        (Some(lhs), Some(rhs)) => {
                            let value = op.operation.apply(lhs, rhs);
                            self.values.insert(monkey.id, value);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
    }

    fn compute(&mut self) -> i64 {
        loop {
            self.move_up();
            match self.values.get("root") {
                Some(result) => return *result,
                _ => {}
            }
        }
    }
}

fn operation(input: &str) -> IResult<&str, Operation> {
    let (input, matched) = one_of("+-/*")(input)?;
    Ok((
        input,
        match matched {
            '+' => Operation::Add,
            '-' => Operation::Substract,
            '*' => Operation::Multiply,
            '/' => Operation::Divide,
            _ => panic!("Unexpected operation character"),
        },
    ))
}

fn operation_monkey(input: &str) -> IResult<&str, MonkeyType> {
    let (input, lhs) = alpha1(input)?;
    let (input, _) = space1(input)?;
    let (input, operand) = operation(input)?;
    let (input, _) = space1(input)?;
    let (input, rhs) = alpha1(input)?;
    Ok((
        input,
        MonkeyType::Operation(OperationMonkey {
            lhs,
            rhs,
            operation: operand,
        }),
    ))
}

fn value_monkey(input: &str) -> IResult<&str, MonkeyType> {
    let (input, value) = number(input)?;
    Ok((input, MonkeyType::Value(value)))
}

fn monkey(input: &str) -> IResult<&str, Monkey> {
    let (input, id) = alpha1(input)?;
    let (input, _) = tag(": ")(input)?;
    let (input, kind) = alt((value_monkey, operation_monkey))(input)?;
    Ok((input, Monkey { id, kind }))
}

fn troop(input: &str) -> IResult<&str, Troop> {
    let (input, monkeys) = separated_list1(line_ending, monkey)(input)?;
    Ok((input, Troop::new(monkeys)))
}

fn main() {
    let mut file = File::open("./src/input.txt").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, mut troop) = troop(&content).unwrap();
    println!("Parsed {} monkeys", troop.monkeys.len());
    let res = troop.compute();
    println!("The monkey yelled {}", res);
}
