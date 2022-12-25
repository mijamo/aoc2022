use nom::branch::alt;
use nom::character::complete::{line_ending, u32 as number};
use nom::multi::separated_list1;
use nom::{bytes::complete::tag, character::complete::alpha1};
use nom::{sequence::preceded, IResult};
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

type InnerValve<'a> = Rc<RefCell<Valve<'a>>>;

struct Valve<'a> {
    id: &'a str,
    flow: u32,
    next: Vec<InnerValve<'a>>,
}

impl<'a> Valve<'a> {
    fn potential_if_move(
        &self,
        remaining_time: u32,
        already_opened: &HashSet<&'a str>,
        since_last_open: &HashSet<&'a str>,
    ) -> u32 {
        if remaining_time == 0 {
            return 0;
        }
        self.next
            .iter()
            .filter(|v| !since_last_open.contains(v.borrow().id))
            .map(|v| {
                let v = v.borrow();
                let mut new_since_last_open = since_last_open.clone();
                new_since_last_open.insert(v.id);
                v.potential(remaining_time - 1, already_opened, &new_since_last_open)
            })
            .max()
            .or(Some(0))
            .unwrap()
    }

    fn potential_if_open(&self, remaining_time: u32, already_opened: &HashSet<&'a str>) -> u32 {
        if remaining_time == 0 {
            return 0;
        }
        let mut newly_opened = already_opened.clone();
        newly_opened.insert(self.id);
        self.flow * (remaining_time - 1)
            + self.potential_if_move(remaining_time - 1, &newly_opened, &HashSet::from([self.id]))
    }

    fn potential(
        &self,
        remaining_time: u32,
        already_opened: &HashSet<&'a str>,
        since_last_open: &HashSet<&'a str>,
    ) -> u32 {
        let potential_if_move =
            self.potential_if_move(remaining_time, &already_opened, since_last_open);
        if already_opened.contains(self.id) || self.flow == 0 {
            return potential_if_move;
        }
        u32::max(
            potential_if_move,
            self.potential_if_open(remaining_time, &already_opened),
        )
    }
}

fn valve(input: &str) -> IResult<&str, (InnerValve, Vec<&str>)> {
    let (input, id) = preceded(tag("Valve "), alpha1)(input)?;
    let (input, flow) = preceded(tag(" has flow rate="), number)(input)?;
    let (input, _) = alt((
        tag("; tunnels lead to valves "),
        tag("; tunnel leads to valve "),
    ))(input)?;
    let (input, destinations) = separated_list1(tag(", "), alpha1)(input)?;
    Ok((
        input,
        (
            Rc::new(RefCell::new(Valve {
                flow,
                id,
                next: Vec::new(),
            })),
            destinations,
        ),
    ))
}

fn valves(input: &str) -> IResult<&str, Vec<InnerValve>> {
    let (_, valve_data) = separated_list1(line_ending, valve)(input)?;
    let mut valves = Vec::new();
    let mut valves_by_id: HashMap<&str, InnerValve> = HashMap::new();
    for (valve, _) in valve_data.iter() {
        valves_by_id.insert(valve.borrow().id, valve.clone());
        valves.push(valve.clone());
    }
    for (valve, destinations) in valve_data.iter() {
        valve.borrow_mut().next = destinations
            .into_iter()
            .map(|id| valves_by_id.get(id).unwrap().clone())
            .collect();
    }
    Ok((input, valves))
}

fn main() {
    let mut file = File::open("./src/input.txt").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, valves) = valves(&content).unwrap();
    println!("parsed {} valves", valves.len());
    let aa = valves
        .iter()
        .find(|v| v.borrow().id == "AA")
        .unwrap()
        .borrow();
    let potential = aa
        .next
        .iter()
        .map(|v| {
            v.borrow()
                .potential(29, &HashSet::new(), &HashSet::from([aa.id]))
        })
        .max()
        .unwrap();
    println!("The maximum potential of valves is {}", potential);
}
