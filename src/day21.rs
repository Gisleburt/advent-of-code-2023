use derive_more::{Deref, From};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::newline;
use nom::combinator::{into, value};
use nom::multi::{many1, separated_list1};
use nom::IResult;

use GardenFeature::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
struct Pos {
    row: usize,
    col: usize,
}

impl Pos {
    fn up(&self) -> Option<Self> {
        (self.row > 0).then_some(Pos {
            row: self.row - 1,
            col: self.col,
        })
    }

    fn down(&self, max: usize) -> Option<Self> {
        (self.row < max).then_some(Pos {
            row: self.row + 1,
            col: self.col,
        })
    }

    fn left(&self) -> Option<Self> {
        (self.col > 0).then_some(Pos {
            row: self.row,
            col: self.col - 1,
        })
    }

    fn right(&self, max: usize) -> Option<Self> {
        (self.col < max).then_some(Pos {
            row: self.row,
            col: self.col + 1,
        })
    }

    fn adjacent(&self, max_row: usize, max_col: usize) -> Vec<Pos> {
        [
            self.up(),
            self.down(max_row),
            self.left(),
            self.right(max_col),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
struct BigPos {
    row: isize,
    col: isize,
}

impl BigPos {
    fn up(&self) -> Self {
        BigPos {
            row: self.row - 1,
            col: self.col,
        }
    }

    fn down(&self) -> Self {
        BigPos {
            row: self.row + 1,
            col: self.col,
        }
    }

    fn left(&self) -> Self {
        BigPos {
            row: self.row,
            col: self.col - 1,
        }
    }

    fn right(&self) -> Self {
        BigPos {
            row: self.row,
            col: self.col + 1,
        }
    }

    fn adjacent(&self) -> Vec<BigPos> {
        vec![self.up(), self.down(), self.left(), self.right()]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GardenFeature {
    Start,
    Plot,
    Rock,
}

#[derive(Debug, Default, Clone, PartialEq, From, Deref)]
struct Map(Vec<Vec<GardenFeature>>);

impl Map {
    fn rows(&self) -> usize {
        self.len()
    }

    fn cols(&self) -> usize {
        self.get(0).map(|row| row.len()).unwrap_or(0)
    }

    fn get_start_pos(&self) -> Pos {
        self.iter()
            .enumerate()
            .find_map(|(row, row_data)| {
                row_data
                    .iter()
                    .enumerate()
                    .find_map(|(col, col_data)| (col_data == &Start).then_some(Pos { row, col }))
            })
            .unwrap()
    }

    fn is_not_rock(&self, pos: Pos) -> bool {
        self[pos.row][pos.col] != Rock
    }

    fn is_not_rock_infinite(&self, pos: BigPos) -> bool {
        let rows = self.rows() as isize;
        let cols = self.cols() as isize;
        let row = ((pos.row % rows) + rows) % rows;
        let col = ((pos.col % cols) + cols) % cols;
        self[row as usize][col as usize] != Rock
    }

    fn reachable_in_n_steps(&self, steps: usize) -> usize {
        let start = self.get_start_pos();
        let mut queue: Vec<Pos> = vec![start];

        for _ in 0..steps {
            let mut temp = vec![];
            while let Some(pos) = queue.pop() {
                temp.append(&mut pos.adjacent(self.rows() - 1, self.cols() - 1))
            }
            queue.extend(
                temp.into_iter()
                    .filter(|pos| self.is_not_rock(*pos))
                    .unique(),
            )
        }

        queue.len()
    }

    fn reachable_in_n_steps_infinite(&self, steps: usize) -> usize {
        let start = self.get_start_pos();
        let start = BigPos {
            row: start.row as isize,
            col: start.col as isize,
        };
        let mut queue: Vec<BigPos> = vec![start];
        let mut could_end_here: Vec<BigPos> = vec![];
        let steps_mod_2 = steps % 2;

        for step in 1..=steps {
            let could_end_this_tile = step % 2 == steps_mod_2;

            let mut temp = vec![];
            while let Some(pos) = queue.pop() {
                temp.append(&mut pos.adjacent())
            }

            let mut tiles = temp
                .into_iter()
                .filter(|pos| self.is_not_rock_infinite(*pos))
                .filter(|pos| !could_end_this_tile || !could_end_here.contains(pos))
                .unique()
                .collect_vec();
            if could_end_this_tile {
                could_end_here.extend(tiles.iter())
            }
            queue.append(&mut tiles)
        }

        could_end_here.len()
    }
}

fn parse_garden_feature(input: &str) -> IResult<&str, GardenFeature> {
    alt((
        value(Start, tag("S")),
        value(Plot, tag(".")),
        value(Rock, tag("#")),
    ))(input)
}

fn parse_garden_map(input: &str) -> IResult<&str, Map> {
    into(separated_list1(newline, many1(parse_garden_feature)))(input)
}

pub fn part1(input: &str) -> String {
    let map = parse_garden_map(input).unwrap().1;
    map.reachable_in_n_steps(64).to_string()
}

pub fn part2(input: &str) -> String {
    let map = parse_garden_map(input).unwrap().1;
    map.reachable_in_n_steps_infinite(26501365).to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    mod parsers {
        use super::*;

        #[test]
        fn test_parse_garden_feature() {
            assert_eq!(parse_garden_feature("S"), Ok(("", Start)));
            assert_eq!(parse_garden_feature("."), Ok(("", Plot)));
            assert_eq!(parse_garden_feature("#"), Ok(("", Rock)));
        }

        #[test]
        fn test_parse_garden_map() {
            let input = "...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........";
            let map = parse_garden_map(input).unwrap().1;
            assert_eq!(map.rows(), 11);
            assert_eq!(map.cols(), 11);
            assert_eq!(map.get_start_pos(), Pos { row: 5, col: 5 });
        }
    }

    #[test]
    fn test_part1() {
        let input = "...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........";
        // assert_eq!(part1(input), "");
        let map = parse_garden_map(input).unwrap().1;
        assert_eq!(map.reachable_in_n_steps(6), 16)
    }

    #[test]
    fn test_part2() {
        let input = "...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........";
        // assert_eq!(part2(input), "");
        let map = parse_garden_map(input).unwrap().1;
        assert_eq!(map.reachable_in_n_steps_infinite(50), 1594)
    }
}
