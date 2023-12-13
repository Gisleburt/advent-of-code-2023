use nom::branch::alt;
use nom::character::complete;
use nom::character::complete::newline;
use nom::combinator::{map, value};
use nom::multi::{many1, separated_list1};
use nom::sequence::pair;
use nom::IResult;

#[derive(Debug, PartialEq)]
struct RockAndAshMap(Vec<Vec<bool>>);

fn is_smudged(v1: &[bool], v2: &[bool]) -> bool {
    v1.iter().zip(v2).filter(|(a, b)| a != b).count() == 1
}

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
        (0..self.0.len()).find(|&row| self.is_mirror_point(row))
    }

    fn is_mirror_point_with_smudge(&self, row: usize) -> bool {
        if row == 0 || row >= self.0.len() {
            return false;
        }

        // We need to work outwards from the row
        let rows_backwards = self.0[0..row].iter().rev();
        let rows_forward = self.0[row..].iter();

        let mut smudge_used = false;
        for (back, forward) in rows_backwards.zip(rows_forward) {
            if back == forward {
                continue;
            }
            if smudge_used || !is_smudged(back, forward) {
                return false;
            }
            smudge_used = true;
        }
        smudge_used
    }

    fn find_mirror_point_with_smudge(&self) -> Option<usize> {
        (0..self.0.len()).find(|&row| self.is_mirror_point_with_smudge(row))
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

pub fn part1(input: &str) -> String {
    let maps = parse_rock_and_ash_maps(input).unwrap().1;

    maps.iter()
        .map(|map| {
            map.find_mirror_point()
                .map(|mirror| mirror * 100)
                .or_else(|| map.transpose().find_mirror_point())
                .unwrap_or(0)
        })
        .sum::<usize>()
        .to_string()
}

pub fn part2(input: &str) -> String {
    let maps = parse_rock_and_ash_maps(input).unwrap().1;

    maps.iter()
        .map(|map| {
            map.find_mirror_point_with_smudge()
                .map(|mirror| mirror * 100)
                .or_else(|| map.transpose().find_mirror_point_with_smudge())
                .unwrap_or(0)
        })
        .sum::<usize>()
        .to_string()
}

#[cfg(test)]
mod test {
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

    mod parsers {
        use super::*;

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

        #[test]
        fn test_is_smudged() {
            let v1 = vec![true, true, true, true];
            let v2 = vec![true, true, false, true];
            assert!(is_smudged(&v1, &v2));

            let v1 = vec![true, true, true, true];
            let v2 = vec![true, false, false, true];
            assert!(!is_smudged(&v1, &v2));
        }

        #[test]
        fn test_find_mirror_point_with_smudge() {
            let input = "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.";
            let map = parse_rock_and_ash_map(input).unwrap().1;
            assert_eq!(map.find_mirror_point_with_smudge(), Some(3))
        }
    }

    #[test]
    fn test_part1() {
        let input = get_test_input();
        assert_eq!(part1(input), "405")
    }

    #[test]
    fn test_part2() {
        let input = get_test_input();
        assert_eq!(part2(input), "400")
    }
}
