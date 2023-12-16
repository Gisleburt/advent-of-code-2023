use std::fmt::{Display, Formatter};

use derive_more::{Deref, DerefMut, From as FromMore};
use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete;
use nom::character::complete::newline;
use nom::combinator::{map, value};
use nom::multi::{many1, separated_list1};
use nom::IResult;

use crate::day16::Direction::*;
use crate::day16::TileType::*;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum TileType {
    // .
    Empty,
    // /
    MirrorForward,
    // \
    MirrorBackward,
    // |
    VerticalSplitter,
    // -
    HorizontalSplitter,
}

impl TileType {
    fn process_light(&self, direction: Direction) -> (Direction, Option<Direction>) {
        match self {
            Empty => (direction, None),
            MirrorForward => match direction {
                Up => (Right, None),
                Down => (Left, None),
                Left => (Down, None),
                Right => (Up, None),
            },
            MirrorBackward => match direction {
                Up => (Left, None),
                Down => (Right, None),
                Left => (Up, None),
                Right => (Down, None),
            },
            VerticalSplitter => match direction {
                Up => (Up, None),
                Down => (Down, None),
                Left => (Up, Some(Down)),
                Right => (Up, Some(Down)),
            },
            HorizontalSplitter => match direction {
                Up => (Left, Some(Right)),
                Down => (Left, Some(Right)),
                Left => (Left, None),
                Right => (Right, None),
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Tile {
    tile_type: TileType,
    seen_up: bool,
    seen_down: bool,
    seen_left: bool,
    seen_right: bool,
}

impl Tile {
    fn new(tile_type: TileType) -> Self {
        Tile {
            tile_type,
            seen_up: false,
            seen_down: false,
            seen_left: false,
            seen_right: false,
        }
    }

    fn is_energized(&self) -> bool {
        self.seen_up || self.seen_down || self.seen_left || self.seen_right
    }

    fn process_light(&mut self, direction: Direction) -> Option<(Direction, Option<Direction>)> {
        match direction {
            Up => {
                if std::mem::replace(&mut self.seen_up, true) {
                    return None;
                }
            }
            Down => {
                if std::mem::replace(&mut self.seen_down, true) {
                    return None;
                }
            }
            Left => {
                if std::mem::replace(&mut self.seen_left, true) {
                    return None;
                }
            }
            Right => {
                if std::mem::replace(&mut self.seen_right, true) {
                    return None;
                }
            }
        };
        Some(self.tile_type.process_light(direction))
    }
}

#[derive(Debug, Clone, Deref, DerefMut, FromMore)]
#[deref(forward)]
struct TileMap(Vec<Vec<Tile>>);

impl TileMap {
    fn energy_level(&self) -> usize {
        self.iter()
            .flat_map(|row| row.iter())
            .filter(|tile| tile.is_energized())
            .count()
    }

    fn process_light(&mut self, pos: Pos, direction: Direction) {
        let Pos { row, column } = pos;
        // This will early return if the tile has already seen light go in that direction
        let Some((next, maybe_also)) = self[row][column].process_light(direction) else {
            return;
        };
        // Deal with the direction we just got back
        if let Some(next_pos) = self.get_next_pos(pos, next) {
            self.process_light(next_pos, next);
        }
        // If the beam hit a spliter
        if let Some(maybe_direction) = maybe_also {
            if let Some(next_pos) = self.get_next_pos(pos, maybe_direction) {
                self.process_light(next_pos, maybe_direction);
            }
        }
    }

    fn get_next_pos(&self, pos: Pos, direction: Direction) -> Option<Pos> {
        pos.apply_direction(direction)
            .filter(|pos| self.pos_is_in_bounds(*pos))
    }

    fn pos_is_in_bounds(&self, pos: Pos) -> bool {
        pos.row < self.height() && pos.column < self.width()
    }

    fn width(&self) -> usize {
        self[0].len()
    }

    fn height(&self) -> usize {
        self.len()
    }
}

impl Display for TileMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|row| row
                    .iter()
                    .map(|tile| if tile.is_energized() { "#" } else { "." })
                    .collect::<String>())
                .join("\n")
        )
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
struct Pos {
    row: usize,
    column: usize,
}

impl Pos {
    fn apply_direction(&self, direction: Direction) -> Option<Self> {
        let Pos { row, column } = *self;
        match direction {
            Up => (row > 0).then_some(Pos {
                row: row.saturating_sub(1),
                column,
            }),
            Down => Some(Pos {
                row: row + 1,
                column,
            }),
            Left => (column > 0).then_some(Pos {
                row,
                column: column.saturating_sub(1),
            }),
            Right => Some(Pos {
                row,
                column: column + 1,
            }),
        }
    }
}

fn parse_tile(input: &str) -> IResult<&str, Tile> {
    alt((
        value(Tile::new(Empty), complete::char('.')),
        value(Tile::new(MirrorForward), complete::char('/')),
        value(Tile::new(MirrorBackward), complete::char('\\')),
        value(Tile::new(VerticalSplitter), complete::char('|')),
        value(Tile::new(HorizontalSplitter), complete::char('-')),
    ))(input)
}

fn parse_tile_map(input: &str) -> IResult<&str, TileMap> {
    map(separated_list1(newline, many1(parse_tile)), TileMap::from)(input)
}

fn input_into_tile_map(input: &str) -> TileMap {
    parse_tile_map(input).expect("failed to parse tile map").1
}

pub fn part1(input: &str) -> String {
    let mut tile_map = input_into_tile_map(input);
    tile_map.process_light(Pos::default(), Right);
    // eprintln!("{tile_map}");
    tile_map.energy_level().to_string()
}

pub fn part2(input: &str) -> String {
    let map = input_into_tile_map(input);
    let mut energy_levels: Vec<usize> = Vec::with_capacity((map.width() + map.height()) * 2);

    for row in 0..map.height() {
        for (direction, column) in [(Right, 0), (Left, map.width() - 1)] {
            let mut clone = map.clone();
            clone.process_light(Pos { row, column }, direction);
            energy_levels.push(clone.energy_level());
        }
    }

    for column in 0..map.width() {
        for (direction, row) in [(Down, 0), (Up, map.height() - 1)] {
            let mut clone = map.clone();
            clone.process_light(Pos { row, column }, direction);
            energy_levels.push(clone.energy_level());
        }
    }

    energy_levels.into_iter().max().unwrap().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    mod tile_type {
        use super::*;

        #[test]
        fn test_process_light() {
            let tile = Empty; // .
            assert_eq!(tile.process_light(Up), (Up, None));
            assert_eq!(tile.process_light(Down), (Down, None));
            assert_eq!(tile.process_light(Left), (Left, None));
            assert_eq!(tile.process_light(Right), (Right, None));
            let tile = MirrorForward; // /
            assert_eq!(tile.process_light(Up), (Right, None));
            assert_eq!(tile.process_light(Down), (Left, None));
            assert_eq!(tile.process_light(Left), (Down, None));
            assert_eq!(tile.process_light(Right), (Up, None));
            let tile = MirrorBackward; // \
            assert_eq!(tile.process_light(Up), (Left, None));
            assert_eq!(tile.process_light(Down), (Right, None));
            assert_eq!(tile.process_light(Left), (Up, None));
            assert_eq!(tile.process_light(Right), (Down, None));
            let tile = VerticalSplitter;
            assert_eq!(tile.process_light(Up), (Up, None));
            assert_eq!(tile.process_light(Down), (Down, None));
            assert_eq!(tile.process_light(Left), (Up, Some(Down)));
            assert_eq!(tile.process_light(Right), (Up, Some(Down)));
            let tile = HorizontalSplitter;
            assert_eq!(tile.process_light(Up), (Left, Some(Right)));
            assert_eq!(tile.process_light(Down), (Left, Some(Right)));
            assert_eq!(tile.process_light(Left), (Left, None));
            assert_eq!(tile.process_light(Right), (Right, None));
        }
    }

    mod tile {
        use super::*;

        #[test]
        fn test_is_energized() {
            let mut tile = Tile::new(Empty);
            assert!(!tile.is_energized());
            tile.process_light(Up);
            assert!(tile.is_energized());

            let mut tile = Tile::new(Empty);
            assert!(!tile.is_energized());
            tile.process_light(Down);
            assert!(tile.is_energized());

            let mut tile = Tile::new(Empty);
            assert!(!tile.is_energized());
            tile.process_light(Left);
            assert!(tile.is_energized());

            let mut tile = Tile::new(Empty);
            assert!(!tile.is_energized());
            tile.process_light(Right);
            assert!(tile.is_energized());
        }

        #[test]
        fn test_process_light() {
            let mut tile = Tile::new(Empty);
            assert_eq!(tile.process_light(Up), Some((Up, None)));
            assert_eq!(tile.process_light(Up), None);

            assert_eq!(tile.process_light(Down), Some((Down, None)));
            assert_eq!(tile.process_light(Down), None);

            assert_eq!(tile.process_light(Left), Some((Left, None)));
            assert_eq!(tile.process_light(Left), None);

            assert_eq!(tile.process_light(Right), Some((Right, None)));
            assert_eq!(tile.process_light(Right), None);
        }
    }

    #[test]
    fn test_part1() {
        let input = r#".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|...."#;
        assert_eq!(part1(input), "46");
    }

    #[test]
    fn test_part2() {
        let input = r#".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|...."#;
        assert_eq!(part2(input), "51");
    }
}
