use std::collections::HashMap;

use itertools::Itertools;
use nom::bytes::complete::is_not;
use nom::character::complete;
use nom::multi::separated_list1;
use nom::IResult;

fn hash(input: &str) -> usize {
    input
        .bytes()
        .map(|byte| byte as usize)
        .fold(0_usize, |acc, cur| ((acc + cur) * 17) % 256)
}

fn parse_steps(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list1(complete::char(','), is_not(",\n"))(input)
}

pub fn part1(input: &str) -> String {
    let v = parse_steps(input).unwrap().1;
    v.into_iter().map(hash).sum::<usize>().to_string()
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
struct Lens {
    label: String,
    focal_length: usize,
}

impl Lens {
    fn new(value: &str) -> Self {
        let (label, focal_length) = value.split_once('=').unwrap();
        Self {
            label: label.to_string(),
            focal_length: focal_length.parse().unwrap(),
        }
    }

    fn get_hash(&self) -> usize {
        hash(&self.label)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Instruction {
    AddLens(Lens),
    RemoveLens(String),
}

impl From<&str> for Instruction {
    fn from(value: &str) -> Self {
        if value.contains('=') {
            Instruction::AddLens(Lens::new(value))
        } else if value.ends_with('-') {
            Instruction::RemoveLens(value.trim_end_matches('-').to_string())
        } else {
            panic!("{value} did not contain = or end with -")
        }
    }
}

struct Box(Vec<Lens>);

impl Box {
    fn new() -> Self {
        Self(vec![])
    }

    fn add_lens(&mut self, lens: Lens) {
        if let Some((pos, _)) = self.0.iter().find_position(|l| l.label == lens.label) {
            let _ = std::mem::replace(&mut self.0[pos], lens);
        } else {
            self.0.push(lens);
        }
    }

    fn remove_lens(&mut self, label: &str) {
        if let Some((pos, _)) = self.0.iter().find_position(|l| l.label == label) {
            self.0.remove(pos);
        }
    }
}

struct Boxes(HashMap<usize, Box>);

impl Boxes {
    fn new() -> Self {
        Boxes(HashMap::new())
    }

    fn apply(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::AddLens(lens) => {
                let bx = self.0.entry(lens.get_hash()).or_insert(Box::new());
                bx.add_lens(lens);
            }
            Instruction::RemoveLens(label) => {
                let bx = self.0.entry(hash(&label)).or_insert(Box::new());
                bx.remove_lens(&label);
            }
        }
    }
}

pub fn part2(input: &str) -> String {
    parse_steps(input)
        .unwrap()
        .1
        .into_iter()
        .map(Instruction::from)
        .fold(Boxes::new(), |mut boxes, instruction| {
            boxes.apply(instruction);
            boxes
        })
        .0
        .into_iter()
        .sorted_by_key(|(hash, _bx)| *hash)
        .flat_map(|(h, bx)| {
            bx.0.into_iter().enumerate().map(move |(slot, lens)| {
                let box_n = h + 1;
                let slot_n = slot + 1;
                let focal_length = lens.focal_length;
                // let focusing_power = box_n * slot_n * focal_length;
                // let label = &lens.label;
                // println!("{label}: {box_n} (box {h}) * {slot_n} (slot) * {focal_length} (focal length) = {focusing_power}");
                // focusing_power
                box_n * slot_n * focal_length
            })
        })
        .sum::<usize>()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash() {
        assert_eq!(hash("HASH"), 52);
        assert_eq!(hash("ot"), 3);
    }

    #[test]
    fn test_part1() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
        assert_eq!(part1(input), "1320");
    }

    #[test]
    fn test_instruction_from() {
        let instruction = "rn=1";
        assert_eq!(
            Instruction::from(instruction),
            Instruction::AddLens(Lens {
                label: "rn".to_string(),
                focal_length: 1,
            })
        );
        let instruction = "cm-";
        assert_eq!(
            Instruction::from(instruction),
            Instruction::RemoveLens("cm".to_string())
        )
    }

    #[test]
    fn test_part2() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
        assert_eq!(part2(input), "145");
    }
}
