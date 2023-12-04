use nom::bytes::complete::tag;
use nom::character::complete::{digit1, space1};
use nom::multi::separated_list1;
use nom::sequence::{delimited, separated_pair, tuple};
use nom::IResult;

#[derive(Debug)]
struct Card {
    number: u32,
    winning_numbers: Vec<u32>,
    card_numbers: Vec<u32>,
}

impl Card {
    fn score(&self) -> usize {
        let matches = self
            .winning_numbers
            .iter()
            .filter(|w| self.card_numbers.contains(w))
            .count();
        if matches > 0 {
            2usize.pow(matches as u32 - 1)
        } else {
            0
        }
    }
}

fn parse_numbers(numbers: &str) -> IResult<&str, Vec<u32>> {
    separated_list1(space1, digit1)(numbers.trim()).map(|(remainder, vec)| {
        (
            remainder,
            vec.into_iter().map(|s| s.parse().unwrap()).collect(),
        )
    })
}

fn parse_card(card: &str) -> IResult<&str, Card> {
    let (remainder, (number, (winning_numbers, card_numbers))) = tuple((
        delimited(
            tuple((tag("Card"), space1)),
            digit1,
            tuple((tag(":"), space1)),
        ),
        separated_pair(parse_numbers, tag(" | "), parse_numbers),
    ))(card)?;

    Ok((
        remainder,
        Card {
            number: number.parse().unwrap(),
            winning_numbers,
            card_numbers,
        },
    ))
}

pub fn part1(input: &str) -> String {
    input
        .lines()
        .map(|line| parse_card(line).unwrap())
        .map(|(_, c)| c.score())
        .sum::<usize>()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";
        assert_eq!(part1(input), "13");
    }

    #[test]
    fn test_card_score() {
        let card = Card {
            number: 1,
            winning_numbers: vec![41, 48, 83, 86, 17],
            card_numbers: vec![83, 86, 6, 31, 17, 9, 48, 53],
        };
        assert_eq!(card.score(), 8);
    }
}
