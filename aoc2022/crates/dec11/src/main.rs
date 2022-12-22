use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, multispace0, one_of, space1};
use nom::combinator::map_res;
use nom::multi::{many0, separated_list0};
use nom::sequence::{pair, preceded};
use nom::IResult;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;

enum Operand {
    Mutiply,
    Add,
}

enum OperationTarget {
    Old,
    Number(i32),
}

impl OperationTarget {
    fn value(&self, old: i32) -> i32 {
        match self {
            Self::Old => old,
            Self::Number(nb) => *nb,
        }
    }
}

struct Operation {
    rhs: OperationTarget,
    lhs: OperationTarget,
    operand: Operand,
}

enum ConditionFn {
    DivisibleBy(i32),
}

struct Condition {
    test: ConditionFn,
    when_true: usize,
    when_false: usize,
}

struct Monkey {
    items: VecDeque<i32>,
    operation: Operation,
    condition: Condition,
    handle_counter: u32,
}

impl Monkey {
    fn handle_item(&mut self, item: i32) -> (i32, usize) {
        self.handle_counter += 1;
        let lhs_value = self.operation.lhs.value(item);
        let rhs_value = self.operation.rhs.value(item);
        let intermediate_value = match self.operation.operand {
            Operand::Add => lhs_value + rhs_value,
            Operand::Mutiply => lhs_value * rhs_value,
        };
        let relief_value = intermediate_value / 3;
        let target = match self.condition.test {
            ConditionFn::DivisibleBy(modulo) if relief_value % modulo == 0 => {
                self.condition.when_true
            }
            ConditionFn::DivisibleBy(_) => self.condition.when_false,
        };
        return (relief_value, target);
    }
}

struct Troop {
    monkeys: Vec<RefCell<Monkey>>,
}

impl Troop {
    fn monkey_turn(&mut self, idx: usize) {
        let mut monkey = self.monkeys.get(idx).unwrap().borrow_mut();
        while let Some(item) = monkey.items.pop_front() {
            let (new_value, target) = monkey.handle_item(item);
            self.monkeys
                .get(target)
                .unwrap()
                .borrow_mut()
                .items
                .push_back(new_value);
        }
    }

    fn round(&mut self) {
        for i in 0..self.monkeys.len() {
            self.monkey_turn(i);
        }
    }
}

fn integer(input: &str) -> IResult<&str, i32> {
    map_res(digit1, str::parse::<i32>)(input)
}

fn index_value(input: &str) -> IResult<&str, usize> {
    map_res(digit1, str::parse::<usize>)(input)
}

fn starting_items(input: &str) -> IResult<&str, VecDeque<i32>> {
    let (input, _) = tag("Starting items: ")(input)?;
    let (input, values) = separated_list0(tag(", "), integer)(input)?;
    Ok((input, VecDeque::from(values)))
}

fn operand(input: &str) -> IResult<&str, Operand> {
    let (input, operand) = one_of("*+")(input)?;
    match operand {
        '*' => Ok((input, Operand::Mutiply)),
        '+' => Ok((input, Operand::Add)),
        _ => panic!("Unexpected operand"),
    }
}

fn old_target(input: &str) -> IResult<&str, OperationTarget> {
    let (input, _) = tag("old")(input)?;
    Ok((input, OperationTarget::Old))
}

fn number_target(input: &str) -> IResult<&str, OperationTarget> {
    let (input, number) = integer(input)?;
    Ok((input, OperationTarget::Number(number)))
}

fn operation_target(input: &str) -> IResult<&str, OperationTarget> {
    alt((old_target, number_target))(input)
}

fn operation(input: &str) -> IResult<&str, Operation> {
    let (input, _) = tag("Operation: new =")(input)?;
    let (input, lhs) = preceded(space1, operation_target)(input)?;
    let (input, operand) = preceded(space1, operand)(input)?;
    let (input, rhs) = preceded(space1, operation_target)(input)?;
    return Ok((input, Operation { lhs, rhs, operand }));
}

fn condition(input: &str) -> IResult<&str, Condition> {
    let (input, divisible_by) = preceded(tag("Test: divisible by "), integer)(input)?;
    let (input, if_true) = preceded(
        pair(multispace0, tag("If true: throw to monkey ")),
        index_value,
    )(input)?;
    let (input, if_false) = preceded(
        pair(multispace0, tag("If false: throw to monkey ")),
        index_value,
    )(input)?;
    Ok((
        input,
        Condition {
            test: ConditionFn::DivisibleBy(divisible_by),
            when_true: if_true,
            when_false: if_false,
        },
    ))
}

fn monkey(input: &str) -> IResult<&str, RefCell<Monkey>> {
    let (input, _) = preceded(tag("Monkey "), integer)(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, starting_items) = preceded(multispace0, starting_items)(input)?;
    let (input, operation) = preceded(multispace0, operation)(input)?;
    let (input, condition) = preceded(multispace0, condition)(input)?;
    Ok((
        input,
        RefCell::new(Monkey {
            items: starting_items,
            operation,
            condition,
            handle_counter: 0,
        }),
    ))
}

fn troop(input: &str) -> IResult<&str, Troop> {
    let (input, monkeys) = many0(preceded(multispace0, monkey))(input)?;
    Ok((input, Troop { monkeys }))
}

fn main() -> std::io::Result<()> {
    let mut file = File::open("./src/input.txt")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let (_, mut troop) = troop(&content).unwrap();
    println!("{} monkeys in the troop", troop.monkeys.len());
    for _ in 0..20 {
        troop.round();
    }
    let mut touched_items: Vec<u32> = troop
        .monkeys
        .iter()
        .map(|m| m.borrow().handle_counter)
        .collect();
    touched_items.sort();
    let top_2_monkeys = touched_items.pop().unwrap() * touched_items.pop().unwrap();
    println!("Monkey business is {}", top_2_monkeys);
    Ok(())
}
