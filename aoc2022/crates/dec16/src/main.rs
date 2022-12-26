use nom::branch::alt;
use nom::character::complete::{line_ending, u32 as number};
use nom::multi::separated_list1;
use nom::{bytes::complete::tag, character::complete::alpha1};
use nom::{sequence::preceded, IResult};
use std::cell::{Ref, RefCell};
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

struct PathExplorer<'a> {
    distances: HashMap<&'a str, u32>,
}

impl<'a> PathExplorer<'a> {
    fn new(origin: &InnerValve<'a>) -> Self {
        Self {
            distances: HashMap::from([(origin.borrow().id, 0)]),
        }
    }

    fn calculate(&mut self, from: InnerValve<'a>, distance: u32) -> HashMap<&'a str, u32> {
        let from_valve = from.borrow();
        let distance = distance + 1;
        let mut next_origins = Vec::new();
        for valve_rc in from_valve.next.iter() {
            let valve = valve_rc.borrow();
            match self.distances.get(valve.id) {
                Some(res) if res <= &distance => continue,
                _ => {}
            }
            println!("Distance to {}: {}", valve.id, distance);
            self.distances.insert(valve.id, distance);
            next_origins.push(valve_rc);
        }
        for next in next_origins {
            self.calculate(next.clone(), distance);
        }
        return self.distances.clone();
    }
}

fn combinations<T>(input: &Vec<Vec<Option<T>>>) -> Vec<Vec<Option<T>>>
where
    T: Copy + Eq,
{
    let mut combinations: Vec<Vec<Option<T>>> = Vec::from([Vec::new()]);
    for actor in input {
        let mut next_combinations = Vec::new();
        for combination in combinations.iter_mut() {
            for position in actor {
                let mut next_combination = combination.clone();
                if combination.iter().any(|m| m == position) {
                    next_combination.push(None);
                } else {
                    next_combination.push(*position);
                }
                next_combinations.push(next_combination);
            }
        }
        combinations = next_combinations;
    }
    return combinations;
}

struct Arena<'a> {
    distances: HashMap<(&'a str, &'a str), u32>,
    valves: HashMap<&'a str, InnerValve<'a>>,
    relevant_valves: HashSet<&'a str>,
}

impl<'a> Arena<'a> {
    fn new(valves: &'a Vec<InnerValve<'a>>) -> Self {
        let mut distances: HashMap<(&str, &str), u32> = HashMap::new();
        for valve in valves.iter() {
            let id = valve.borrow().id;
            println!("DISTANCES FROM {}", id);
            for (to, distance) in PathExplorer::new(valve).calculate(valve.clone(), 0).iter() {
                distances.insert((id, to), *distance);
            }
        }
        Self {
            distances,
            valves: valves.iter().map(|v| (v.borrow().id, v.clone())).collect(),
            relevant_valves: valves
                .iter()
                .filter(|v| v.borrow().flow > 0)
                .map(|m| m.borrow().id)
                .collect(),
        }
    }

    fn try_scenario(&'a self, actors: Vec<Actor<'a>>) -> u32 {
        Scenario {
            actors,
            arena: &self,
            flow: 0,
            pressure: 0,
            activated: HashSet::new(),
            turn: 0,
        }
        .eval()
    }

    fn get(&self, id: &str) -> Ref<Valve<'a>> {
        self.valves.get(id).unwrap().borrow()
    }
}

#[derive(Clone, Copy)]
struct Actor<'a> {
    destination: Option<&'a str>,
    remaining_time: u32,
}

struct Scenario<'a> {
    actors: Vec<Actor<'a>>,
    arena: &'a Arena<'a>,
    flow: u32,
    pressure: u32,
    activated: HashSet<&'a str>,
    turn: u32,
}

impl<'a> Scenario<'a> {
    fn eval(&mut self) -> u32 {
        self.turn += 1;
        self.pressure += self.flow;
        if self.turn == 31 {
            return self.pressure;
        }
        let mut next_moves: Vec<Vec<Option<&str>>> = Vec::new();
        for actor in self.actors.iter_mut() {
            let mut next_actions: Vec<Option<&str>> = Vec::new();
            match (actor.remaining_time, actor.destination) {
                (1, Some(destination)) => {
                    self.activated.insert(destination);
                    self.flow += self.arena.get(destination).flow;
                    for remaining in self.arena.relevant_valves.difference(&self.activated) {
                        next_actions.push(Some(&remaining));
                    }
                    if next_actions.len() == 0 {
                        // reached and activated the last valve
                        actor.destination = None;
                        actor.remaining_time = 0;
                        next_actions.push(None);
                    }
                }
                (_, Some(_)) => {
                    actor.remaining_time -= 1;
                    next_actions.push(None);
                }
                (_, _) => {
                    next_actions.push(None);
                }
            }
            next_moves.push(next_actions);
        }
        return combinations(&next_moves)
            .iter()
            .map(|c| self.alternate(c).eval())
            .max()
            .unwrap();
    }

    fn alternate(&self, next_moves: &Vec<Option<&'a str>>) -> Self {
        let mut new_actors = Vec::new();
        for (i, next_position) in next_moves.iter().enumerate() {
            let mut actor = self.actors.get(i).unwrap().clone();
            match next_position {
                None => {}
                Some(pos) => {
                    actor.remaining_time = self
                        .arena
                        .distances
                        .get(&(actor.destination.unwrap(), pos))
                        .unwrap()
                        .clone()
                        + 1;
                    actor.destination = Some(pos);
                }
            }
            new_actors.push(actor);
        }
        Self {
            actors: new_actors,
            arena: self.arena,
            flow: self.flow,
            pressure: self.pressure,
            activated: self.activated.clone(),
            turn: self.turn,
        }
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
    let mut file = File::open("./src/test.txt").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, valves) = valves(&content).unwrap();
    println!("parsed {} valves", valves.len());
    let arena = Arena::new(&valves);
    println!("Generated the arena");
    let potential = arena.try_scenario(Vec::from([Actor {
        destination: Some("AA"),
        remaining_time: 1,
    }]));
    println!("The maximum potential of valves is {}", potential);
}
