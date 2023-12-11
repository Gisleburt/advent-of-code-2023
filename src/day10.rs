use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::{char, newline};
use nom::combinator::{map, value};
use nom::multi::{many1, separated_list1};
use nom::IResult;
use num::Integer;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use Direction::*;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq, Ord)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq, Ord)]
enum Pipe {
    NS,
    EW,
    NE,
    NW,
    SW,
    SE,
    Ground,
    Start,
}

impl Display for Pipe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl Pipe {
    fn is_start(&self) -> bool {
        self == &Self::Start
    }

    fn is_ground(&self) -> bool {
        self == &Self::Ground
    }

    fn is_nw_edge(&self) -> bool {
        match self {
            Pipe::NS => true,
            Pipe::EW => true,
            Pipe::NE => false,
            Pipe::NW => true,
            Pipe::SW => false,
            Pipe::SE => true,
            Pipe::Ground => false,
            Pipe::Start => true, // guess that its not a `-`
        }
    }

    // The direction you face once exiting
    fn get_exit_direction(&self, arrive_from: Direction) -> Option<Direction> {
        match self {
            // |
            Pipe::NS => match arrive_from {
                North => Some(North),
                South => Some(South),
                _ => None,
            },
            // -
            Pipe::EW => match arrive_from {
                East => Some(East),
                West => Some(West),
                _ => None,
            },
            // L
            Pipe::NE => match arrive_from {
                West => Some(North),
                South => Some(East),
                _ => None,
            },
            // J
            Pipe::NW => match arrive_from {
                South => Some(West),
                East => Some(North),
                _ => None,
            },
            // 7
            Pipe::SW => match arrive_from {
                East => Some(South),
                North => Some(West),
                _ => None,
            },
            // F
            Pipe::SE => match arrive_from {
                West => Some(South),
                North => Some(East),
                _ => None,
            },
            Pipe::Ground => None,
            Pipe::Start => None,
        }
    }

    fn as_char(&self) -> char {
        match self {
            Self::NS => '|',
            Self::EW => '-',
            Self::NE => 'L',
            Self::NW => 'J',
            Self::SW => '7',
            Self::SE => 'F',
            Self::Ground => '.',
            Self::Start => 'S',
        }
    }
}

#[derive(Debug, Clone)]
struct PipeMap(Vec<Vec<Pipe>>);

impl PipeMap {
    fn get_start(&self) -> Point {
        self.0
            .iter()
            .enumerate()
            .find_map(|(row, pipes)| {
                pipes.iter().enumerate().find_map(|(column, pipe)| {
                    pipe.is_start().then_some(Some(Point { row, column }))
                })
            })
            .unwrap()
            .unwrap()
    }

    fn pipe_at_point(&self, point: Point) -> Pipe {
        self.0[point.row][point.column]
    }

    fn next_point_and_direction(
        &self,
        current_point: Point,
        direction: Direction,
    ) -> (Option<Point>, Option<Direction>) {
        let Some(next_point) = current_point.next_point(direction) else {
            return (None, None);
        };
        let next_pipe = self.pipe_at_point(next_point);
        let next_direction = next_pipe.get_exit_direction(direction);
        (Some(next_point), next_direction)
    }

    fn path_to_start(&self, point: Point, dir: Direction) -> Option<Vec<Point>> {
        let mut path = Vec::with_capacity(self.0.len() * self.0[0].len()); // Worst case

        // Shadow
        let mut point = point;
        let mut dir = dir;

        loop {
            let (next_point, next_dir) = self.next_point_and_direction(point, dir);
            // Check if the pipe has ended
            let Some(next_point) = next_point else {
                return None;
            };
            let pipe = self.pipe_at_point(next_point);
            path.push(next_point);
            if pipe.is_start() {
                return Some(path);
            }
            // Check if there is somewhere to go next
            let Some(next_dir) = next_dir else {
                return None;
            };
            point = next_point;
            dir = next_dir;
        }
    }

    fn get_shortest_path(&self) -> Vec<Point> {
        let start = self.get_start();
        let mut paths: Vec<_> = [North, South, East, West]
            .into_iter()
            .filter_map(|dir| self.path_to_start(start, dir))
            .collect();
        paths.sort_by_key(|path| path.len());
        paths.remove(0)
    }

    fn remove_all_but_path(&self, path: Vec<Point>) -> PipeMap {
        PipeMap(
            self.0
                .iter()
                .enumerate()
                .map(|(row, pipes)| {
                    pipes
                        .iter()
                        .enumerate()
                        .map(|(column, pipe)| {
                            if path.contains(&Point { row, column }) {
                                *pipe
                            } else {
                                Pipe::Ground
                            }
                        })
                        .collect()
                })
                .collect(),
        )
    }

    fn count_pipes_nw(&self, point: &Point) -> usize {
        let Point {
            mut row,
            mut column,
        } = point;
        let mut count = 0;
        while row > 0 && column > 0 {
            row -= 1;
            column -= 1;
            let next = Point::new(row, column);
            if self.pipe_at_point(next).is_nw_edge() {
                count += 1;
            }
        }
        count
    }

    fn n_points_inside_pipes(&self) -> usize {
        // We'll simple find each ground point, then run to the left edge and see how many times
        // it crossed a pipe. Note, this only works if there's only one specific
        self.0
            .iter()
            .enumerate()
            .flat_map(|(row, pipes)| {
                pipes
                    .iter()
                    .enumerate()
                    .filter(|(_column, pipe)| pipe.is_ground())
                    .map(|(column, _pipe)| Point { row, column })
                    .collect::<Vec<_>>()
            })
            .filter(|point| self.count_pipes_nw(point).is_odd())
            .count()
    }
}

