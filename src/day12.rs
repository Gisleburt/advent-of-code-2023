use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use indicatif::ProgressIterator;
use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::{self, char, newline, space1};
use nom::combinator::{map, value};
use nom::multi::{many1, separated_list1};
use nom::sequence::separated_pair;
use nom::IResult;
use rayon::prelude::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Condition {
    Good,
    Bad,
}

type Groups = Vec<u64>;

#[derive(Debug, Clone, PartialEq)]
struct ConditionReport {
    conditions: Vec<Option<Condition>>,
    groups: Groups,
    good_number: u32,
    bad_number: u32,
}

impl ConditionReport {
    fn get_good_number(conditions: &[Option<Condition>]) -> u32 {
        let bitvec: BitVec = conditions
            .iter()
            .map(|condition| *condition == Some(Condition::Good))
            .rev()
            .collect();
        bitvec.load()
    }
    fn get_bad_number(conditions: &[Option<Condition>]) -> u32 {
        let bitvec: BitVec = conditions
            .iter()
            .map(|condition| *condition == Some(Condition::Bad))
            .rev()
            .collect();
        bitvec.load()
    }

    fn new(conditions: Vec<Option<Condition>>, groups: Groups) -> Self {
        let good_number = Self::get_good_number(&conditions);
        let bad_number = Self::get_bad_number(&conditions);
        Self {
            conditions,
            good_number,
            bad_number,
            groups,
        }
    }

    // 1101
    // 1001
    // 0010

    fn could_number_fit(&self, number: u32) -> bool {
        number_to_groups(number) == self.groups
            && (number & self.bad_number == self.bad_number)
            && (!number & self.good_number == self.good_number)
    }

    fn get_possible_broken_groups(&self) -> Vec<Vec<Option<Condition>>> {
        get_bad_and_unknown_conditions(&self.conditions)
    }

    fn validate_possible_conditions(&self, possible_conditions: Vec<Condition>) -> bool {
        conditions_to_groups(&possible_conditions) == self.groups
            && validate_possible_conditions(&self.conditions, &possible_conditions)
    }

    fn find_possible_arrangements(&self) -> usize {
        (0..(2_u32.pow(self.conditions.len() as u32)))
            .into_par_iter()
            .filter(|test| self.could_number_fit(*test as u32))
            .count()
    }
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

fn number_to_groups(number: u32) -> Groups {
    let bitvec: BitVec = number.view_bits::<Msb0>().iter().collect();

    let result = bitvec
        .into_iter()
        .group_by(|bit| *bit)
        .into_iter()
        .filter(|(key, _value)| *key)
        .map(|(_key, value)| value.count() as u64)
        .collect();
    result
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

impl From<(Vec<Option<Condition>>, Groups)> for ConditionReport {
    fn from((known_conditions, groups): (Vec<Option<Condition>>, Groups)) -> Self {
        Self::new(known_conditions, groups)
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

fn parse_condition_reports(input: &str) -> IResult<&str, Vec<ConditionReport>> {
    separated_list1(newline, parse_condition_report)(input)
}

fn input_to_report(input: &str) -> ConditionReport {
    parse_condition_report(input).unwrap().1
}

fn input_to_reports(input: &str) -> Vec<ConditionReport> {
    parse_condition_reports(input).unwrap().1
}

pub fn part1(input: &str) -> String {
    let reports = input_to_reports(input);
    reports
        .into_par_iter()
        .map(|report| report.find_possible_arrangements())
        .sum::<usize>()
        .to_string()
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
            assert_eq!(report.conditions.len(), 14);
            assert_eq!(report.groups, vec![1, 1, 3]);
        }

        #[test]
        fn test_get_possible_broken_group_sizes() {
            let report = input_to_report(".??..??...?##. 1,1,3");
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
            let report = ConditionReport::new(
                vec![Some(Condition::Bad), None, Some(Condition::Good), None],
                vec![1, 1],
            );

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

        #[test]
        fn test_condition_report_numbers() {
            let conditions = vec![
                Some(Condition::Good),
                Some(Condition::Good),
                None,
                Some(Condition::Bad),
            ];
            let num = ConditionReport::new(conditions, vec![]);

            assert_eq!(num.good_number, 12);
            assert_eq!(num.bad_number, 1);
        }

        #[test]
        fn test_could_number_fit() {
            let report = input_to_report(".??..??...?##. 1,1,3");

            let number = 0b01000100001110;
            assert!(report.could_number_fit(number));
            let number = 0b00100100001110;
            assert!(report.could_number_fit(number));
            let number = 0b01000010001110;
            assert!(report.could_number_fit(number));
            let number = 0b00100010001110;
            assert!(report.could_number_fit(number));
        }

        #[test]
        fn test_find_possible_conditions() {
            let report = input_to_report("???.### 1,1,3");
            assert_eq!(report.find_possible_arrangements(), 1);
            let report = input_to_report(".??..??...?##. 1,1,3");
            assert_eq!(report.find_possible_arrangements(), 4);
            let report = input_to_report("?#?#?#?#?#?#?#? 1,3,1,6");
            assert_eq!(report.find_possible_arrangements(), 1);
            let report = input_to_report("????.######..#####. 1,6,5");
            assert_eq!(report.find_possible_arrangements(), 4);
            let report = input_to_report("?###???????? 3,2,1");
            assert_eq!(report.find_possible_arrangements(), 10);
        }

        #[test]
        fn test_number_to_groups() {
            assert_eq!(number_to_groups(5), vec![1, 1]);
            assert_eq!(number_to_groups(13), vec![2, 1]);
        }

        #[test]
        fn test_must_may_contain() {
            // let report = input_to_report(".??..??...?##. 1,1,3");
            // assert_eq!(
            //     report.must_contain(),
            //     vec![vec![], vec![], vec![vec![2], vec![3]]]
            // );
            // assert_eq!(
            //     report.may_contaon(),
            //     vec![
            //         vec![vec![1], vec![1]],
            //         vec![vec![1], vec![1]],
            //         vec![vec![2], vec![3]]
            //     ]
            // );
        }
    }

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
