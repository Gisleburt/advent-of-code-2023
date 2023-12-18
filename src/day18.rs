use std::cmp::{max, min};
use std::ops::Add;

use derive_more::{Deref, DerefMut, From};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete;
use nom::character::complete::{newline, space1};
use nom::combinator::{map, map_res, value};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;
use num::abs;

use Direction::*;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn parse_direction(input: &str) -> IResult<&str, Direction> {
    alt((
        value(Up, complete::char('U')),
        value(Down, complete::char('D')),
        value(Left, complete::char('L')),
        value(Right, complete::char('R')),
    ))(input)
}

fn parse_direction_alt(input: &str) -> IResult<&str, Direction> {
    alt((
        value(Up, complete::char('3')),
        value(Down, complete::char('1')),
        value(Left, complete::char('2')),
        value(Right, complete::char('0')),
    ))(input)
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct AltInstruction {
    direction: Direction,
    distance: u64,
}

fn from_hex(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn parse_distance_alt(input: &str) -> IResult<&str, u64> {
    map_res(take_while_m_n(5, 5, is_hex_digit), from_hex)(input)
}

fn parse_alt_instruction(input: &str) -> IResult<&str, AltInstruction> {
    map(
        preceded(
            complete::char('#'),
            tuple((parse_distance_alt, parse_direction_alt)),
        ),
        |(distance, direction)| AltInstruction {
            distance,
            direction,
        },
    )(input)
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Instruction {
    direction: Direction,
    distance: u8,
    alt: AltInstruction,
}

fn parse_instruction(input: &str) -> IResult<&str, Instruction> {
    map(
        tuple((
            parse_direction,
            space1,
            complete::u8,
            space1,
            delimited(
                complete::char('('),
                parse_alt_instruction,
                complete::char(')'),
            ),
        )),
        |(direction, _, distance, _, alt_instruction)| Instruction {
            direction,
            distance,
            alt: alt_instruction,
        },
    )(input)
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Bounds {
    min: isize,
    max: isize,
}

impl Bounds {
    fn apply(self, num: isize) -> Self {
        Bounds {
            min: min(self.min, num),
            max: max(self.max, num),
        }
    }

    fn len(&self) -> usize {
        (self.max - self.min) as usize
    }
}

#[derive(Debug, Clone, PartialEq, From, Deref)]
struct Instructions(Vec<Instruction>);

impl Instructions {
    fn get_width_bounds(&self) -> Bounds {
        let mut width = 0_isize;
        self.iter()
            .filter_map(|instruction| match instruction.direction {
                Up => None,
                Down => None,
                Left => Some(0 - (instruction.distance as isize)),
                Right => Some(instruction.distance as isize),
            })
            .fold(Bounds::default(), |bounds: Bounds, num| {
                width += num;
                bounds.apply(width)
            })
    }

    fn get_height_bounds(&self) -> Bounds {
        let mut height = 0_isize;
        self.iter()
            .filter_map(|instruction| match instruction.direction {
                Up => Some(0 - (instruction.distance as isize)),
                Down => Some(instruction.distance as isize),
                Left => None,
                Right => None,
            })
            .fold(Bounds::default(), |bounds: Bounds, num| {
                height += num;
                bounds.apply(height)
            })
    }

    fn get_width_bounds_alt(&self) -> Bounds {
        let mut width = 0_isize;
        self.iter()
            .filter_map(|instruction| match instruction.alt.direction {
                Up => None,
                Down => None,
                Left => Some(0 - (instruction.alt.distance as isize)),
                Right => Some(instruction.alt.distance as isize),
            })
            .fold(Bounds::default(), |bounds: Bounds, num| {
                width += num;
                bounds.apply(width)
            })
    }

    fn get_height_bounds_alt(&self) -> Bounds {
        let mut height = 0_isize;
        self.iter()
            .filter_map(|instruction| match instruction.alt.direction {
                Up => Some(0 - (instruction.alt.distance as isize)),
                Down => Some(instruction.alt.distance as isize),
                Left => None,
                Right => None,
            })
            .fold(Bounds::default(), |bounds: Bounds, num| {
                height += num;
                bounds.apply(height)
            })
    }
}

fn parse_instructions(input: &str) -> IResult<&str, Instructions> {
    map(
        separated_list1(newline, parse_instruction),
        Instructions::from,
    )(input)
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
struct Tile {
    is_dug: bool,
}

#[derive(Debug, Clone, Deref, DerefMut)]
struct Grid {
    #[deref]
    #[deref_mut]
    grid: Vec<Vec<Tile>>,
    initial_start: Pos,
}

impl Grid {
    fn from(instructions: &Instructions) -> Self {
        let height = instructions.get_height_bounds();
        let width = instructions.get_width_bounds();

        Grid::with_bounds(height, width)
    }

    fn from_alt(instructions: &Instructions) -> Self {
        let height = instructions.get_height_bounds_alt();
        let width = instructions.get_width_bounds_alt();

        Grid::with_bounds(height, width)
    }

    fn with_bounds(height: Bounds, width: Bounds) -> Self {
        let initial_start = Pos {
            row: abs(height.min) as usize,
            col: abs(width.min) as usize,
        };

        let row = vec![Tile::default(); width.len() + 1];
        let grid = vec![row.clone(); height.len() + 1];

        Grid {
            grid,
            initial_start,
        }
    }

    fn height(&self) -> usize {
        self.len()
    }

    fn width(&self) -> usize {
        self[0].len()
    }

    fn dig_at(&mut self, pos: Pos) {
        self[pos.row][pos.col].is_dug = true;
    }

    fn dig_trench(&mut self, instructions: &[Instruction]) {
        let mut pos = self.initial_start;
        self.dig_at(pos);
        instructions.iter().for_each(|instruction| {
            for _ in 0..instruction.distance {
                pos = pos + instruction.direction;
                self.dig_at(pos)
            }
        })
    }

    fn dig_trench_alt(&mut self, instructions: &[Instruction]) {
        let mut pos = self.initial_start;
        self.dig_at(pos);
        instructions.iter().for_each(|instruction| {
            for _ in 0..instruction.alt.distance {
                pos = pos + instruction.alt.direction;
                self.dig_at(pos)
            }
        })
    }

    fn point_is_definitely_inside_trench(&self, pos: Pos) -> bool {
        let up: Vec<_> = self[..pos.row].iter().map(|row| &row[pos.col]).collect();
        let down: Vec<_> = self[pos.row..].iter().map(|row| &row[pos.col]).collect();
        let left: Vec<_> = self[pos.row][..pos.col].iter().collect();
        let right: Vec<_> = self[pos.row][pos.col..].iter().collect();

        for ray in [up, down, left, right] {
            let groups = ray
                .iter()
                .group_by(|tile| tile.is_dug)
                .into_iter()
                .filter(|(key, _tiles)| *key)
                .map(|(_key, tiles)| tiles.into_iter().collect_vec())
                .collect_vec();
            if groups.iter().any(|groups| groups.len() > 1) {
                continue;
            }
            return groups.len() % 2 == 1;
        }
        false
    }

    fn get_tile(&mut self, pos: Pos) -> &mut Tile {
        &mut self[pos.row][pos.col]
    }

    fn flood_fill(&mut self, pos: Pos) {
        if self.get_tile(pos).is_dug {
            return;
        }
        let width = self.width();
        let height = self.height();

        self.get_tile(pos).is_dug = true;
        // straight
        pos.up().into_iter().for_each(|up| self.flood_fill(up));
        pos.down(height)
            .into_iter()
            .for_each(|down| self.flood_fill(down));
        pos.left()
            .into_iter()
            .for_each(|left| self.flood_fill(left));
        pos.right(width)
            .into_iter()
            .for_each(|right| self.flood_fill(right));

        // diagonal, just in case
        pos.up()
            .into_iter()
            .filter_map(|up| up.left())
            .for_each(|up| self.flood_fill(up));
        pos.up()
            .into_iter()
            .filter_map(|up| up.right(width))
            .for_each(|up| self.flood_fill(up));
        pos.down(height)
            .into_iter()
            .filter_map(|down| down.left())
            .for_each(|down| self.flood_fill(down));
        pos.down(height)
            .into_iter()
            .filter_map(|down| down.right(width))
            .for_each(|down| self.flood_fill(down));
    }

    fn fill_trench(&mut self) {
        let to_dig = self
            .iter()
            .enumerate()
            .map(|(row, tiles)| {
                tiles
                    .iter()
                    .enumerate()
                    .map(move |(col, tile)| (Pos { row, col }, tile))
            })
            .flatten()
            .filter_map(|(pos, tile)| (!tile.is_dug).then_some(pos))
            .filter(|pos| self.point_is_definitely_inside_trench(*pos))
            .collect_vec();
        for pos in to_dig.into_iter() {
            self.flood_fill(pos);
        }
    }

    fn count_holes(&self) -> usize {
        self.iter()
            .flat_map(|row| row.iter().map(|tile| tile.is_dug))
            .filter(|is_dug| *is_dug)
            .count()
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
struct Pos {
    row: usize,
    col: usize,
}

impl Pos {
    fn up(&self) -> Option<Pos> {
        (self.row > 0).then_some(Pos {
            row: self.row.saturating_sub(1),
            col: self.col,
        })
    }

    fn down(&self, max: usize) -> Option<Pos> {
        (self.row + 1 < max).then_some(Pos {
            row: self.row + 1,
            col: self.col,
        })
    }

    fn left(&self) -> Option<Pos> {
        (self.col > 0).then_some(Pos {
            row: self.row,
            col: self.col.saturating_sub(1),
        })
    }

    fn right(&self, max: usize) -> Option<Pos> {
        (self.col + 1 < max).then_some(Pos {
            row: self.row,
            col: self.col + 1,
        })
    }
}

impl Add<Direction> for Pos {
    type Output = Pos;

    fn add(self, direction: Direction) -> Self::Output {
        match direction {
            Up => Pos {
                row: self.row - 1,
                col: self.col,
            },
            Down => Pos {
                row: self.row + 1,
                col: self.col,
            },
            Left => Pos {
                row: self.row,
                col: self.col - 1,
            },
            Right => Pos {
                row: self.row,
                col: self.col + 1,
            },
        }
    }
}

pub fn part1(input: &str) -> String {
    let instructions = parse_instructions(input).unwrap().1;
    let mut grid = Grid::from(&instructions);
    grid.dig_trench(&instructions);
    grid.fill_trench();
    grid.count_holes().to_string()
}

pub fn part2(input: &str) -> String {
    let instructions = parse_instructions(input).unwrap().1;
    let mut grid = Grid::from_alt(&instructions);
    grid.dig_trench_alt(&instructions);
    grid.fill_trench();
    grid.count_holes().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    mod direction {
        use super::*;

        #[test]
        fn test_parse_direction() {
            let input = "UDLR";
            let (remainder, direction) = parse_direction(input).unwrap();
            assert_eq!(direction, Up);
            assert_eq!(remainder, "DLR");
            let (remainder, direction) = parse_direction(remainder).unwrap();
            assert_eq!(direction, Down);
            assert_eq!(remainder, "LR");
            let (remainder, direction) = parse_direction(remainder).unwrap();
            assert_eq!(direction, Left);
            assert_eq!(remainder, "R");
            let (remainder, direction) = parse_direction(remainder).unwrap();
            assert_eq!(direction, Right);
            assert_eq!(remainder, "");
        }
    }

    mod alt_instruction {
        use super::*;

        #[test]
        fn test_parse_color() {
            let input = "#332211"; // intentional case change
            assert_eq!(
                parse_alt_instruction(input),
                Ok((
                    "",
                    AltInstruction {
                        direction: Down,
                        distance: 209441,
                    }
                ))
            );
        }
    }

    mod instruction {
        use super::*;

        #[test]
        fn test_parse_instruction() {
            let input = "L 2 (#002a22)";
            let instruction = parse_instruction(input).unwrap().1;
            assert_eq!(
                instruction,
                Instruction {
                    direction: Left,
                    distance: 2,
                    alt: AltInstruction {
                        direction: Left,
                        distance: 674,
                    },
                }
            )
        }

        #[test]
        fn test_parse_instructions() {
            let input = "R 6 (#70c710)
D 5 (#0dc571)";
            let instructions = parse_instructions(input).unwrap().1;
            assert_eq!(instructions.len(), 2);
        }
    }

    mod grid {
        use super::*;

        #[test]
        fn test_from_instructions() {
            let input = "R 6 (#000000)
D 5 (#000000)
L 2 (#000000)
D 2 (#000000)
";
            let instructions = parse_instructions(input).unwrap().1;
            let grid = Grid::from(&instructions);
            assert_eq!(grid.width(), 7);
            assert_eq!(grid.height(), 8);
        }

        #[test]
        fn test_dig_trench() {
            let input = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)
";
            let instructions = parse_instructions(input).unwrap().1;
            let mut grid = Grid::from(&instructions);
            grid.dig_trench(&instructions);
            assert_eq!(grid.count_holes(), 38);
        }

        #[test]
        fn test_fill_trench() {
            let input = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)
";
            let instructions = parse_instructions(input).unwrap().1;
            let mut grid = Grid::from(&instructions);
            grid.dig_trench(&instructions);
            grid.fill_trench();
            assert_eq!(grid.count_holes(), 62);
        }
    }

    #[test]
    fn test_part1() {
        let input = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)
";
        assert_eq!(part1(input), "62");
    }

    #[test]
    fn test_part2() {
        let input = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)
";
        assert_eq!(part2(input), "952408144115");
    }
}
