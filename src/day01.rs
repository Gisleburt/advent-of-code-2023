use anyhow::Result;
use nom::sequence::tuple;
use nom::{branch::alt, bytes::complete::tag, bytes::complete::take, combinator::value, IResult};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Day1Error {
    #[error("Number not found in string")]
    NoNumberFound,
}

fn first_number_char(input: &str) -> Result<usize> {
    input
        .chars()
        .find(|c| c.is_numeric())
        .and_then(|c| (c as usize).checked_sub(48))
        .ok_or_else(|| Day1Error::NoNumberFound.into())
        .into()
}

fn last_number_char(input: &str) -> Result<usize> {
    input
        .chars()
        .rev()
        .find(|c| c.is_numeric())
        .and_then(|c| (c as usize).checked_sub(48))
        .ok_or_else(|| Day1Error::NoNumberFound.into())
}

pub fn part1(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            (
                first_number_char(line).unwrap(),
                last_number_char(line).unwrap(),
            )
        })
        .map(|(a, b)| (a * 10) + b)
        .sum::<usize>()
        .to_string()
}

fn each_number(input: &str) -> Vec<usize> {
    let mut v = Vec::new();
    for p in 0..input.len() {
        let (_, (_, option)) = tuple((take(p), parse_numeric))(input).unwrap();
        if let Some(num) = option {
            v.push(num);
        }
    }
    v
}

fn parse_numeric(input: &str) -> IResult<&str, Option<usize>> {
    alt((
        value(Some(1), alt((tag("1"), tag("one")))),
        value(Some(2), alt((tag("2"), tag("two")))),
        value(Some(3), alt((tag("3"), tag("three")))),
        value(Some(4), alt((tag("4"), tag("four")))),
        value(Some(5), alt((tag("5"), tag("five")))),
        value(Some(6), alt((tag("6"), tag("six")))),
        value(Some(7), alt((tag("7"), tag("seven")))),
        value(Some(8), alt((tag("8"), tag("eight")))),
        value(Some(9), alt((tag("9"), tag("nine")))),
        value(None, take(1usize)),
    ))(input)
}

pub fn part2(input: &str) -> String {
    input
        .lines()
        .map(|l| each_number(l))
        .map(|v| {
            (
                v.iter().nth(0).copied().unwrap(),
                v.iter().rev().nth(0).copied().unwrap(),
            )
        })
        .map(|(a, b)| (a * 10) + b)
        .sum::<usize>()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet";
        assert_eq!(part1(input), "142");
    }

    #[test]
    fn test_part2() {
        let input = "two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen";
        assert_eq!(part2(input), "281")
    }

    #[test]
    fn test_parse_numeric() {
        assert_eq!(parse_numeric("1"), Ok(((""), Some(1))));
        assert_eq!(parse_numeric("a1"), Ok((("1"), None)));
        assert_eq!(parse_numeric("one2"), Ok((("2"), Some(1))));
    }

    #[test]
    fn test_each_number() {
        assert_eq!(each_number("oneight"), vec![1, 8]);
    }
}
