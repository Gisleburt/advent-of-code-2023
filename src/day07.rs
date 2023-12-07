use nom::character::complete;
use nom::character::complete::space1;
use nom::multi::fill;
use nom::sequence::separated_pair;
use nom::IResult;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum CardValue {
    Wild, // For wild cards only
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl CardValue {
    fn as_wild_value(&self) -> Self {
        if *self == CardValue::Jack {
            CardValue::Wild
        } else {
            *self
        }
    }
}

impl From<char> for CardValue {
    fn from(c: char) -> Self {
        match c {
            '2' => CardValue::Two,
            '3' => CardValue::Three,
            '4' => CardValue::Four,
            '5' => CardValue::Five,
            '6' => CardValue::Six,
            '7' => CardValue::Seven,
            '8' => CardValue::Eight,
            '9' => CardValue::Nine,
            'T' => CardValue::Ten,
            'J' => CardValue::Jack,
            'Q' => CardValue::Queen,
            'K' => CardValue::King,
            'A' => CardValue::Ace,
            _ => panic!("invalid card found {c}"),
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Hand([CardValue; 5]);

impl Hand {
    fn get_hand_type(&self) -> HandType {
        let mut occurrences = HashMap::new();
        for card in self.0.iter() {
            *occurrences.entry(card).or_insert(0) += 1;
        }
        let mut occurrences: Vec<_> = occurrences
            .into_iter()
            .map(|(value, count)| (*value, count))
            .collect();
        occurrences.sort_by(|a, b| b.1.cmp(&a.1));
        let counts: Vec<&i32> = occurrences.iter().map(|(_, count)| count).collect();
        match counts[..] {
            [5] => HandType::FiveOfAKind,
            [4, 1] => HandType::FourOfAKind,
            [3, 2] => HandType::FullHouse,
            [3, 1, 1] => HandType::ThreeOfAKind,
            [2, 2, 1] => HandType::TwoPair,
            [2, 1, 1, 1] => HandType::OnePair,
            _ => HandType::HighCard,
        }
    }

    fn activate_wild_card(&self) -> WildHand {
        let mut occurrences = HashMap::new();
        for card in self.0.iter() {
            *occurrences.entry(card).or_insert(0) += 1;
        }

        let mut occurrences: Vec<_> = occurrences
            .into_iter()
            .filter(|(value, _)| **value != CardValue::Jack)
            .map(|(value, count)| (*value, count))
            .collect();
        occurrences.sort_by(|a, b| b.1.cmp(&a.1));

        let new_card = occurrences
            .first()
            .map(|(c, _)| c)
            .unwrap_or(&CardValue::Ace);

        let mut new_cards = self.0;
        new_cards
            .iter_mut()
            .filter(|v| **v == CardValue::Jack)
            .for_each(|j| *j = *new_card);
        WildHand {
            wild: Hand(new_cards),
            original: *self,
        }
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 == other.0 {
            return Ordering::Equal;
        }
        let self_type = self.get_hand_type();
        let other_type = other.get_hand_type();
        if self_type != other_type {
            self_type.cmp(&other_type)
        } else {
            let first_mismatch = self.0.iter().zip(other.0).find(|(a, b)| *a != b).unwrap();
            first_mismatch.0.cmp(&first_mismatch.1)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct WildHand {
    original: Hand,
    wild: Hand,
}

impl PartialOrd for WildHand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WildHand {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.original == other.original {
            return Ordering::Equal;
        }
        let self_type = self.wild.get_hand_type();
        let other_type = other.wild.get_hand_type();
        if self_type != other_type {
            self_type.cmp(&other_type)
        } else {
            let iter_1 = self.original.0.iter().map(|c| c.as_wild_value());
            let iter_2 = other.original.0.iter().map(|c| c.as_wild_value());
            let first_mismatch = iter_1.zip(iter_2).find(|(a, b)| a != b).unwrap();
            first_mismatch.0.cmp(&first_mismatch.1)
        }
    }
}

fn parse_card(input: &str) -> IResult<&str, CardValue> {
    let (r, c): (_, char) = complete::anychar(input)?;
    Ok((r, c.into()))
}

fn parse_hand(input: &str) -> IResult<&str, Hand> {
    let mut buf = [CardValue::Two; 5];
    let (r, ()) = fill(parse_card, &mut buf)(input)?;
    Ok((r, Hand(buf)))
}

fn parse_hand_and_bid(input: &str) -> IResult<&str, (Hand, u64)> {
    separated_pair(parse_hand, space1, complete::u64)(input)
}

pub fn part1(input: &str) -> String {
    let mut hands_and_bids: Vec<_> = input
        .lines()
        .map(|l| parse_hand_and_bid(l).unwrap())
        .map(|(_, hb)| hb)
        // .inspect(|x| {
        //     dbg!(x);
        // })
        .collect();
    hands_and_bids.sort_by_key(|hb| hb.0);
    hands_and_bids
        .iter()
        .enumerate()
        // .inspect(|(rank, (hand, bid))| {
        //     dbg!((rank, hand, bid));
        // })
        .map(|(rank, (_hand, bid))| (rank + 1) * (*bid as usize))
        .sum::<usize>()
        .to_string()
}

pub fn part2(input: &str) -> String {
    let mut hands_and_bids: Vec<_> = input
        .lines()
        .map(|l| parse_hand_and_bid(l).unwrap())
        .map(|(_, (hand, bid))| (hand.activate_wild_card(), bid))
        // .inspect(|x| {
        //     dbg!(x);
        // })
        .collect();
    hands_and_bids.sort_by_key(|hb| hb.0);
    hands_and_bids
        .into_iter()
        .enumerate()
        // .inspect(|x| {
        //     dbg!(x);
        // })
        .map(|(rank, (_hand, bid))| (rank + 1) * (bid as usize))
        .sum::<usize>()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    mod parts {

        use super::*;
        #[test]
        fn test_card_value_order() {
            assert!(CardValue::Ace > CardValue::Two)
        }

        #[test]
        fn test_hand_type_order() {
            assert!(HandType::FiveOfAKind > HandType::FourOfAKind)
        }

        #[test]
        fn test_parse_card() {
            assert_eq!(parse_card("32T3K 765"), Ok(("2T3K 765", CardValue::Three)))
        }

        #[test]
        fn test_parse_hand() {
            assert_eq!(
                parse_hand("32T3K 765"),
                Ok((
                    " 765",
                    Hand([
                        CardValue::Three,
                        CardValue::Two,
                        CardValue::Ten,
                        CardValue::Three,
                        CardValue::King
                    ])
                ))
            )
        }

        #[test]
        fn test_parse_hand_and_bid() {
            assert_eq!(
                parse_hand_and_bid("32T3K 765"),
                Ok((
                    "",
                    (
                        Hand([
                            CardValue::Three,
                            CardValue::Two,
                            CardValue::Ten,
                            CardValue::Three,
                            CardValue::King
                        ]),
                        765
                    )
                ))
            )
        }

        #[test]
        fn test_hand_order() {
            let hand1 = parse_hand("KK677").unwrap().1;
            let hand2 = parse_hand("KTJJT").unwrap().1;
            assert_eq!(hand1.get_hand_type(), HandType::TwoPair);
            assert_eq!(hand2.get_hand_type(), HandType::TwoPair);
            assert!(hand1 > hand2);
        }

        #[test]
        fn test_wild_hand_order() {
            let hand1 = parse_hand("QQQQ2").unwrap().1;
            let hand2 = parse_hand("JKKK2").unwrap().1;
            let wild_hand_1 = hand1.activate_wild_card();
            let wild_hand_2 = hand2.activate_wild_card();
            assert!(wild_hand_1 > wild_hand_2);
        }
    }

    #[test]
    fn test_part1() {
        let input = "32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";
        assert_eq!(part1(input), "6440")
    }

    #[test]
    fn test_part2() {
        let input = "32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";
        assert_eq!(part2(input), "5905")
    }
}
