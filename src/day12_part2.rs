use std::iter;

use bitvec::field::BitField;
use bitvec::prelude::BitVec;
use bitvec::view::BitView;
use indicatif::ProgressIterator;
use itertools::Itertools;
use nom::bytes::complete::{take_while, take_while1};
use nom::combinator::map;
use nom::multi::separated_list0;
use nom::sequence::delimited;
use nom::IResult;
use rayon::prelude::*;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
struct Group {
    number: u128,
    length: usize,
}

impl From<&str> for Group {
    fn from(value: &str) -> Self {
        let length = value.len();
        let bitvec: BitVec = value.chars().map(|c| c == '#').into_iter().rev().collect();
        let number = bitvec.load();

        Self { number, length }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Clone)]
struct RawConditionReport<'a>(Vec<&'a str>);

#[derive(Debug)]
struct ConditionReport(Vec<Group>);

impl From<Vec<Group>> for ConditionReport {
    fn from(value: Vec<Group>) -> Self {
        Self(value)
    }
}

fn is_bad_or_unknown(c: char) -> bool {
    c == '#' || c == '?'
}

fn is_good(c: char) -> bool {
    c == '.'
}

fn parse_bad_or_unknown_group(input: &str) -> IResult<&str, &str> {
    take_while1(is_bad_or_unknown)(input)
}

fn parse_ok_group(input: &str) -> IResult<&str, &str> {
    take_while(is_good)(input)
}

fn parse_raw_condition_report(input: &str) -> IResult<&str, RawConditionReport> {
    map(
        delimited(
            parse_ok_group,
            separated_list0(parse_ok_group, parse_bad_or_unknown_group),
            parse_ok_group,
        ),
        |v| RawConditionReport(v),
    )(input)
}

pub fn part2(input: &str) -> String {
    let bigger_input: Vec<_> = input
        .lines()
        .map(|line| iter::repeat(line).take(5).collect::<String>())
        .collect();

    let reports_as_groups: Vec<ConditionReport> = bigger_input
        .iter()
        .map(|line| parse_raw_condition_report(line))
        .map(|result| result.expect("invalid report").1)
        .map(|report| {
            report
                .0
                .iter()
                .map(|group_str| Group::from(*group_str))
                .collect::<Vec<_>>()
                .into()
        })
        .collect::<Vec<_>>();

    "oh god".to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    mod parser {
        use super::*;

        #[test]
        fn test_parse_bad_or_unknown_group() {
            assert_eq!(parse_bad_or_unknown_group("#?##.."), Ok(("..", "#?##")))
        }

        #[test]
        fn test_parse_ok_group() {
            assert_eq!(parse_ok_group("...#"), Ok(("#", "...")));
            assert_eq!(parse_ok_group(""), Ok(("", "")));
        }

        #[test]
        fn test_parse_condition_report() {
            assert_eq!(
                parse_raw_condition_report("..##??..?#.."),
                Ok(("", RawConditionReport(vec!["##??", "?#"])))
            );
        }
    }

    mod group {
        use super::*;

        #[test]
        fn test_group() {
            let group_str = "#?##";
            let group: Group = group_str.into();
            let expected = Group {
                number: 11,
                length: 4,
            };
            assert_eq!(group, expected);
        }
    }

    // #[test]
    //     fn test_part1() {
    //         let input = "???.### 1,1,3
    // .??..??...?##. 1,1,3
    // ?#?#?#?#?#?#?#? 1,3,1,6
    // ????.#...#... 4,1,1
    // ????.######..#####. 1,6,5
    // ?###???????? 3,2,1";
    //         assert_eq!(part1(input), "21")
    //     }

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
