use std::cmp::{max, min};
use std::ops::Add;

use derive_more::{Deref, DerefMut, From};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while_m_n};
use nom::character::complete;
use nom::character::complete::{newline, space1};
use nom::combinator::{map, map_res, value};
use nom::multi::separated_list1;
use nom::sequence::{delimited, tuple};
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

#[derive(Debug, Copy, Clone, PartialEq)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

//// All parts of the color parser ripped from nom: docs https://docs.rs/nom/7.1.3/nom/index.html
fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn parse_hex(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

fn parse_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("#")(input)?;
    let (input, (red, green, blue)) = tuple((parse_hex, parse_hex, parse_hex))(input)?;

    Ok((input, Color { red, green, blue }))
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Instruction {
    direction: Direction,
    amount: u8,
    color: Color,
}

fn parse_instruction(input: &str) -> IResult<&str, Instruction> {
    map(
        tuple((
            parse_direction,
            space1,
            complete::u8,
            space1,
            delimited(complete::char('('), parse_color, complete::char(')')),
        )),
        |(direction, _, amount, _, color)| Instruction {
            direction,
            amount,
            color,
        },
    )(input)
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Bounds {
    min: i64,
    max: i64,
}

impl Bounds {
    fn apply(self, num: i64) -> Self {
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
        let mut width = 0_i64;
        self.iter()
            .filter_map(|instruction| match instruction.direction {
                Up => None,
                Down => None,
                Left => Some(0 - (instruction.amount as i64)),
                Right => Some(instruction.amount as i64),
            })
            .fold(Bounds::default(), |bounds: Bounds, num| {
                width += num;
                bounds.apply(width)
            })
    }

    fn get_height_bounds(&self) -> Bounds {
        let mut height = 0_i64;
        self.iter()
            .filter_map(|instruction| match instruction.direction {
                Up => Some(0 - (instruction.amount as i64)),
                Down => Some(instruction.amount as i64),
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
    color: Option<Color>,
    // edge_horizontal: bool,
    // edge_vertical: bool,
    // edge_left: bool,
    // edge_right: bool,
}

#[derive(Debug, Clone, Deref, DerefMut)]
struct Grid {
    #[deref]
    #[deref_mut]
    grid: Vec<Vec<Tile>>,
    initial_start: Pos,
}

impl Grid {
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

    fn dig_at(&mut self, pos: Pos, color: Option<Color>) {
        self[pos.row][pos.col].is_dug = true;
        if color.is_some() {
            self[pos.row][pos.col].color = color
        }
    }

    fn dig_trench(&mut self, instructions: &[Instruction]) {
        let mut pos = self.initial_start;
        self.dig_at(pos, Some(instructions.first().unwrap().color));
        instructions.iter().for_each(|instruction| {
            for _ in 0..instruction.amount {
                pos = pos + instruction.direction;
                self.dig_at(pos, Some(instruction.color))
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

impl From<&Instructions> for Grid {
    fn from(instructions: &Instructions) -> Self {
        let height = instructions.get_height_bounds();
        let width = instructions.get_width_bounds();

        Grid::with_bounds(height, width)
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

pub fn part2(_input: &str) -> String {
    todo!()
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

    mod color {
        use super::*;

        #[test]
        fn test_parse_color() {
            let input = "#fFc864"; // intentional case change
            assert_eq!(
                parse_color(input),
                Ok((
                    "",
                    Color {
                        red: 255,
                        green: 200,
                        blue: 100,
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
                    amount: 2,
                    color: Color {
                        red: 0,
                        green: 42,
                        blue: 34,
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

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "");
    }
}
