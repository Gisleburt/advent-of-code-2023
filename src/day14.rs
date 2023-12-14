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
        .map(|(_, subset)| {
            let mut v = subset.into_iter().collect_vec();
            v.sort();
            v.reverse();
            v
        })
        .flatten()
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
    fn transpose(&self) -> Self {
        let v = &self.0;
        let rows = v.len();
        let cols = v[0].len();

        let transposed: Vec<Vec<_>> = (0..cols)
            .map(|col| (0..rows).map(|row| v[row][col]).collect())
            .collect();

        RockMap(transposed)
    }

    fn roll_rocks(&self) -> Self {
        RockMap(self.0.iter().map(|row| roll_rocks(row)).collect())
    }

    fn get_load(&self) -> usize {
        self.0.iter().map(|row| get_load(row)).sum()
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
    map(separated_list1(newline, parse_rocks), |v| RockMap(v))(input)
}

pub fn part1(input: &str) -> String {
    let rock_map = parse_rock_map(input).unwrap().1.transpose().roll_rocks();
    rock_map.get_load().to_string()
}

pub fn part2(_input: &str) -> String {
    todo!()
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
        fn test_transpose() {
            let rocks = RockMap(vec![
                vec![Some(Cube), None, Some(Round)],
                vec![Some(Cube), None, None],
                vec![Some(Cube), None, Some(Cube)],
            ]);
            let expected = RockMap(vec![
                vec![Some(Cube), Some(Cube), Some(Cube)],
                vec![None, None, None],
                vec![Some(Round), None, Some(Cube)],
            ]);

            assert_eq!(rocks.transpose(), expected)
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
    }

    #[test]
    fn test_part1() {
        let input = get_test_input();
        assert_eq!(part1(input), "136");
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "")
    }
}
