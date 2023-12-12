use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::{self, char, space1};
use nom::combinator::{map, value};
use nom::IResult;
use nom::multi::{many1, separated_list1};
use nom::sequence::separated_pair;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Condition {
    Good,
    Bad,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
struct ConditionReport {
    conditions: Vec<Condition>,
    groups: Vec<u64>,
}

impl ConditionReport {
    fn get_possible_broken_group_sizes(&self) -> Vec<usize> {
        self.conditions.iter()
            .group_by(|condition| *condition != &Condition::Good).into_iter()
            .filter(|(key, _value)| *key)
            .map(|(_key, value)| value.into_iter().count())
            .collect()
    }

    fn n_ways_to_fit_in_set(set: usize, inner_set: &[u64]) -> Vec<(usize, Vec<u64>)> {
        if inner_set.is_empty() {
            return vec![];
        }

        let mut result = Vec::new();

        // First, number of ways the full set fits in, pop an item and do it again
        let inner_set_size = inner_set.iter().sum() + inner_set.len() - 1;

        if set > inner_set_size {
            let jiggle_room = set - inner_set_size;.,
        }

        let (removed, lesser_set) = inner_set.split_last().unwrap();

        let sub_sets = ConditionReport::n_ways_to_fit_in_set(set, lesser_set)
            .into_iter().map(|(count, mut remainder)| remainder.push(*removed)).collect();
        result.append(sub_sets);

        result
    }

    fn n_possible_combinations(&self) -> usize {
        let possible_group_sizes = self.get_possible_broken_group_sizes();
    }
}

impl From<(Vec<Condition>, Vec<u64>)> for ConditionReport {
    fn from((conditions, groups): (Vec<Condition>, Vec<u64>)) -> Self {
        Self { conditions, groups }
    }
}

fn parse_condition(input: &str) -> IResult<&str, Condition> {
    alt((
        value(Condition::Good, char('.')),
        value(Condition::Bad, char('#')),
        value(Condition::Unknown, char('?')),
    ))(input)
}

fn parse_condition_report(input: &str) -> IResult<&str, ConditionReport> {
    map(separated_pair(many1(parse_condition), space1, separated_list1(char(','), complete::u64)), |parts| parts.into())(input)
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

    mod parts {
        use super::*;

        #[test]
        fn test_parse_condition_report() {
            let input = ".??..??...?##. 1,1,3";
            let report = parse_condition_report(input).unwrap().1;
            assert_eq!(report.conditions.len(), 14);
            assert_eq!(report.groups, vec![1, 1, 3]);
        }

        #[test]
        fn test_get_possible_broken_group_sizes() {
            let input = ".??..??...?##. 1,1,3";
            let report = parse_condition_report(input).unwrap().1;
            assert_eq!(report.get_possible_broken_group_sizes(), vec![2, 2, 3])
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
