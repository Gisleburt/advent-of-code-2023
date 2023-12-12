use bitvec::field::BitField;
use bitvec::prelude::BitVec;
use indicatif::ProgressIterator;
use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::{self, char, space1};
use nom::combinator::{map, value};
use nom::multi::{many1, separated_list1};
use nom::sequence::separated_pair;
use nom::IResult;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Condition {
    Good,
    Bad,
}

type Groups = Vec<u64>;

#[derive(Debug, Clone, PartialEq)]
struct ConditionReport {
    known_conditions: Vec<Option<Condition>>,
    groups: Groups,
}

fn triangular_number(input: u64, increasing_base_size: u64) -> u64 {
    let mut count = 1;
    for layer in 1..input {
        count += (increasing_base_size * layer) + 1
    }
    count
}

fn combinations_of_set(set_size: u64, jiggle_room: u64) -> u64 {
    triangular_number(jiggle_room + 1, set_size)
}

fn conditions_to_groups(conditions: &[Condition]) -> Groups {
    conditions
        .iter()
        .group_by(|condition| *condition == &Condition::Bad)
        .into_iter()
        .filter(|(key, _value)| *key)
        .map(|(_key, value)| value.into_iter().count() as u64)
        .collect()
}

// Split Good, Bad and unknown conditions into groups of bad and unknown seperated by where good is
fn get_bad_and_unknown_conditions(conditions: &[Option<Condition>]) -> Vec<Vec<Option<Condition>>> {
    conditions
        .iter()
        .group_by(|condition| condition.as_ref() != Some(&Condition::Good))
        .into_iter()
        .filter(|(key, _value)| *key)
        .map(|(_key, value)| value.into_iter().copied().collect())
        .collect()
}

// Split Good, Bad and unknown conditions into groups of bad and unknown seperated by where good is
fn get_bad_conditions(conditions: &[Option<Condition>]) -> Vec<Vec<Condition>> {
    conditions
        .iter()
        .group_by(|condition| condition.as_ref() == Some(&Condition::Bad))
        .into_iter()
        .filter(|(key, _value)| *key)
        .map(|(_key, value)| {
            value
                .into_iter()
                .filter_map(|cond| *cond)
                .collect::<Vec<_>>()
        })
        .collect()
}

struct ConditionsToNumber {
    conditions: Vec<Option<Condition>>,
    good_number: u32,
    bad_number: u32,
}

impl ConditionsToNumber {
    fn get_good_number(conditions: &[Option<Condition>]) -> u32 {
        let bitvec: BitVec = conditions
            .iter()
            .map(|condition| *condition == Some(Condition::Good))
            .collect();
        bitvec.load()
    }
    fn get_bad_number(conditions: &[Option<Condition>]) -> u32 {
        let bitvec: BitVec = conditions
            .iter()
            .map(|condition| *condition == Some(Condition::Good))
            .collect();
        bitvec.load()
    }

    fn new(conditions: Vec<Option<Condition>>) -> Self {
        let good_number = Self::get_good_number(&conditions);
        let bad_number = Self::get_bad_number(&conditions);
        Self {
            conditions,
            good_number,
            bad_number,
        }
    }
}

fn may_contain(conditions: &[Option<Condition>]) -> Vec<Groups> {
    if conditions
        .iter()
        .any(|condition| condition.as_ref() == Some(&Condition::Good))
    {
        panic!("split out good conditions before using may_contain")
    }

    // ??#?#
    // 00101
    // 00111
    // 01101
    // 01111
    // 11101
    // 11111
    // This is binary!

    let box_size = conditions.len();

    vec![]
}

fn validate_possible_conditions(
    known_conditions: &[Option<Condition>],
    possible_conditions: &[Condition],
) -> bool {
    // Could return false here but if these don't match it won't work
    if possible_conditions.len() != known_conditions.len() {
        panic!("Invalid length of possible conditions (did not match length of known conditions")
    }

    possible_conditions
        .iter()
        .zip(known_conditions.iter())
        .all(|(possible, known)| known.is_none() || known.as_ref() == Some(possible))
}

impl ConditionReport {
    fn get_possible_broken_groups(&self) -> Vec<Vec<Option<Condition>>> {
        get_bad_and_unknown_conditions(&self.known_conditions)
    }

    fn validate_possible_conditions(&self, possible_conditions: Vec<Condition>) -> bool {
        conditions_to_groups(&possible_conditions) == self.groups
            && validate_possible_conditions(&self.known_conditions, &possible_conditions)
    }
}

impl From<(Vec<Option<Condition>>, Groups)> for ConditionReport {
    fn from((known_conditions, groups): (Vec<Option<Condition>>, Groups)) -> Self {
        Self {
            known_conditions,
            groups,
        }
    }
}

fn parse_condition(input: &str) -> IResult<&str, Option<Condition>> {
    alt((
        value(Some(Condition::Good), char('.')),
        value(Some(Condition::Bad), char('#')),
        value(None, char('?')),
    ))(input)
}

fn parse_condition_report(input: &str) -> IResult<&str, ConditionReport> {
    map(
        separated_pair(
            many1(parse_condition),
            space1,
            separated_list1(char(','), complete::u64),
        ),
        |parts| parts.into(),
    )(input)
}

