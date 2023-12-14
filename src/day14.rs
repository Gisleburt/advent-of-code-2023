use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete;
use nom::character::complete::newline;
use nom::combinator::{map, value};
use nom::multi::{many1, separated_list1};
use nom::IResult;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
enum Rock {
    Round,
    Cube,
}

fn roll_rocks(rocks: &[Option<Rock>]) -> Vec<Option<Rock>> {
    rocks
        .iter()
        .copied()
        .group_by(|rock| rock == &Some(Rock::Cube))
        .into_iter()
        .flat_map(|(_, subset)| {
            let mut v = subset.into_iter().collect_vec();
            v.sort();
            v.reverse();
            v
        })
        .collect()
}

fn get_load(rocks: &[Option<Rock>]) -> usize {
    rocks
        .iter()
        .rev()
        .enumerate()
        .filter_map(|(load, rock)| rock.map(|rock| (load, rock)))
        .filter(|(_, rock)| rock == &Rock::Round)
        .map(|(load, _)| load + 1) // indexing starts from 1
        .sum::<usize>()
}

#[derive(Debug, Clone, PartialEq)]
struct RockMap(Vec<Vec<Option<Rock>>>);

impl RockMap {
    fn roll_rocks(&self) -> Self {
        RockMap(self.0.iter().map(|row| roll_rocks(row)).collect())
    }

    fn get_load(&self) -> usize {
        self.0.iter().map(|row| get_load(row)).sum()
    }

    fn rotate_counter_clockwise(&self) -> Self {
        let mut temp = self.0.clone(); // Temp store, we'll rewrite all data but its now the same size
        let row_length = self.0.len();
        let column_length = self.0[0].len();

        for row in 0..row_length {
            for col in 0..column_length {
                temp[column_length - col - 1][row] = self.0[row][col];
            }
        }

        RockMap(temp)
    }

    #[allow(clippy::needless_range_loop)] // Want to keep this the same as the other loop
    fn rotate_clockwise(&self) -> Self {
        let mut temp = self.0.clone(); // Temp store, we'll rewrite all data but its now the same size
        let row_length = self.0.len();
        let column_length = self.0[0].len();

        for row in 0..row_length {
            for col in 0..column_length {
                temp[col][column_length - row - 1] = self.0[row][col];
            }
        }

        RockMap(temp)
    }

    fn spin(&self) -> Self {
        self.roll_rocks()
            .rotate_clockwise()
            .roll_rocks()
            .rotate_clockwise()
            .roll_rocks()
            .rotate_clockwise()
            .roll_rocks()
            .rotate_clockwise()
    }
}

fn parse_rock(input: &str) -> IResult<&str, Option<Rock>> {
    alt((
        value(Some(Rock::Round), complete::char('O')),
        value(Some(Rock::Cube), complete::char('#')),
        value(None, complete::char('.')),
    ))(input)
}

fn parse_rocks(input: &str) -> IResult<&str, Vec<Option<Rock>>> {
    many1(parse_rock)(input)
}

fn parse_rock_map(input: &str) -> IResult<&str, RockMap> {
    map(separated_list1(newline, parse_rocks), RockMap)(input)
}

fn get_prerotated_map(input: &str) -> RockMap {
    parse_rock_map(input).unwrap().1.rotate_counter_clockwise()
}

pub fn part1(input: &str) -> String {
    let rock_map = get_prerotated_map(input).roll_rocks();
    rock_map.get_load().to_string()
}

pub fn part2(input: &str) -> String {
    let mut history = vec![get_prerotated_map(input)];
    let loop_start = loop {
        let new_map = history.last().unwrap().spin();
        let found_pos = history.iter().position(|map| map == &new_map);
        if let Some(pos) = found_pos {
            break pos;
        }
        history.push(new_map);
    };
    let loop_size = history.len() - loop_start;
    let billionth_map_pos = ((1_000_000_000_usize - loop_start) % loop_size) + loop_start;
    history[billionth_map_pos].get_load().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_test_input() -> &'static str {
        "O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#...."
    }

    mod rocks {
        use Rock::*;

        use super::*;

        #[test]
        fn test_get_prerotated_map() {
            let rock_map = get_prerotated_map(
                "#.O
..O
..O",
            );
            let expected = RockMap(vec![
                vec![Some(Round), Some(Round), Some(Round)],
                vec![None, None, None],
                vec![Some(Cube), None, None],
            ]);

            assert_eq!(rock_map, expected);
        }

        #[test]
        fn test_roll_rocks() {
            let rocks = vec![
                None,
                Some(Round),
                Some(Cube),
                None,
                Some(Round),
                None,
                Some(Round),
            ];
            let expected = vec![
                Some(Round),
                None,
                Some(Cube),
                Some(Round),
                Some(Round),
                None,
                None,
            ];
            assert_eq!(roll_rocks(&rocks), expected);
        }

        #[test]
        fn test_get_load() {
            let rocks = vec![
                Some(Round), // 5 -> 5
                Some(Cube),  // 4 -> None
                None,        // 3 -> None
                Some(Round), // 2 -> 2
                None,        // 1 -> None
            ];
            let expected = 7;
            assert_eq!(get_load(&rocks), expected);
        }

        #[test]
        fn test_rotate_counter_clockwise() {
            let rocks = RockMap(vec![
                vec![Some(Cube), None, Some(Round)],
                vec![Some(Cube), None, None],
                vec![Some(Cube), None, Some(Cube)],
            ]);
            let expected = RockMap(vec![
                vec![Some(Round), None, Some(Cube)],
                vec![None, None, None],
                vec![Some(Cube), Some(Cube), Some(Cube)],
            ]);

            assert_eq!(rocks.rotate_counter_clockwise(), expected)
        }

        #[test]
        fn test_rotate_clockwise() {
            let rocks = RockMap(vec![
                vec![Some(Cube), None, Some(Round)],
                vec![Some(Cube), None, None],
                vec![Some(Cube), None, Some(Cube)],
            ]);
            let expected = RockMap(vec![
                vec![Some(Cube), Some(Cube), Some(Cube)],
                vec![None, None, None],
                vec![Some(Cube), None, Some(Round)],
            ]);

            assert_eq!(rocks.rotate_clockwise(), expected)
        }

        #[test]
        fn test_roll_map() {
            let rocks = RockMap(vec![
                vec![Some(Cube), None, Some(Round)],
                vec![None, Some(Round), Some(Round)],
                vec![None, Some(Cube), Some(Round)],
            ]);
            let expected = RockMap(vec![
                vec![Some(Cube), Some(Round), None],
                vec![Some(Round), Some(Round), None],
                vec![None, Some(Cube), Some(Round)],
            ]);

            assert_eq!(rocks.roll_rocks(), expected)
        }

        #[test]
        fn test_spin() {
            let initial = get_prerotated_map(get_test_input());
            let expected_input_1 = ".....#....
....#...O#
...OO##...
.OO#......
.....OOO#.
.O#...O#.#
....O#....
......OOOO
#...O###..
#..OO#....";
            let expected = get_prerotated_map(expected_input_1);
            assert_eq!(initial.spin(), expected);
        }
    }

    #[test]
    fn test_part1() {
        let input = get_test_input();
        assert_eq!(part1(input), "136");
    }

    #[test]
    fn test_part2() {
        let input = get_test_input();
        assert_eq!(part2(input), "64")
    }
}
