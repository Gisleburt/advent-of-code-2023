use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Day1Error {
    #[error("Number not found in string")]
    NoNumberFound,
}

fn first_number(input: &str) -> Result<usize> {
    input
        .chars()
        .find(|c| c.is_numeric())
        .and_then(|c| (c as usize).checked_sub(48))
        .ok_or_else(|| Day1Error::NoNumberFound.into())
        .into()
}

fn last_number(input: &str) -> Result<usize> {
    input
        .chars()
        .rev()
        .find(|c| c.is_numeric())
        .and_then(|c| (c as usize).checked_sub(48))
        .ok_or_else(|| Day1Error::NoNumberFound.into())
}

pub fn day1part1(input: &str) -> String {
    input
        .lines()
        .map(|line| (first_number(line).unwrap(), last_number(line).unwrap()))
        .map(|(a, b)| (a * 10) + b)
        .sum::<usize>()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_day1part1() {
        let input = "1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet";
        assert_eq!(day1part1(input), "142");
    }
}