pub fn part1(_input: &str) -> String {
    let vec: Vec<_> = (0..(2_u32.pow(20))).into_iter().progress().collect();
    vec.len().to_string()
}

pub fn part2(_input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    mod parts {
        use super::*;

        #[test]
        fn test_triangle_number() {
            let base = 1;
            assert_eq!(triangular_number(1, base), 1); // O
            assert_eq!(triangular_number(2, base), 3); // O OO
            assert_eq!(triangular_number(3, base), 6); // O OO OOO
            assert_eq!(triangular_number(4, base), 10); // O OO OOO OOOO

            let base = 2;
            assert_eq!(triangular_number(1, base), 1); // O
            assert_eq!(triangular_number(2, base), 4); // O OOO
            assert_eq!(triangular_number(3, base), 9); // O OOO OOOOO
            assert_eq!(triangular_number(4, base), 16); // O OOO OOOOO OOOOOOO

            let base = 3;
            assert_eq!(triangular_number(1, base), 1); // O
            assert_eq!(triangular_number(2, base), 5); // O OOOO
            assert_eq!(triangular_number(3, base), 12); // O OOOO OOOOOOO
            assert_eq!(triangular_number(4, base), 22); // O OOOO OOOOOOO OOOOOOOOOO
        }

        #[test]
        fn test_combinations_of_set() {
            let set_size = 1;
            assert_eq!(combinations_of_set(set_size, 0), 1); // O
            assert_eq!(combinations_of_set(set_size, 1), 3); // O OO
            assert_eq!(combinations_of_set(set_size, 2), 6); // O OO OOO
            assert_eq!(combinations_of_set(set_size, 3), 10); // O OO OOO OOOO

            let set_size = 2;
            assert_eq!(combinations_of_set(set_size, 0), 1); // O
            assert_eq!(combinations_of_set(set_size, 1), 4); // O OOO
            assert_eq!(combinations_of_set(set_size, 2), 9); // O OOO OOOOO
            assert_eq!(combinations_of_set(set_size, 3), 16); // O OOO OOOOO OOOOOOO

            let set_size = 3;
            assert_eq!(combinations_of_set(set_size, 0), 1); // O
            assert_eq!(combinations_of_set(set_size, 1), 5); // O OOOO
            assert_eq!(combinations_of_set(set_size, 2), 12); // O OOOO OOOOOOO
            assert_eq!(combinations_of_set(set_size, 3), 22); // O OOOO OOOOOOO OOOOOOOOOO
        }

        #[test]
        fn test_grouping_of_valid_conditions() {
            let conditions = [Condition::Bad];
            assert_eq!(conditions_to_groups(&conditions), vec![1]);
            let conditions = [Condition::Bad, Condition::Bad];
            assert_eq!(conditions_to_groups(&conditions), vec![2]);
            let conditions = [Condition::Bad, Condition::Good, Condition::Bad];
            assert_eq!(conditions_to_groups(&conditions), vec![1, 1]);
            let conditions = [
                Condition::Good,
                Condition::Bad,
                Condition::Good,
                Condition::Bad,
                Condition::Bad,
                Condition::Good,
            ];
            assert_eq!(conditions_to_groups(&conditions), vec![1, 2]);
        }

        #[test]
        fn test_parse_condition_report() {
            let input = ".??..??...?##. 1,1,3";
            let report = parse_condition_report(input).unwrap().1;
            assert_eq!(report.known_conditions.len(), 14);
            assert_eq!(report.groups, vec![1, 1, 3]);
        }

        #[test]
        fn test_get_possible_broken_group_sizes() {
            let input = ".??..??...?##. 1,1,3";
            let report = parse_condition_report(input).unwrap().1;
            let groups = report.get_possible_broken_groups();
            assert_eq!(groups[0], vec![None, None]);
            assert_eq!(groups[1], vec![None, None]);
            assert_eq!(
                groups[2],
                vec![None, Some(Condition::Bad), Some(Condition::Bad)]
            );
        }

        #[test]
        fn test_validate_possible_conditions() {
            let report = ConditionReport {
                known_conditions: vec![Some(Condition::Bad), None, Some(Condition::Good), None],
                groups: vec![1, 1],
            };

            // Valid
            assert!(report.validate_possible_conditions(vec![
                Condition::Bad,
                Condition::Good,
                Condition::Good,
                Condition::Bad
            ]));

            // Invalid
            // Group is 1,2
            assert!(!report.validate_possible_conditions(vec![
                Condition::Bad,
                Condition::Good,
                Condition::Bad,
                Condition::Bad
            ]));
            // Group is 2,1
            assert!(!report.validate_possible_conditions(vec![
                Condition::Bad,
                Condition::Bad,
                Condition::Good,
                Condition::Bad
            ]));
            // Group is 1
            assert!(!report.validate_possible_conditions(vec![
                Condition::Bad,
                Condition::Good,
                Condition::Good,
                Condition::Good
            ]));
            // Good is Bad
            assert!(!report.validate_possible_conditions(vec![
                Condition::Bad,
                Condition::Good,
                Condition::Bad,
                Condition::Good
            ]));
            // Bad is Good
            assert!(!report.validate_possible_conditions(vec![
                Condition::Good,
                Condition::Bad,
                Condition::Good,
                Condition::Bad
            ]));
        }
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
        assert_eq!(part1(input), "12")
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "")
    }
}
