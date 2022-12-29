use std::cmp::Ordering;
use std::fs::File;
use std::io::Read;

use nom::bytes::complete::tag;
use nom::character::complete::{char, multispace0, u32 as number};
use nom::multi::separated_list1;
use nom::sequence::{preceded, terminated};
use nom::IResult;

struct Clay(u32);
struct Ore(u32);
struct Obsidian(u32);
struct Geode(u32);

struct Blueprint {
    id: u32,
    ore_robot: Ore,
    clay_robot: Ore,
    obsidian_robot: (Ore, Clay),
    geode_robot: (Ore, Obsidian),
}

#[derive(Clone, Copy)]
enum ProdChoice {
    Clay,
    Ore,
    Obsidian,
    Geode,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Resources {
    clay: u32,
    ore: u32,
    obsidian: u32,
    geode: u32,
}

impl PartialOrd for Resources {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (
            self.clay.cmp(&other.clay),
            self.ore.cmp(&other.ore),
            self.obsidian.cmp(&other.obsidian),
            self.geode.cmp(&other.geode),
        ) {
            (Ordering::Equal, Ordering::Equal, Ordering::Equal, Ordering::Equal) => {
                Some(Ordering::Equal)
            }
            (Ordering::Less, Ordering::Less, Ordering::Less, Ordering::Less) => {
                Some(Ordering::Less)
            }
            (Ordering::Greater, Ordering::Greater, Ordering::Greater, Ordering::Greater) => {
                Some(Ordering::Greater)
            }
            _ => None,
        }
    }
}

impl Resources {
    fn can_afford_ore_robot(&self, blueprint: &Blueprint) -> bool {
        self.ore >= blueprint.ore_robot.0
    }

    fn can_afford_clay_robot(&self, blueprint: &Blueprint) -> bool {
        self.ore >= blueprint.clay_robot.0
    }

    fn can_afford_obsidian_robot(&self, blueprint: &Blueprint) -> bool {
        self.ore >= blueprint.obsidian_robot.0 .0 && self.clay >= blueprint.obsidian_robot.1 .0
    }

    fn can_afford_geode_robot(&self, blueprint: &Blueprint) -> bool {
        self.ore >= blueprint.geode_robot.0 .0 && self.obsidian >= blueprint.geode_robot.1 .0
    }

    fn take_ore_robot(&mut self, blueprint: &Blueprint) {
        self.ore -= blueprint.ore_robot.0;
    }

    fn take_clay_robot(&mut self, blueprint: &Blueprint) {
        self.ore -= blueprint.clay_robot.0;
    }

    fn take_obsidian_robot(&mut self, blueprint: &Blueprint) {
        self.ore -= blueprint.obsidian_robot.0 .0;
        self.clay -= blueprint.obsidian_robot.1 .0;
    }

    fn take_geode_robot(&mut self, blueprint: &Blueprint) {
        self.ore -= blueprint.geode_robot.0 .0;
        self.obsidian -= blueprint.geode_robot.1 .0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RobotsCount {
    clay: u32,
    ore: u32,
    obsidian: u32,
    geode: u32,
}

impl PartialOrd for RobotsCount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (
            self.clay.cmp(&other.clay),
            self.ore.cmp(&other.ore),
            self.obsidian.cmp(&other.obsidian),
            self.geode.cmp(&other.geode),
        ) {
            (Ordering::Equal, Ordering::Equal, Ordering::Equal, Ordering::Equal) => {
                Some(Ordering::Equal)
            }
            (Ordering::Less, Ordering::Less, Ordering::Less, Ordering::Less) => {
                Some(Ordering::Less)
            }
            (Ordering::Greater, Ordering::Greater, Ordering::Greater, Ordering::Greater) => {
                Some(Ordering::Greater)
            }
            _ => None,
        }
    }
}

struct Scenario<'a> {
    resources: Resources,
    robots: RobotsCount,
    blueprint: &'a Blueprint,
}

impl<'a> Scenario<'a> {
    fn new(blueprint: &'a Blueprint) -> Self {
        Self {
            resources: Resources {
                clay: 0,
                ore: 0,
                obsidian: 0,
                geode: 0,
            },
            robots: RobotsCount {
                clay: 0,
                ore: 1,
                obsidian: 0,
                geode: 0,
            },
            blueprint,
        }
    }