impl From<Vec<Vec<Pipe>>> for PipeMap {
    fn from(value: Vec<Vec<Pipe>>) -> Self {
        Self(value)
    }
}

impl Deref for PipeMap {
    type Target = Vec<Vec<Pipe>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for PipeMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string: String = self
            .0
            .iter()
            .map(|row| row.iter().map(|pipe| pipe.as_char()).collect::<String>())
            .join("\n");

        write!(f, "{string}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Point {
    row: usize,
    column: usize,
}

impl Point {
    fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }

    fn next_point(self, direction: Direction) -> Option<Point> {
        let Self {
            mut row,
            mut column,
        } = self;

        match direction {
            North => {
                if row == 0 {
                    return None;
                }
                row -= 1
            }
            South => row += 1,
            East => column += 1,
            West => {
                if column == 0 {
                    return None;
                }
                column -= 1
            }
        };

        Some(Self { row, column })
    }
}

fn parse_pipe(input: &str) -> IResult<&str, Pipe> {
    alt((
        value(Pipe::NS, char('|')),
        value(Pipe::EW, char('-')),
        value(Pipe::NE, char('L')),
        value(Pipe::NW, char('J')),
        value(Pipe::SW, char('7')),
        value(Pipe::SE, char('F')),
        value(Pipe::Ground, char('.')),
        value(Pipe::Start, char('S')),
    ))(input)
}

fn parse_row(input: &str) -> IResult<&str, Vec<Pipe>> {
    many1(parse_pipe)(input)
}

fn parse_pipe_map(input: &str) -> IResult<&str, PipeMap> {
    map(separated_list1(newline, parse_row), |pipes| pipes.into())(input)
}

pub fn part1(input: &str) -> String {
    let pipe_map = parse_pipe_map(input).unwrap().1;
    pipe_map.get_shortest_path().len().div_ceil(2).to_string()
}

pub fn part2(input: &str) -> String {
    let pipe_map = parse_pipe_map(input).unwrap().1;
    let path = pipe_map.get_shortest_path();
    let new_map = pipe_map.remove_all_but_path(path);
    new_map.n_points_inside_pipes().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    mod parts {
        use super::*;

        // -L|F7
        // 7S-7|
        // L|7||
        // -L-J|
        // L|-JF
        fn helper_create_pipe_map_1() -> PipeMap {
            let input = "-L|F7
7S-7|
L|7||
-L-J|
L|-JF";
            parse_pipe_map(input).unwrap().1
        }

        // 7-F7-
        // .FJ|7
        // SJLL7
        // |F--J
        // LJ.LJ
        fn helper_create_pipe_map_2() -> PipeMap {
            let input = "7-F7-
.FJ|7
SJLL7
|F--J
LJ.LJ";
            parse_pipe_map(input).unwrap().1
        }

        // .....
        // .S-7.
        // .|.|.
        // .L-J.
        // .....
        fn helper_create_simple_pipe_map() -> PipeMap {
            let input = ".....
.S-7.
.|.|.
.L-J.
.....";
            parse_pipe_map(input).unwrap().1
        }

        #[test]
        fn test_parser() {
            let pipe_map = helper_create_pipe_map_1();
            assert_eq!(pipe_map.len(), 5);
            assert!(pipe_map.iter().all(|row| row.len() == 5));

            let pipe_map = helper_create_pipe_map_2();
            assert_eq!(pipe_map.len(), 5);
            assert!(pipe_map.iter().all(|row| row.len() == 5));

            let pipe_map = helper_create_simple_pipe_map();
            assert_eq!(pipe_map.len(), 5);
            assert!(pipe_map.iter().all(|row| row.len() == 5));
        }

        #[test]
        fn test_find_start() {
            let pipe_map = helper_create_pipe_map_1();
            assert_eq!(pipe_map.get_start(), Point { row: 1, column: 1 });

            let pipe_map2 = helper_create_pipe_map_2();
            assert_eq!(pipe_map2.get_start(), Point { row: 2, column: 0 });

            let pipe_map2 = helper_create_simple_pipe_map();
            assert_eq!(pipe_map2.get_start(), Point { row: 1, column: 1 });
        }

        #[test]
        fn test_next_point_and_direction() {
            let pipe_map = helper_create_simple_pipe_map();
            let point = Point { row: 1, column: 1 };
            let direction = East;
            let expected_point = Point { row: 1, column: 2 };
            let expected_direction = East;
            assert_eq!(
                pipe_map.next_point_and_direction(point, direction),
                (Some(expected_point), Some(expected_direction))
            )
        }

        #[test]
        fn test_path_to_start() {
            let pipe_map = helper_create_simple_pipe_map();
            let path_to_start = pipe_map.path_to_start(pipe_map.get_start(), East);
            assert_eq!(path_to_start.map(|path| path.len()), Some(8))
        }
    }

    #[test]
    fn test_part1() {
        let input = ".....
.S-7.
.|.|.
.L-J.
.....";
        assert_eq!(part1(input), "4");
        let input = "..F7.
.FJ|.
SJ.L7
|F--J
LJ...";
        assert_eq!(part1(input), "8");
    }

    #[test]
    fn test_part2() {
        let input = "FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L
";
        assert_eq!(part2(input), "10")
    }
}
