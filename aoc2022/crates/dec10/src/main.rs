use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::str::FromStr;

enum Instruction {
    Noop,
    AddX(i32),
}

impl Instruction {
    const fn cost(&self) -> u32 {
        match &self {
            Self::Noop => 1,
            Self::AddX(_) => 2,
        }
    }

    fn take_effect(&self, register: &mut Register) {
        match &self {
            Self::Noop => {}
            Self::AddX(value) => {
                register.value += value;
            }
        }
    }
}

struct Register {
    cycle: u32,
    value: i32,
}

struct Executer {
    cmd: Rc<Instruction>,
    ttc: u32,
}

enum State {
    InCycle,
    Rest,
}

struct Program {
    instructions: Vec<Rc<Instruction>>,
    register: Register,
    executer: Option<Executer>,
    completed: usize,
    state: State,
}

impl Program {
    fn new(instructions: Vec<Rc<Instruction>>) -> Self {
        Self {
            register: Register { cycle: 0, value: 1 },
            executer: None,
            completed: 0,
            instructions,
            state: State::Rest,
        }
    }

    fn start_cycle(&mut self) {
        self.register.cycle += 1;
        match self.executer {
            None => {
                let next_task = self.instructions[self.completed].clone();
                self.executer = Some(Executer {
                    cmd: next_task.clone(),
                    ttc: next_task.cost(),
                })
            }
            Some(_) => {}
        }
        self.state = State::InCycle;
    }

    fn end_cycle(&mut self) {
        match &mut self.executer {
            None => {}
            Some(executer) => match executer.ttc {
                1 => {
                    executer.cmd.take_effect(&mut self.register);
                    self.executer = None;
                    self.completed += 1;
                }
                _ => {
                    executer.ttc -= 1;
                }
            },
        }
        self.state = State::Rest
    }

    fn run_until(&mut self, target: u32) {
        let remaining_cycles = target - self.register.cycle;
        match self.state {
            State::InCycle => self.end_cycle(),
            _ => {}
        }
        for _ in 0..remaining_cycles - 1 {
            self.start_cycle();
            self.end_cycle();
        }
        self.start_cycle();
    }

    fn signal_strength(&self) -> i32 {
        return self.register.value * <u32 as TryInto<i32>>::try_into(self.register.cycle).unwrap();
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let mut instructions: Vec<Rc<Instruction>> = Vec::new();
    for line in lines {
        let content = line.unwrap();
        let command = &content[0..4];
        let instruction = match command {
            "noop" => Instruction::Noop,
            "addx" => Instruction::AddX(i32::from_str(&content[5..]).unwrap()),
            other => panic!("Unexpected command {}", other),
        };
        instructions.push(Rc::new(instruction));
    }
    let mut program = Program::new(instructions);
    let target_cycles: [u32; 6] = [20, 60, 100, 140, 180, 220];
    let total_strength: i32 = target_cycles
        .into_iter()
        .map(|step| {
            program.run_until(step);
            program.signal_strength()
        })
        .sum();
    println!(
        "Total signal strength for the targetted cycles: {}",
        total_strength
    );
    Ok(())
}