    fn next_move(&self, choice: ProdChoice) -> Self {
        let mut resources = self.resources.clone();
        let mut robots = self.robots.clone();
        resources.ore += robots.ore;
        resources.clay += robots.clay;
        resources.obsidian += robots.obsidian;
        resources.geode += robots.geode;
        match &choice {
            ProdChoice::Clay => {
                robots.clay += 1;
                resources.take_clay_robot(self.blueprint);
            }
            ProdChoice::Geode => {
                robots.geode += 1;
                resources.take_geode_robot(self.blueprint);
            }
            ProdChoice::Obsidian => {
                robots.obsidian += 1;
                resources.take_obsidian_robot(self.blueprint);
            }
            ProdChoice::Ore => {
                robots.ore += 1;
                resources.take_ore_robot(self.blueprint);
            }
            _ => {}
        };
        //println!("Resources: {:?}", resources);
        //println!("Robots: {:?}", robots);
        Self {
            robots,
            resources,
            blueprint: self.blueprint,
        }
    }

    fn has_enough_ore(&self) -> bool {
        self.robots.ore >= self.blueprint.clay_robot.0
            && self.robots.ore >= self.blueprint.obsidian_robot.0 .0
            && self.robots.ore >= self.blueprint.geode_robot.0 .0
    }

    fn has_enough_clay(&self) -> bool {
        self.robots.clay >= self.blueprint.obsidian_robot.1 .0
    }

    fn has_enough_obsidian(&self) -> bool {
        self.robots.obsidian >= self.blueprint.geode_robot.1 .0
    }

    fn max_potential(&self, remaining: u32) -> u32 {
        let first_geode = if self.robots.clay == 0 {
            remaining.checked_sub(3).or(Some(0)).unwrap()
        } else if self.robots.obsidian == 0 {
            remaining.checked_sub(2).or(Some(0)).unwrap()
        } else {
            remaining.checked_sub(1).or(Some(0)).unwrap()
        };
        self.resources.geode + self.robots.geode * remaining + first_geode * first_geode / 2
    }

    fn safe_potential(&self, remaining: u32) -> u32 {
        self.resources.geode + self.robots.geode * remaining
    }
}

impl<'a> PartialEq for Scenario<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.resources == other.resources && self.robots == other.robots;
    }
}

impl<'a> PartialOrd for Scenario<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (
            self.resources.partial_cmp(&other.resources),
            self.robots.partial_cmp(&other.robots),
        ) {
            (Some(Ordering::Equal), Some(Ordering::Equal)) => Some(Ordering::Equal),
            (Some(Ordering::Less), Some(Ordering::Less)) => Some(Ordering::Less),
            (Some(Ordering::Greater), Some(Ordering::Greater)) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

struct ScenarioTester<'a> {
    scenarios: Vec<Scenario<'a>>,
    blueprint: &'a Blueprint,
    turns: u32,
}

impl<'a> ScenarioTester<'a> {
    fn new(blueprint: &'a Blueprint) -> Self {
        Self {
            blueprint,
            scenarios: Vec::from([Scenario::new(blueprint)]),
            turns: 0,
        }
    }

    fn run_until(&mut self, minute: u32) -> u32 {
        while self.turns < minute {
            self.next_move(minute - self.turns - 1);
        }
        self.scenarios
            .iter()
            .map(|s| s.resources.geode)
            .max()
            .or(Some(0))
            .unwrap()
    }

