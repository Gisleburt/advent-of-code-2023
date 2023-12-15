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

pub fn part2(_input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash() {
        let input = "HASH";
        assert_eq!(hash(input), 52);
    }

    #[test]
    fn test_part1() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
        assert_eq!(part1(input), "1320");
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "");
    }
}
