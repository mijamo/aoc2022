use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

struct HeightLayout {
    trees: Rc<Vec<Vec<i8>>>,
}

impl HeightLayout {
    fn new(trees: Vec<Vec<i8>>) -> Self {
        Self {
            trees: Rc::new(trees),
        }
    }

    fn width(&self) -> usize {
        return self.trees[0].len();
    }

    fn height(&self) -> usize {
        return self.trees.len();
    }

    fn spiral_iter(&self) -> Spiral<i8> {
        Spiral::<i8>::new(self.trees.clone())
    }

    fn iter_line<'a>(&'a self, i: usize) -> impl DoubleEndedIterator<Item = i8> + 'a {
        return self.trees[i].iter().map(|v| *v);
    }

    fn iter_column<'a>(&'a self, i: usize) -> impl DoubleEndedIterator<Item = i8> + 'a {
        return self.trees.iter().map(move |row| row[i].to_owned());
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

struct Spiral<T> {
    layout: Rc<Vec<Vec<T>>>,
    position: Option<[usize; 2]>,
    width: usize,
    height: usize,
    direction: Direction,
    cleared_lines: usize,
}

impl<T> Spiral<T> {
    fn new(layout: Rc<Vec<Vec<T>>>) -> Self {
        let width = layout[0].len();
        let height = layout.len();
        Self {
            direction: Direction::Right,
            layout: layout,
            position: Some([0, 0]),
            width,
            height,
            cleared_lines: 0,
        }
    }
}

struct CellValue<T> {
    position: [usize; 2],
    value: T,
}

impl<T: Copy> Iterator for Spiral<T> {
    type Item = CellValue<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.position {
            None => return None,
            Some([x, y]) => {
                let value = self.layout[y][x];
                let result = Some(CellValue::<T> {
                    position: [x, y],
                    value,
                });
                self.position = match (self.direction, x, y) {
                    (Direction::Right, x, y) if x < self.width - 1 - self.cleared_lines => {
                        Some([x + 1, y])
                    }
                    (Direction::Right, x, y) if y < self.height - 1 - self.cleared_lines => {
                        self.direction = Direction::Bottom;
                        Some([x, y + 1])
                    }
                    (Direction::Right, _, _) => None,
                    (Direction::Bottom, x, y) if y < self.height - 1 - self.cleared_lines => {
                        Some([x, y + 1])
                    }
                    (Direction::Bottom, x, y) if x > self.cleared_lines => {
                        self.direction = Direction::Left;
                        Some([x - 1, y])
                    }
                    (Direction::Bottom, _, _) => None,
                    (Direction::Left, x, y) if x > self.cleared_lines => Some([x - 1, y]),
                    (Direction::Left, x, y) if y > self.cleared_lines => {
                        self.direction = Direction::Top;
                        Some([x, y - 1])
                    }
                    (Direction::Left, _, _) => None,
                    (Direction::Top, x, y) if y > self.cleared_lines + 1 => Some([x, y - 1]),
                    (Direction::Top, x, y) if x < self.width - 1 - self.cleared_lines => {
                        self.cleared_lines += 1;
                        self.direction = Direction::Right;
                        Some([x + 1, y])
                    }
                    (Direction::Top, _, _) => None,
                };
                result
            }
        }
    }
}

struct VisibleIterator<'a> {
    tallest: Option<i8>,
    iterator: Box<dyn Iterator<Item = i8> + 'a>,
}

impl<'a, T> From<T> for VisibleIterator<'a>
where
    T: Iterator<Item = i8> + 'a,
{
    fn from(value: T) -> Self {
        Self {
            tallest: None,
            iterator: Box::new(value),
        }
    }
}

impl<'a> Iterator for VisibleIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.iterator.next(), self.tallest) {
            (None, _) => None,
            (Some(value), None) => {
                self.tallest = Some(value);
                Some(true)
            }
            (Some(value), Some(tallest)) if value > tallest => {
                self.tallest = Some(value);
                Some(true)
            }
            (_, _) => Some(false),
        }
    }
}

struct VisibleTrees {
    visible: Vec<Vec<bool>>,
}

impl VisibleTrees {
    fn from_iterators(mut iterators: Vec<VisibleIterator>) -> Self {
        Self {
            visible: iterators.iter_mut().map(|it| it.collect()).collect(),
        }
    }

