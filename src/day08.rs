use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, newline};
use nom::sequence::{delimited, separated_pair, terminated, tuple};
use nom::IResult;
use num::integer::lcm;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
struct MapTo<'a> {
    left: &'a str,
    right: &'a str,
}

struct HashMapping<'a>(HashMap<&'a str, MapTo<'a>>);

impl<'a> HashMapping<'a> {
    fn next_pos(&'a self, current_pos: &'_ str, instruction: char) -> &'a str {
        let next_choice = self
            .get(current_pos)
            .expect("position did not exist on map");
        match instruction {
            'L' => next_choice.left,
            'R' => next_choice.right,
            _ => panic!("Unexpected instruction {instruction}"),
        }
    }
}

impl<'a> Deref for HashMapping<'a> {
    type Target = HashMap<&'a str, MapTo<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn parse_instructions(input: &str) -> IResult<&str, &str> {
    terminated(alpha1, tuple((newline, newline)))(input)
}

fn parse_map_to(input: &str) -> IResult<&str, MapTo> {
    let (r, (left, right)) = delimited(
        tag("("),
        separated_pair(alphanumeric1, tag(", "), alphanumeric1),
        tag(")"),
    )(input)?;
    Ok((r, MapTo { left, right }))
}

fn parse_mapping(input: &str) -> IResult<&str, (&str, MapTo)> {
    separated_pair(alphanumeric1, tag(" = "), parse_map_to)(input)
}

pub fn part1(input: &str) -> String {
    let (remainder, instructions) = parse_instructions(input).unwrap();
    let map = HashMapping(
        remainder
            .lines()
            .map(|line| parse_mapping(line).unwrap().1)
            .collect(),
    );
    let mut current_position = "AAA";
    let mut steps = 0;
    loop {
        for instruction in instructions.chars() {
            steps += 1;
            current_position = map.next_pos(current_position, instruction);
            if current_position == "ZZZ" {
                return steps.to_string();
            }
        }
    }
}

fn is_finish(pos: &str) -> bool {
    pos.ends_with('Z')
}

fn is_start(pos: &str) -> bool {
    pos.ends_with('A')
}

// So it turns out that there is only one exit on each loop so we'll go a different function that
// just gets the first
//
// fn get_all_exists(start: &str, map: &HashMapping, instructions: &str) -> Vec<usize> {
//     let mut pos = start;
//     let mut exits = Vec::new();
//     let mut steps = 0;
//
//     let mut seen_steps = vec![(start, 0)];
//
//     loop {
//         for (inst_n, inst) in instructions.chars().enumerate() {
//             steps += 1;
//             pos = map.next_pos(pos, inst);
//
//             if is_finish(pos) {
//                 exits.push(steps);
//             }
//
//             if seen_steps.contains(&(pos, inst_n)) {
//                 return exits;
//             }
//             seen_steps.push((pos, inst_n));
//         }
//     }
// }

fn get_first_exit(start: &str, map: &HashMapping, instructions: &str) -> usize {
    let mut pos = start;
    let mut steps = 0;

    let mut seen_steps = vec![(start, 0)];

    loop {
        for (inst_n, inst) in instructions.chars().enumerate() {
            steps += 1;
            pos = map.next_pos(pos, inst);

            if is_finish(pos) {
                return steps;
            }

            seen_steps.push((pos, inst_n));
        }
    }
}

pub fn part2(input: &str) -> String {
    let (remainder, instructions) = parse_instructions(input).unwrap();
    let map = HashMapping(
        remainder
            .lines()
            .map(|line| parse_mapping(line).unwrap().1)
            .collect(),
    );

    map.keys()
        .copied()
        .filter(|key| is_start(key))
        .map(|start| get_first_exit(start, &map, instructions))
        .fold(None, |acc, cur| {
            if let Some(acc) = acc {
                Some(lcm(acc, cur))
            } else {
                Some(cur)
            }
        })
        .unwrap()
        .to_string()
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

    #[test]
    fn test_part2() {
        let input = "LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)";
        assert_eq!(part2(input), "6")
    }
}
