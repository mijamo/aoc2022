use nom::branch::alt;
use nom::character::complete::{line_ending, u32 as number};
use nom::multi::separated_list1;
use nom::{bytes::complete::tag, character::complete::alpha1};
use nom::{sequence::preceded, IResult};
use std::cell::{Ref, RefCell};
use std::collections::VecDeque;
use std::fs::File;
use std::hash::Hash;
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

fn combinations<T>(input: HashSet<T>, len: usize) -> Vec<Vec<Option<T>>>
where
    T: Copy + Eq + Hash,
{
    let mut results: Vec<Vec<Option<T>>> = Vec::new();
    if len == 0 {
        return Vec::new();
    }
    if len == 1 {
        if input.len() == 0 {
            return Vec::from([Vec::from([None])]);
        }
        return input.iter().map(|o| Vec::from([Some(*o)])).collect();
    }
    if input.len() == 0 {
        return Vec::from([(0..len).into_iter().map(|_| None).collect()]);
    }
    for v in input.iter() {
        let mut next_set = input.clone();
        next_set.remove(&v);
        for c in combinations(next_set, len - 1) {
            let start = Vec::from([Some(*v)]);
            results.push([start, c].concat());
        }
    }
    assert_eq!(results.iter().map(|r| r.len()).min().unwrap(), 2);
    return results;
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
            chosen: HashSet::new(),
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
    last_position: &'a str,
}

impl<'a> Actor<'a> {
    fn is_available(&self) -> bool {
        return self.remaining_time == 0 && self.destination.is_none();
    }
}

struct Scenario<'a> {
    actors: Vec<Actor<'a>>,
    arena: &'a Arena<'a>,
    flow: u32,
    pressure: u32,
    activated: HashSet<&'a str>,
    chosen: HashSet<&'a str>,
    turn: u32,
}

impl<'a> Scenario<'a> {
    fn eval(&mut self) -> u32 {
        self.turn += 1;
        self.pressure += self.flow;
        if self.turn == 27 {
            return self.pressure;
        }
        for actor in self.actors.iter_mut() {
            let mut next_actions: Vec<Option<&str>> = Vec::new();
            match (actor.remaining_time, actor.destination) {
                (1, Some(destination)) => {
                    self.activated.insert(destination);
                    self.flow += self.arena.get(destination).flow;
                    if self.activated.len() == self.arena.relevant_valves.len() + 1 {
                        return (27 - self.turn) * self.flow + self.pressure;
                    }
                    actor.remaining_time -= 1;
                    actor.last_position = destination;
                }
                (_, Some(_)) => {
                    actor.remaining_time -= 1;
                    next_actions.push(None);
                }
                (_, _) => {
                    next_actions.push(None);
                }
            }
        }
        let available_actors = self.actors.iter().filter(|a| a.remaining_time == 0).count();
        let next_moves: HashSet<&str> = self
            .arena
            .relevant_valves
            .difference(&self.chosen)
            .map(|r| *r)
            .collect();
        let possibilities = combinations(next_moves, available_actors);
        if possibilities.len() == 0 {
            return self.alternate(&Vec::new()).eval();
        }
        return possibilities
            .iter()
            .map(|c| self.alternate(c).eval())
            .max()
            .unwrap();
    }

    fn alternate(&self, next_moves: &Vec<Option<&'a str>>) -> Self {
        let mut new_actors = Vec::new();
        let mut next_moves = VecDeque::from_iter(next_moves.iter());
        let mut next_chosen = self.chosen.clone();
        for actor in self.actors.iter() {
            let mut new_actor = actor.clone();
            if new_actor.remaining_time == 0 {
                match next_moves.pop_front().unwrap() {
                    None => {
                        new_actor.destination = None;
                    }
                    Some(pos) => {
                        new_actor.remaining_time = self
                            .arena
                            .distances
                            .get(&(actor.last_position, pos))
                            .unwrap()
                            .clone()
                            + 1;
                        new_actor.destination = Some(pos);
                        next_chosen.insert(pos);
                    }
                }
            }
            new_actors.push(new_actor);
        }
        Self {
            actors: new_actors,
            arena: self.arena,
            flow: self.flow,
            pressure: self.pressure,
            activated: self.activated.clone(),
            turn: self.turn,
            chosen: next_chosen,
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
    let mut file = File::open("./src/input.txt").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, valves) = valves(&content).unwrap();
    println!("parsed {} valves", valves.len());
    let arena = Arena::new(&valves);
    println!("Generated the arena");
    let potential = arena.try_scenario(Vec::from([
        Actor {
            destination: Some("AA"),
            remaining_time: 1,
            last_position: "AA",
        },
        Actor {
            destination: Some("AA"),
            remaining_time: 1,
            last_position: "AA",
        },
    ]));
    println!("The maximum potential of valves is {}", potential);
}
