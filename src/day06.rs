use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{self, digit1, newline};
use nom::multi::many1;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;

#[derive(Debug, PartialEq)]
struct TimeAndDistance {
    time: u32,
    distance: u32,
}

impl TimeAndDistance {
    fn distance_travelled(&self, held: u32) -> u32 {
        self.time.saturating_sub(held).saturating_mul(held)
    }

    fn winning_possbilities(&self) -> u32 {
        (1..(self.time - 1))
            .map(|t| self.distance_travelled(t))
            .skip_while(|d| *d <= self.distance)
            .take_while(|d| *d > self.distance)
            .count() as u32
    }
}

fn parse_numbers(input: &str) -> IResult<&str, Vec<u32>> {
    many1(preceded(take_while(char::is_whitespace), complete::u32))(input)
}

fn parse_time(input: &str) -> IResult<&str, Vec<u32>> {
    preceded(tag("Time:"), parse_numbers)(input)
}

fn parse_distance(input: &str) -> IResult<&str, Vec<u32>> {
    preceded(tag("Distance:"), parse_numbers)(input)
}

fn input_into_time_and_distance(input: &str) -> Vec<TimeAndDistance> {
    let (_, (times, distances)) =
        separated_pair(parse_time, newline, parse_distance)(input).unwrap();

    times
        .into_iter()
        .zip(distances.into_iter())
        .map(|(time, distance)| TimeAndDistance { time, distance })
        .collect()
}

fn parse_numbers2(input: &str) -> IResult<&str, u32> {
    let (remainder, strings) = many1(preceded(take_while(char::is_whitespace), digit1))(input)?;
    Ok((remainder, strings.join("").parse().unwrap()))
}

fn parse_time2(input: &str) -> IResult<&str, u32> {
    preceded(tag("Time:"), parse_numbers2)(input)
}

fn parse_distance2(input: &str) -> IResult<&str, u32> {
    preceded(tag("Distance:"), parse_numbers2)(input)
}

fn input_into_time_and_distance2(input: &str) -> TimeAndDistance {
    let (_, (time, distance)) =
        separated_pair(parse_time2, newline, parse_distance2)(input).unwrap();
    TimeAndDistance { time, distance }
}

pub fn part1(input: &str) -> String {
    input_into_time_and_distance(input)
        .into_iter()
        .map(|dt| dt.winning_possbilities())
        .product::<u32>()
        .to_string()
}

pub fn part2(input: &str) -> String {
    input_into_time_and_distance2(input)
        .winning_possbilities()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_numbers() {
        let input = "  7  15   30";
        assert_eq!(parse_numbers(input).unwrap().1, vec![7, 15, 30])
    }

    #[test]
    fn test_parsers() {
        let input = "Time:      7  15   30
Distance:  9  40  200";
        assert_eq!(
            input_into_time_and_distance(input),
            vec![
                TimeAndDistance {
                    time: 7,
                    distance: 9
                },
                TimeAndDistance {
                    time: 15,
                    distance: 40
                },
                TimeAndDistance {
                    time: 30,
                    distance: 200
                },
            ]
        )
    }

    #[test]
    fn test_part1() {
        let input = "Time:      7  15   30
Distance:  9  40  200";
        assert_eq!(part1(input), "288")
    }

    #[test]
    fn test_part2() {
        let input = "Time:      7  15   30
Distance:  9  40  200";
        assert_eq!(part2(input), "71503")
    }
}
