use std::ops::Add;

use derive_more::{Deref, DerefMut, From};
use itertools::Itertools;

use Movement::*;

const MAX_STRAIGHT: usize = 3;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Movement {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
}

impl Movement {
    fn magnitude(&self) -> usize {
        match self {
            Up(magnitude) => *magnitude,
            Down(magnitude) => *magnitude,
            Left(magnitude) => *magnitude,
            Right(magnitude) => *magnitude,
        }
    }
}

impl Add for Movement {
    type Output = Movement;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Up(first), Up(second)) => Up(first + second),
            (Down(first), Down(second)) => Down(first + second),
            (Left(first), Left(second)) => Left(first + second),
            (Right(first), Right(second)) => Right(first + second),
            _ => rhs,
        }
    }
}

impl Default for Movement {
    fn default() -> Self {
        Up(0)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
struct Pos {
    row: usize,
    column: usize,
}

impl Pos {
    fn distance_to_goal(&self, goal: Pos) -> usize {
        (goal.row - self.row) + (goal.column - self.column)
    }

    fn movement_to(&self, to: Pos) -> Movement {
        if self.row == to.row {
            if self.column > to.column {
                return Left(self.column - to.column);
            }
            if to.column > self.column {
                return Left(to.column - self.column);
            }
        }
        if self.column == to.column {
            if self.row > to.row {
                return Up(self.row - to.row);
            }
            if to.row > self.row {
                return Down(to.row - self.row);
            }
        }
        panic!("Invalid movement from {self:?} to {to:?}");
    }
}

#[derive(Debug, Default, Clone, PartialEq, From, Deref, DerefMut)]
struct Grid(Vec<Vec<usize>>);

impl From<&str> for Grid {
    fn from(value: &str) -> Self {
        value
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| c as usize - '0' as usize) // Ha!
                    .collect_vec()
            })
            .collect_vec()
            .into()
    }
}

struct SmartGrid {
    grid: Grid,
    start: Pos,
    goal: Pos,
}

impl SmartGrid {
    fn least_cooling_path(&self) -> usize {
        let mut tree = Tree::default();
        let mut queue: Vec<&mut Node> = tree.edge_nodes();
        let mut found_goal: Option<&Node> = None;

        while let Some(node) = queue.pop() {
            queue.sort_by_key(|node| node.heat_loss)
        }

        found_goal.unwrap().heat_loss
    }

    fn order_nodes(&self, mut nodes: Vec<&Node>) {
        nodes.sort_by_key(|node| node.distance_to_goal(self.goal) + node.heat_loss);
    }

    fn height(&self) -> usize {
        self.grid.len()
    }

    fn width(&self) -> usize {
        self.grid[0].len()
    }
}

impl From<Grid> for SmartGrid {
    fn from(grid: Grid) -> Self {
        let goal = Pos {
            row: grid.len() - 1,
            column: grid[0].len() - 1,
        };
        Self {
            grid,
            start: Pos::default(),
            goal,
        }
    }
}

#[derive(Debug, Default, Clone, Deref, DerefMut)]
struct Tree(Node);

impl Tree {
    fn edge_nodes(&mut self) -> Vec<&mut Node> {
        // First lets do a search for all children we haven't checked over
        if self.children.is_none() {
            return vec![&mut self.0];
        };
        let mut queue = self.children.as_mut().unwrap().iter_mut().collect_vec();
        let mut edge_node = vec![];
        while let Some(child) = queue.pop() {
            if child.has_children() {
                queue.extend(child.children.as_mut().unwrap().iter_mut());
            } else {
                edge_node.push(child);
            }
        }
        edge_node
    }