    fn next_move(&mut self, remaining: u32) {
        let mut next_scenarios = Vec::new();
        //println!("turn {}", self.turns + 1);
        //println!("{} scenarios", self.scenarios.len());
        'scena: for (i, scenario) in self.scenarios.iter().enumerate() {
            for other in self.scenarios[i + 1..].iter() {
                if scenario < other {
                    continue;
                }
            }
            if scenario.resources.can_afford_geode_robot(self.blueprint) {
                next_scenarios.push(scenario.next_move(ProdChoice::Geode));
                continue 'scena;
            }
            let aff_ore = scenario.resources.can_afford_ore_robot(self.blueprint);
            let aff_clay = scenario.resources.can_afford_clay_robot(self.blueprint);
            let aff_obs = scenario.resources.can_afford_obsidian_robot(self.blueprint);
            let should_ore = !scenario.has_enough_ore();
            let should_clay = !scenario.has_enough_clay();
            let should_obs = !scenario.has_enough_obsidian();
            let prod_clay = scenario.robots.clay > 0;
            let prod_obs = scenario.robots.obsidian > 0;
            let mut should_wait = false;
            if should_ore {
                if aff_ore {
                    next_scenarios.push(scenario.next_move(ProdChoice::Ore));
                } else {
                    should_wait = true;
                }
            }
            if should_clay {
                if aff_clay {
                    next_scenarios.push(scenario.next_move(ProdChoice::Clay));
                } else {
                    should_wait = true;
                }
            }
            if should_obs {
                if aff_obs {
                    next_scenarios.push(scenario.next_move(ProdChoice::Obsidian));
                } else if prod_clay {
                    should_wait = true;
                }
            }
            if prod_obs {
                should_wait = true
            }
            if should_wait {
                next_scenarios.push(scenario.next_move(ProdChoice::None))
            }
        }
        let safe_potential = next_scenarios
            .iter()
            .map(|s| s.safe_potential(remaining))
            .max()
            .or(Some(9999999))
            .unwrap();
        next_scenarios = next_scenarios
            .into_iter()
            .filter(|s| s.max_potential(remaining) >= safe_potential)
            .collect();
        self.scenarios = next_scenarios;
        self.turns += 1;
    }
}

fn ore(input: &str) -> IResult<&str, Ore> {
    let (input, nb) = terminated(number, tag(" ore"))(input)?;
    Ok((input, Ore(nb)))
}

fn clay(input: &str) -> IResult<&str, Clay> {
    let (input, nb) = terminated(number, tag(" clay"))(input)?;
    Ok((input, Clay(nb)))
}

fn obsidian(input: &str) -> IResult<&str, Obsidian> {
    let (input, nb) = terminated(number, tag(" obsidian"))(input)?;
    Ok((input, Obsidian(nb)))
}

fn ore_robot(input: &str) -> IResult<&str, Ore> {
    preceded(tag("Each ore robot costs "), terminated(ore, char('.')))(input)
}

fn clay_robot(input: &str) -> IResult<&str, Ore> {
    preceded(tag("Each clay robot costs "), terminated(ore, char('.')))(input)
}

fn obsidian_robot(input: &str) -> IResult<&str, (Ore, Clay)> {
    let (input, ore) = preceded(tag("Each obsidian robot costs "), ore)(input)?;
    let (input, _) = tag(" and ")(input)?;
    let (input, clay) = clay(input)?;
    let (input, _) = char('.')(input)?;
    Ok((input, (ore, clay)))
}

fn geode_robot(input: &str) -> IResult<&str, (Ore, Obsidian)> {
    let (input, ore) = preceded(tag("Each geode robot costs "), ore)(input)?;
    let (input, _) = tag(" and ")(input)?;
    let (input, obsidian) = obsidian(input)?;
    let (input, _) = char('.')(input)?;
    Ok((input, (ore, obsidian)))
}

fn blueprint(input: &str) -> IResult<&str, Blueprint> {
    let (input, id) = preceded(tag("Blueprint "), number)(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, ore_robot) = ore_robot(input)?;
    let (input, _) = multispace0(input)?;
    let (input, clay_robot) = clay_robot(input)?;
    let (input, _) = multispace0(input)?;
    let (input, obsidian_robot) = obsidian_robot(input)?;
    let (input, _) = multispace0(input)?;
    let (input, geode_robot) = geode_robot(input)?;
    Ok((
        input,
        Blueprint {
            id,
            ore_robot,
            clay_robot,
            obsidian_robot,
            geode_robot,
        },
    ))
}

fn blueprints(input: &str) -> IResult<&str, Vec<Blueprint>> {
    separated_list1(multispace0, blueprint)(input)
}

fn main() {
    let mut content = String::new();
    let mut file = File::open("./src/test.txt").unwrap();
    file.read_to_string(&mut content).unwrap();
    let (_, blueprints) = blueprints(&content).unwrap();
    println!("Parsed {} blueprints", blueprints.len());
    let mut quality_level = 0;
    for blueprint in blueprints.iter() {
        let mut explorer = ScenarioTester::new(blueprint);
        let result = explorer.run_until(32);
        println!("Scenario {} gave {} geodes at most", blueprint.id, result);
        quality_level += result * blueprint.id;
    }
    println!("Total quality level is {}", quality_level);
}
