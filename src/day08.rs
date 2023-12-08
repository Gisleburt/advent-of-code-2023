use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, newline};
use nom::sequence::{delimited, separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
struct MapTo<'a> {
    left: &'a str,
    right: &'a str,
}

fn parse_instructions(input: &str) -> IResult<&str, &str> {
    terminated(alpha1, tuple((newline, newline)))(input)
}

fn parse_map_to(input: &str) -> IResult<&str, MapTo> {
    let (r, (left, right)) = delimited(
        tag("("),
        separated_pair(alpha1, tag(", "), alpha1),
        tag(")"),
    )(input)?;
    Ok((r, MapTo { left, right }))
}

fn parse_mapping(input: &str) -> IResult<&str, (&str, MapTo)> {
    separated_pair(alpha1, tag(" = "), parse_map_to)(input)
}

pub fn part1(input: &str) -> String {
    let (remainder, instructions) = parse_instructions(input).unwrap();
    let map: HashMap<_, _> = remainder
        .lines()
        .map(|line| parse_mapping(line).unwrap().1)
        .collect();
    let mut current_position = "AAA";
    let mut steps = 0;
    loop {
        for instruction in instructions.chars() {
            steps += 1;
            let next_choice = map.get(current_position).unwrap();
            match instruction {
                'L' => current_position = next_choice.left,
                'R' => current_position = next_choice.right,
                _ => panic!("Unexpected instruction {instruction}"),
            }
            if current_position == "ZZZ" {
                return steps.to_string();
            }
        }
    }
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
        fn test_parse_instructions() {
            let input = "RL

AAA = (BBB, CCC)";
            assert_eq!(parse_instructions(input), Ok(("AAA = (BBB, CCC)", "RL")))
        }
        #[test]
        fn test_parse_map_to() {
            let input = "(BBB, CCC)\n";
            assert_eq!(
                parse_map_to(input),
                Ok((
                    "\n",
                    MapTo {
                        left: "BBB",
                        right: "CCC"
                    }
                ))
            )
        }
        #[test]
        fn test_parse_mapping() {
            let input = "AAA = (BBB, CCC)
BBB = (DDD, EEE)";
            assert_eq!(
                parse_mapping(input),
                Ok((
                    "\nBBB = (DDD, EEE)",
                    (
                        "AAA",
                        MapTo {
                            left: "BBB",
                            right: "CCC"
                        }
                    )
                ))
            )
        }
    }

    #[test]
    fn test_part1() {
        let input = "RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)";
        assert_eq!(part1(input), "2")
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "")
    }
}