    fn path_to(&self, pos: Pos) -> Vec<&Node> {
        let node_search: Vec<&Node> = Vec::new();
        let path: Vec<&Node> = Vec::new();
        todo!()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Node {
    pos: Pos,
    heat_loss: usize,
    children: Option<Vec<Node>>,
    recent_movement: Movement,
}

impl Node {
    fn id(&self) -> Pos {
        self.pos
    }

    fn distance_to_goal(&self, goal: Pos) -> usize {
        self.pos.distance_to_goal(goal)
    }

    fn has_children(&self) -> bool {
        self.children.is_some()
    }

    fn append_child(&mut self, pos: Pos, heat_loss: usize) -> &Node {
        let new_node = Node {
            pos,
            heat_loss: self.heat_loss + heat_loss,
            children: None,
            recent_movement: Default::default(),
        };

        let mut children = std::mem::replace(&mut self.children, None).unwrap_or_else(|| vec![]);
        if let Some((pos, _)) = children.iter().find_position(|node| node.pos == pos) {
            children[pos] = new_node;
            // Might need to recalculate children if we even come this way do lets panic for now
            todo!("Did not expect repeated child, more work to do")
        } else {
            children.push(new_node);
        }
        self.children = Some(children);
        self.children.as_ref().unwrap().last().unwrap()
    }

    fn find_child(&mut self, child: &Node) -> Option<&mut Node> {
        if self == child {
            Some(self)
        } else if let Some(children) = self.children.as_mut() {
            children.iter_mut().find(|node| node == &child)
        } else {
            None
        }
    }

    fn find_pos(&self, pos: Pos) -> Option<&Node> {
        if self.pos == pos {
            return Some(self);
        }

        if let Some(children) = self.children.as_ref() {
            return children
                .iter()
                .map(|node| node.find_pos(pos))
                .filter_map(|maybe_node| maybe_node)
                .sorted_by_key(|node| node.heat_loss)
                .next();
        }

        None
    }

    fn possible_next_positions(&self, grid: &SmartGrid) -> Vec<Pos> {
        match self.recent_movement {
            Up(x) => [
                if x < MAX_STRAIGHT {
                    self.possible_up(grid)
                } else {
                    None
                },
                self.possible_left(grid),
                self.possible_right(grid),
            ]
            .into_iter()
            .filter_map(|p| p)
            .collect(),
            Down(x) => [
                if x < MAX_STRAIGHT {
                    self.possible_down(grid)
                } else {
                    None
                },
                self.possible_left(grid),
                self.possible_right(grid),
            ]
            .into_iter()
            .filter_map(|p| p)
            .collect(),

            Left(x) => [
                if x < MAX_STRAIGHT {
                    self.possible_left(grid)
                } else {
                    None
                },
                self.possible_up(grid),
                self.possible_down(grid),
            ]
            .into_iter()
            .filter_map(|p| p)
            .collect(),

            Right(x) => [
                if x < MAX_STRAIGHT {
                    self.possible_right(grid)
                } else {
                    None
                },
                self.possible_up(grid),
                self.possible_down(grid),
            ]
            .into_iter()
            .filter_map(|p| p)
            .collect(),
        }
    }

    fn possible_up(&self, grid: &SmartGrid) -> Option<Pos> {
        (self.pos.row > 0).then_some(Pos {
            row: self.pos.row.saturating_sub(1),
            column: self.pos.column,
        })
    }

    fn possible_down(&self, grid: &SmartGrid) -> Option<Pos> {
        (self.pos.row < grid.height()).then_some(Pos {
            row: self.pos.row + 1,
            column: self.pos.column,
        })
    }

    fn possible_left(&self, grid: &SmartGrid) -> Option<Pos> {
        (self.pos.column > 0).then_some(Pos {
            row: self.pos.row,
            column: self.pos.column.saturating_sub(0),
        })
    }

    fn possible_right(&self, grid: &SmartGrid) -> Option<Pos> {
        (self.pos.column < grid.width()).then_some(Pos {
            row: self.pos.row,
            column: self.pos.column + 1,
        })
    }
}

pub fn part1(input: &str) -> String {
    let grid = SmartGrid::from(Grid::from(input));
    grid.least_cooling_path().to_string()
}

pub fn part2(_input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    mod grid {
        use super::*;

        #[test]
        fn test_from_str() {
            let input = "123
456
789";
            assert_eq!(
                Grid::from(input),
                Grid(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]])
            )
        }
    }

    #[ignore]
    #[test]
    fn test_part1() {
        let input = "2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533";
        assert_eq!(part1(input), "102");
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "");
    }
}
