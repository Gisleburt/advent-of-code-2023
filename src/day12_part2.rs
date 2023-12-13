use std::collections::HashMap;
use std::iter;

use bitvec::field::BitField;
use bitvec::vec::BitVec;
use itertools::Itertools;
use nom::bytes::complete::take_while;
use nom::character::complete;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;
use rayon::prelude::*;

struct JiggleMachine(HashMap<usize, Vec<Vec<usize>>>);

impl JiggleMachine {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn get_jiggle_combinations(&mut self, value: usize) -> Vec<Vec<usize>> {
        eprintln!("Jiggle Factor: {value}");

        if let Some(entry) = self.0.get(&value) {
            return entry.clone();
        }

        if value == 0 {
            self.0.insert(0, vec![]);
            return self.0.get(&0).unwrap().clone();
        }

        let mut combinations = vec![vec![value]];
        for sub in 0..value {
            let start = value - sub;
            let sub_results = self.get_jiggle_combinations(sub);
            let altered_results: Vec<_> = sub_results
                .into_iter()
                .zip(iter::repeat(start))
                .map(|(mut v, sub)| {
                    v.push(sub);
                    v.sort();
                    v
                })
                .unique()
                .collect();
            combinations.extend(altered_results);
        }
        combinations.sort();
        combinations.dedup();
        self.0.insert(value, combinations);
        self.0.get(&value).unwrap().clone()
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
struct ConditionReport<'a> {
    conditions: &'a str,
    groups: Vec<u64>,
    good_number: u128,
    bad_number: u128,
}

impl<'a> ConditionReport<'a> {
    fn get_good_number(conditions: &str) -> u128 {
        let bitvec: BitVec = conditions.chars().map(|c| c == '.').rev().collect();
        bitvec.load()
    }
    fn get_bad_number(conditions: &str) -> u128 {
        let bitvec: BitVec = conditions.chars().map(|c| c == '#').rev().collect();
        bitvec.load()
    }

    fn new(conditions: &'a str, groups: Vec<u64>) -> Self {
        let good_number = Self::get_good_number(&conditions);
        let bad_number = Self::get_bad_number(&conditions);
        Self {
            conditions,
            groups,
            good_number,
            bad_number,
        }
    }

    fn get_number_of_possibles(&self, jiggle_machine: &mut JiggleMachine) -> usize {
        // This gives me something like [[true, true], [false], [true, true]]
        let binary_groups: Vec<_> = self
            .groups
            .iter()
            .copied()
            .map(|size| iter::repeat(true).take(size as usize).collect::<Vec<_>>())
            .(itertools::Itertools::intersperse)(vec![false])
            .collect::<Vec<_>>();

        // Get the jiggles
        let groups_min_length = binary_groups.len();
        let jiggle_room = self.conditions.len() - groups_min_length;
        eprintln!("jiggle {jiggle_room}");

        let vec1 = jiggle_machine.get_jiggle_combinations(jiggle_room);
        let binary_jiggles: Vec<_> = vec1
            .into_iter()
            .map(|groups| {
                groups
                    .iter()
                    .copied()
                    .map(|size| iter::repeat(true).take(size as usize).collect::<Vec<_>>())
                    .intersperse(vec![false])
                    .collect::<Vec<_>>()
            })
            .collect();

        binary_jiggles
            .iter()
            .permutations(binary_jiggles.len())
            .flatten()
            .map(|jiggles| jiggles.iter().interleave(binary_groups.iter()))
            .map(|parts| {
                let bv: BitVec = parts.collect_vec().into_iter().flatten().collect();
                bv.load::<u128>()
            })
            .dedup()
            .filter(|number| {
                (number & self.bad_number == self.bad_number)
                    && (!number & self.good_number == self.good_number)
            })
            .count()
    }
}

fn is_condition(c: char) -> bool {
    c == '.' || c == '#' || c == '?'
}

fn parse_conditions(input: &str) -> IResult<&str, &str> {
    take_while(is_condition)(input)
}

fn parse_groups(input: &str) -> IResult<&str, Vec<u64>> {
    separated_list1(complete::char(','), complete::u64)(input)
}

fn parse_condition_report(input: &str) -> IResult<&str, ConditionReport> {
    map(
        separated_pair(parse_conditions, space1, parse_groups),
        |(conditions, groups)| ConditionReport::new(conditions, groups),
    )(input)
}

fn line_to_condition_report(input: &str) -> ConditionReport {
    parse_condition_report(input).unwrap().1
}

pub fn part1(input: &str) -> String {
    let mut jiggle_machine = JiggleMachine::new();
    let reports = input.lines().map(line_to_condition_report).collect_vec();
    reports
        .iter()
        .map(|report| report.get_number_of_possibles(&mut jiggle_machine))
        .sum::<usize>()
        .to_string()
}

pub fn part2(input: &str) -> String {
    let bigger_input = input
        .lines()
        .map(|line| iter::repeat(line).take(5).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    part1(&bigger_input)
}

#[cfg(test)]
mod test {
    use super::*;

    mod parser {
        use super::*;

        #[test]
        fn test_parse_conditions() {
            assert_eq!(parse_conditions(".#??#. "), Ok((" ", ".#??#.")));
        }

        #[test]
        fn test_parse_groups() {
            assert_eq!(parse_groups("1,2,3,4\n"), Ok(("\n", vec![1, 2, 3, 4])));
        }

        #[test]
        fn test_parse_condition_report() {
            let expected = ConditionReport {
                conditions: "#???????#??????.#??.",
                groups: vec![10, 1, 1, 1],
                //             #???????#??????.#??.
                good_number: 0b00000000000000010001,
                bad_number: 0b10000000100000001000,
            };
            let result = parse_condition_report("#???????#??????.#??. 10,1,1,1\n");
            assert_eq!(result, Ok(("\n", expected)));

            let expected = ConditionReport {
                conditions: ".??..??...?##.",
                groups: vec![1, 1, 3],
                //             .??..??...?##.
                good_number: 0b10011001110001,
                bad_number: 0b00000000000110,
            };
            let result = parse_condition_report(".??..??...?##. 1,1,3");
            assert_eq!(result, Ok(("", expected)));
        }
    }

    #[test]
    fn test_combinations_that_sum_to() {
        let mut jiggle_machine = JiggleMachine::new();
        let combinations = jiggle_machine.get_jiggle_combinations(4);
        let expected = vec![
            vec![1, 1, 1, 1],
            vec![1, 1, 2],
            vec![1, 3],
            vec![2, 2],
            vec![4],
        ];
        assert_eq!(combinations, expected)
    }

    #[test]
    fn test_interleave() {
        let v1 = [1, 3];
        let v2 = [2, 4];
        let interleaved = v1.iter().interleave(v2.iter()).copied().collect_vec();
        assert_eq!(interleaved, vec![1, 2, 3, 4])
    }

    #[ignore]
    #[test]
    fn test_part1() {
        let input = "???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1";
        assert_eq!(part1(input), "21")
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1";
        assert_eq!(part2(input), "525152")
    }
}