    fn from_layout<'a>(layout: &'a HeightLayout, direction: Direction) -> Self {
        match direction {
            Direction::Bottom => {
                let iterators: Vec<VisibleIterator> = (0..layout.width())
                    .map(|i| VisibleIterator::from(layout.iter_column(i)))
                    .collect();
                Self::from_iterators(iterators).transpose()
            }
            Direction::Left => {
                let iterators: Vec<VisibleIterator> = (0..layout.height())
                    .map(|i| VisibleIterator::from(layout.iter_line(i).rev()))
                    .collect();
                Self::from_iterators(iterators).reverse_x()
            }
            Direction::Top => {
                let iterators: Vec<VisibleIterator> = (0..layout.width())
                    .map(|i| VisibleIterator::from(layout.iter_column(i).rev()))
                    .collect();
                Self::from_iterators(iterators).reverse_x().transpose()
            }
            Direction::Right => {
                let iterators: Vec<VisibleIterator> = (0..layout.height())
                    .map(|i| VisibleIterator::from(layout.iter_line(i)))
                    .collect();
                Self::from_iterators(iterators)
            }
        }
    }

    fn combine(&self, other: &Self) -> Self {
        let value = self
            .visible
            .iter()
            .zip(other.visible.iter())
            .map(|(left, right)| {
                left.into_iter()
                    .zip(right.into_iter())
                    .map(|(c1, c2)| *c1 || *c2)
                    .collect()
            })
            .collect();
        Self { visible: value }
    }

    fn transpose(&self) -> Self {
        let width = self.visible.len();
        let height = self.visible[0].len();
        let mut visible: Vec<Vec<bool>> = Vec::with_capacity(height);
        for y in 0..height {
            visible.push((0..width).map(|x| self.visible[x][y]).collect());
        }
        Self { visible }
    }

    fn reverse_x(&self) -> Self {
        let width = self.visible.len();
        let height = self.visible[0].len();
        let mut visible: Vec<Vec<bool>> = Vec::with_capacity(height);
        for y in 0..height {
            visible.push((0..width).map(|x| self.visible[y][width - x - 1]).collect());
        }
        Self { visible }
    }

    fn print(&self) {
        self.visible.iter().for_each(|line| {
            let line_str = line
                .iter()
                .map(|v| match v {
                    true => "T",
                    false => "F",
                })
                .collect::<Vec<&str>>()
                .join("");
            println!("{}", line_str);
        });
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("./src/input.txt")?;
    let lines = BufReader::new(file).lines();
    let layout = HeightLayout::new(
        lines
            .map(|l| {
                let content = l.unwrap();
                let heights: Vec<i8> = content
                    .chars()
                    .map(|c| i8::try_from(c.to_digit(10).unwrap()).unwrap())
                    .collect();
                return heights;
            })
            .collect::<Vec<Vec<i8>>>(),
    );
    let height = layout.height();
    let width = layout.width();
    let highest_vis = layout
        .trees
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(|(x, tree_height)| {
                    let mut pos_x = x;
                    let mut pos_y = y;
                    let mut top_score = 0;
                    let mut bottom_score = 0;
                    let mut left_score = 0;
                    let mut right_score = 0;
                    while pos_x < width - 1 {
                        right_score += 1;
                        pos_x += 1;
                        if layout.trees[y][pos_x] >= *tree_height {
                            break;
                        }
                    }
                    pos_x = x;
                    while pos_x > 0 {
                        left_score += 1;
                        pos_x -= 1;
                        if layout.trees[y][pos_x] >= *tree_height {
                            break;
                        }
                    }
                    pos_x = x;
                    while pos_y < height - 1 {
                        bottom_score += 1;
                        pos_y += 1;
                        if layout.trees[pos_y][x] >= *tree_height {
                            break;
                        }
                    }
                    pos_y = y;
                    while pos_y > 0 {
                        top_score += 1;
                        pos_y -= 1;
                        if layout.trees[pos_y][x] >= *tree_height {
                            break;
                        }
                    }
                    return top_score * bottom_score * left_score * right_score;
                })
                .max()
                .unwrap()
        })
        .max()
        .unwrap();
    println!("Largest scenary score is {}", highest_vis);
    Ok(())
}
