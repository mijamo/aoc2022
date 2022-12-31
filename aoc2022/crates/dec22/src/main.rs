use nom::branch::alt;
use nom::character::complete::{line_ending, multispace1, one_of, u32 as number};
use nom::combinator::map;
use nom::multi::{many1, separated_list1};
use nom::sequence::separated_pair;
use nom::IResult;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::{Add, Range};

#[derive(Debug, PartialEq)]
struct Pos {
    x: usize,
    y: usize,
}

impl Add<&(i32, i32)> for &Pos {
    type Output = Pos;

    fn add(self, rhs: &(i32, i32)) -> Self::Output {
        Pos {
            x: (self.x as i32 + rhs.0) as usize,
            y: (self.y as i32 + rhs.1) as usize,
        }
    }
}

#[derive(Debug)]
enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

impl Direction {
    fn score(&self) -> usize {
        match self {
            Self::Right => 0,
            Self::Bottom => 1,
            Self::Left => 2,
            Self::Top => 3,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Cell {
    Void,
    Open,
    Wall,
}

struct Bounds {
    x: Range<usize>,
    y: Range<usize>,
}

impl Bounds {
    fn contains(&self, pos: &Pos) -> bool {
        self.x.contains(&pos.x) && self.y.contains(&pos.y)
    }
}

struct Face {
    bounds: Bounds,
}

impl Face {
    fn at(&self, position: usize, from_dir: &Direction, direction: &Direction) -> Pos {
        let y = match (from_dir, direction) {
            (_, Direction::Bottom) => self.bounds.y.start,
            (_, Direction::Top) => self.bounds.y.end - 1,
            (Direction::Left, Direction::Left)
            | (Direction::Right, Direction::Right)
            | (Direction::Top, Direction::Right)
            | (Direction::Bottom, Direction::Left) => self.bounds.y.start + position,
            (Direction::Bottom, Direction::Right)
            | (Direction::Top, Direction::Left)
            | (Direction::Right, Direction::Left)
            | (Direction::Left, Direction::Right) => self.bounds.y.end - 1 - position,
        };
        let x = match (from_dir, direction) {
            (_, Direction::Right) => self.bounds.x.start,
            (_, Direction::Left) => self.bounds.x.end - 1,
            (Direction::Top, Direction::Top)
            | (Direction::Bottom, Direction::Bottom)
            | (Direction::Left, Direction::Bottom)
            | (Direction::Right, Direction::Top) => self.bounds.x.start + position,
            (Direction::Bottom, Direction::Top)
            | (Direction::Top, Direction::Bottom)
            | (Direction::Right, Direction::Bottom)
            | (Direction::Left, Direction::Top) => self.bounds.x.end - 1 - position,
        };
        Pos { x, y }
    }
}

struct Cube {
    faces: Vec<Face>,
}

impl Cube {
    fn new_test() -> Self {
        Self {
            faces: Vec::from([
                Face {
                    bounds: Bounds {
                        x: (9..13),
                        y: (1..5),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (1..5),
                        y: (5..9),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (5..9),
                        y: (5..9),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (9..13),
                        y: (5..9),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (13..17),
                        y: (9..13),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (9..13),
                        y: (9..13),
                    },
                },
            ]),
        }
    }

    fn new_input() -> Self {
        Self {
            faces: Vec::from([
                Face {
                    bounds: Bounds {
                        x: (51..101),
                        y: (1..51),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (1..51),
                        y: (151..201),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (1..51),
                        y: (101..151),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (51..101),
                        y: (51..101),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (101..151),
                        y: (1..51),
                    },
                },
                Face {
                    bounds: Bounds {
                        x: (51..101),
                        y: (101..151),
                    },
                },
            ]),
        }
    }

    fn cube_index(&self, pos: &Pos) -> Option<u8> {
        self.faces
            .iter()
            .position(|f| f.bounds.contains(pos))
            .and_then(|index| Some((index + 1) as u8))
    }

    fn get_face(&self, number: u8) -> &Face {
        self.faces.get(number as usize - 1).unwrap()
    }

    fn from(&self, from: &Pos, direction: &Direction) -> (Pos, Direction) {
        println!("Searching face for {:?}", from);
        let face_nb = self.cube_index(from).unwrap();
        let face = self.get_face(face_nb);
        let position = match direction {
            Direction::Bottom | Direction::Top => from.x - face.bounds.x.start,
            Direction::Left | Direction::Right => from.y - face.bounds.y.start,
        };
        let (destination_face_nb, new_direction) = match direction {
            Direction::Bottom => Self::normal_down(face_nb),
            Direction::Left => Self::normal_left(face_nb),
            Direction::Right => Self::normal_right(face_nb),
            Direction::Top => Self::normal_up(face_nb),
        };
        let destination_face = self.get_face(destination_face_nb);
        println!(
            "Moving from cube {:?} facing {:?} to cube {:?} facing {:?}",
            face_nb, direction, destination_face_nb, new_direction
        );
        (
            destination_face.at(position, direction, &new_direction),
            new_direction,
        )
    }

    fn normal_up(from: u8) -> (u8, Direction) {
        match from {
            1 => (2, Direction::Right),
            2 => (3, Direction::Top),
            3 => (4, Direction::Right),
            4 => (1, Direction::Top),
            5 => (2, Direction::Top),
            6 => (4, Direction::Top),
            _ => panic!("Invalid index"),
        }
    }

    fn normal_down(from: u8) -> (u8, Direction) {
        match from {
            1 => (4, Direction::Bottom),
            2 => (5, Direction::Bottom),
            3 => (2, Direction::Bottom),
            4 => (6, Direction::Bottom),
            5 => (4, Direction::Left),
            6 => (2, Direction::Left),
            _ => panic!("Invalid index"),
        }
    }

    fn normal_left(from: u8) -> (u8, Direction) {
        match from {
            1 => (3, Direction::Right),
            2 => (1, Direction::Bottom),
            3 => (1, Direction::Right),
            4 => (3, Direction::Bottom),
            5 => (1, Direction::Left),
            6 => (3, Direction::Left),
            _ => panic!("Invalid index"),
        }
    }

    fn normal_right(from: u8) -> (u8, Direction) {
        match from {
            1 => (5, Direction::Right),
            2 => (6, Direction::Top),
            3 => (6, Direction::Right),
            4 => (5, Direction::Top),
            5 => (6, Direction::Left),
            6 => (5, Direction::Left),
            _ => panic!("Invalid index"),
        }
    }
}

struct Blocked {}

struct World {
    cells: Vec<Cell>,
    cube: Cube,
    width: usize,
    height: usize,
    player_dir: Direction,
    player_pos: Pos,
}

impl World {
    fn new(cells: Vec<Vec<Cell>>) -> Self {
        let width = cells.iter().map(|a| a.len()).max().or(Some(0)).unwrap() + 2;
        let height = cells.len() + 2;
        let mut final_cells = Vec::with_capacity(width * height);
        final_cells.extend((0..width).map(|_| Cell::Void));
        for (y, line) in cells.iter().enumerate() {
            final_cells.push(Cell::Void);
            final_cells.extend_from_slice(line);
            final_cells.extend((0..(width - line.len() - 1)).map(|_| Cell::Void));
        }
        final_cells.extend((0..width).map(|_| Cell::Void));
        let first_x = final_cells.iter().position(|c| c == &Cell::Open).unwrap() - width;
        Self {
            width,
            height,
            cells: final_cells,
            player_dir: Direction::Right,
            player_pos: Pos { x: first_x, y: 1 },
            cube: Cube::new_input(),
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y.checked_mul(self.width)
            .or_else(|| panic!("Tried to get {}, {}", x, y))
            .unwrap()
            + x
    }

    fn at(&self, Pos { x, y }: &Pos) -> Option<&Cell> {
        self.cells.get(self.index(*x, *y))
    }

    fn move_to(&mut self, destination: Pos) -> Result<(), Blocked> {
        match self.at(&destination) {
            Some(Cell::Open) => {
                self.player_pos = destination;
                Ok(())
            }
            Some(Cell::Void) => {
                let (new_dest, new_dir) = self.cube.from(&self.player_pos, &self.player_dir);
                assert_ne!(new_dest, destination);
                self.move_to(new_dest).and_then(|_| {
                    self.player_dir = new_dir;
                    println!("Now at {:?} facing {:?}", self.player_pos, self.player_dir);
                    Ok(())
                })
            }
            Some(Cell::Wall) => Err(Blocked {}),
            _ => panic!("Unexpected destination"),
        }
    }

    fn move_player(&mut self, distance: usize) {
        for _ in 0..distance {
            let offset = match self.player_dir {
                Direction::Bottom => (0, 1),
                Direction::Left => (-1, 0),
                Direction::Top => (0, -1),
                Direction::Right => (1, 0),
            };
            let destination = &self.player_pos + &offset;
            println!("Moving to {:?}", destination);
            match self.move_to(destination) {
                Ok(_) => {}
                Err(_) => break,
            }
        }
    }

    fn turn_clockwise(&mut self) {
        self.player_dir = match self.player_dir {
            Direction::Left => Direction::Top,
            Direction::Top => Direction::Right,
            Direction::Right => Direction::Bottom,
            Direction::Bottom => Direction::Left,
        }
    }

    fn turn_counter_clockwize(&mut self) {
        self.player_dir = match self.player_dir {
            Direction::Left => Direction::Bottom,
            Direction::Top => Direction::Left,
            Direction::Right => Direction::Top,
            Direction::Bottom => Direction::Right,
        }
    }

    fn print(&self) {
        for y in 0..self.height {
            let line: String = (0..self.width)
                .map(|x| match self.cells[self.index(x, y)] {
                    Cell::Open => '.',
                    Cell::Void => ' ',
                    Cell::Wall => '#',
                })
                .collect();
            println!("{}", line);
        }
    }
}

#[derive(Debug)]
enum Action {
    Move(u32),
    TurnClockwise,
    TurnAntiClockwise,
}

fn layout_line(input: &str) -> IResult<&str, Vec<Cell>> {
    many1(map(one_of(" .#"), |c| match c {
        ' ' => Cell::Void,
        '.' => Cell::Open,
        '#' => Cell::Wall,
        _ => panic!("Unexpected char"),
    }))(input)
}

fn world(input: &str) -> IResult<&str, World> {
    let (input, lines) = separated_list1(line_ending, layout_line)(input)?;
    let world = World::new(lines);
    Ok((input, world))
}

fn move_action(input: &str) -> IResult<&str, Action> {
    map(number, |n| Action::Move(n))(input)
}

fn rotate_action(input: &str) -> IResult<&str, Action> {
    map(one_of("LR"), |c| match c {
        'R' => Action::TurnClockwise,
        'L' => Action::TurnAntiClockwise,
        _ => panic!("Unexpected character"),
    })(input)
}

fn directions(input: &str) -> IResult<&str, Vec<Action>> {
    many1(alt((move_action, rotate_action)))(input)
}

fn program(input: &str) -> IResult<&str, (World, Vec<Action>)> {
    separated_pair(world, multispace1, directions)(input)
}

fn main() {
    let mut file = File::open("./src/input.txt").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let (_, (mut world, actions)) = program(&content).unwrap();
    println!(
        "position {:?}, facing {:?}",
        world.player_pos, world.player_dir
    );
    for action in actions {
        println!();
        println!("{:?}", action);
        match action {
            Action::Move(distance) => world.move_player(distance as usize),
            Action::TurnAntiClockwise => world.turn_counter_clockwize(),
            Action::TurnClockwise => world.turn_clockwise(),
        }
        println!(
            "position {:?}, facing {:?}",
            world.player_pos, world.player_dir
        );
    }
    let final_pos = world.player_pos;
    let final_direction = world.player_dir;
    println!(
        "Final position {:?}, facing {:?}",
        final_pos, final_direction
    );
    let password = 1000 * (final_pos.y) + 4 * (final_pos.x) + final_direction.score();
    println!("Password is {}", password);
}
