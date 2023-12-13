use nom::branch::alt;
use nom::character::complete;
use nom::character::complete::newline;
use nom::combinator::{map, value};
use nom::multi::{many1, separated_list1};
use nom::sequence::pair;
use nom::IResult;

#[derive(Debug, PartialEq)]
struct RockAndAshMap(Vec<Vec<bool>>);

impl RockAndAshMap {
    fn is_mirror_point(&self, row: usize) -> bool {
        if row == 0 || row >= self.0.len() {
            return false;
        }

        // We need to work outwards from the row
        let rows_backwards = self.0[0..row].iter().rev();
        let rows_forward = self.0[row..].iter();

        rows_backwards
            .zip(rows_forward)
            .all(|(back, forward)| back == forward)
    }

    fn find_mirror_point(&self) -> Option<usize> {
        for row in 0..self.0.len() {
            if self.is_mirror_point(row) {
                return Some(row);
            }
        }
        None
    }

    fn transpose(&self) -> RockAndAshMap {
        let v = &self.0;
        let rows = v.len();
        let cols = v[0].len();

        let transposed: Vec<Vec<_>> = (0..cols)
            .map(|col| (0..rows).map(|row| v[row][col]).collect())
            .collect();

        RockAndAshMap(transposed)
    }
}

/// Rock will be true, ash will be false
fn parse_rock_or_ash(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, complete::char('#')),
        value(false, complete::char('.')),
    ))(input)
}

fn parse_rock_and_ash_map(input: &str) -> IResult<&str, RockAndAshMap> {
    map(separated_list1(newline, many1(parse_rock_or_ash)), |map| {
        RockAndAshMap(map)
    })(input)
}

fn parse_rock_and_ash_maps(input: &str) -> IResult<&str, Vec<RockAndAshMap>> {
    separated_list1(pair(newline, newline), parse_rock_and_ash_map)(input)
}

pub fn part1(_input: &str) -> String {
    todo!()
}

pub fn part2(_input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    mod parsers {
        use super::*;

        fn get_test_input() -> &'static str {
            "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#"
        }

        #[test]
        fn test_parse_rock_or_ash() {
            assert_eq!(parse_rock_or_ash("#."), Ok((".", true)));
            assert_eq!(parse_rock_or_ash(".#"), Ok(("#", false)));
        }

        #[test]
        fn test_parse_rock_and_ash_map() {
            let input = "#..
.#.
.##";
            let expected = RockAndAshMap(vec![
                vec![true, false, false],
                vec![false, true, false],
                vec![false, true, true],
            ]);
            assert_eq!(parse_rock_and_ash_map(input), Ok(("", expected)));
        }

        #[test]
        fn test_parse_rock_and_ash_maps() {
            let input = get_test_input();
            let maps = parse_rock_and_ash_maps(input).unwrap().1;

            assert_eq!(maps.len(), 2);

            assert_eq!(
                maps[0].0[0],
                vec![true, false, true, true, false, false, true, true, false]
            );

            assert_eq!(
                maps[1].0[0],
                vec![true, false, false, false, true, true, false, false, true]
            );
        }
    }

    mod rock_and_ash_map {
        use super::*;

        #[test]
        fn test_transpose() {
            let map = RockAndAshMap(vec![
                vec![true, false, false],
                vec![true, false, false],
                vec![true, false, false],
                vec![true, false, false],
            ]);
            let expected = RockAndAshMap(vec![
                vec![true, true, true, true],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ]);
            assert_eq!(map.transpose(), expected);
        }

        #[test]
        fn test_find_mirror() {
            let map = RockAndAshMap(vec![
                vec![true, false, true],
                vec![true, false, false],
                vec![true, true, false],
                vec![true, true, false],
                vec![true, false, false],
            ]);
            assert_eq!(map.find_mirror_point(), Some(3));

            let map = RockAndAshMap(vec![
                vec![true, true, false],
                vec![true, true, false],
                vec![true, false, false],
                vec![true, false, false],
                vec![true, false, false],
            ]);
            assert_eq!(map.find_mirror_point(), Some(1));

            let map = RockAndAshMap(vec![
                vec![true, false, false],
                vec![true, false, true],
                vec![true, false, false],
                vec![true, true, false],
                vec![true, true, false],
            ]);
            assert_eq!(map.find_mirror_point(), Some(4));
        }
    }

    #[ignore]
    #[test]
    fn test_part1() {
        let input = "";
        assert_eq!(part1(input), "")
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "")
    }
}
