use nom::branch::alt;
use nom::character::complete::{line_ending, multispace1, one_of, u32 as number};
use nom::combinator::map;
use nom::multi::{many1, separated_list1};
use nom::sequence::separated_pair;
use nom::IResult;
use std::fs::File;
use std::io::Read;
use std::ops::Add;

#[derive(Debug)]
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

struct Blocked {}

struct World {
    cells: Vec<Cell>,
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
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y.checked_mul(self.width)
            .or_else(|| panic!("Tried to get {}, {}", x, y))
            .unwrap()
            + x
    }

    fn first_in_line(&self, y: usize) -> Pos {
        let x = self.cells[y * self.width..(y + 1) * self.width]
            .iter()
            .position(|c| c != &Cell::Void)
            .unwrap();
        Pos { x, y }
    }

    fn last_in_line(&self, y: usize) -> Pos {
        let x = self.cells[y * self.width..(y + 1) * self.width]
            .iter()
            .rev()
            .position(|c| c != &Cell::Void)
            .unwrap();
        Pos {
            x: self.width - 1 - x,
            y,
        }
    }

    fn first_in_column(&self, x: usize) -> Pos {
        let mut y = 1;
        loop {
            let pos = Pos { x, y };
            match self.at(&pos) {
                Some(Cell::Open | Cell::Wall) => return pos,
                _ => {}
            }
            y += 1;
        }
    }

    fn last_in_column(&self, x: usize) -> Pos {
        let mut y = self.height;
        loop {
            let pos = Pos { x, y };
            match self.at(&pos) {
                Some(Cell::Open | Cell::Wall) => return pos,
                _ => {}
            }
            y -= 1;
        }
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
            Some(Cell::Void) => match self.player_dir {
                Direction::Right => self.move_to(self.first_in_line(destination.y)),
                Direction::Left => self.move_to(self.last_in_line(destination.y)),
                Direction::Bottom => self.move_to(self.first_in_column(destination.x)),
                Direction::Top => self.move_to(self.last_in_column(destination.x)),
            },
            Some(Cell::Wall) => Err(Blocked {}),
            _ => panic!("Unexpected destination"),
        }
    }

    fn move_player(&mut self, distance: usize) {
        let offset = match self.player_dir {
            Direction::Bottom => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Top => (0, -1),
            Direction::Right => (1, 0),
        };
        for _ in 0..distance {
            let destination = &self.player_pos + &offset;
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
