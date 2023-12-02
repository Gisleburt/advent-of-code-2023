use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, space0};
use nom::multi::separated_list1;
use nom::sequence::tuple;
use nom::IResult;
use std::cmp::max;

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
enum Color {
    Red(usize),
    Green(usize),
    Blue(usize),
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
struct Set {
    red: usize,
    green: usize,
    blue: usize,
}

impl From<Vec<Color>> for Set {
    fn from(value: Vec<Color>) -> Self {
        let mut set = Set::default();
        for color in value {
            match color {
                Color::Red(red) => set.red = red,
                Color::Green(green) => set.green = green,
                Color::Blue(blue) => set.blue = blue,
            }
        }
        set
    }
}

impl Set {
    fn from_raw(red: usize, green: usize, blue: usize) -> Self {
        Self { red, green, blue }
    }

    fn contains(&self, other: &Self) -> bool {
        self.red >= other.red && self.blue >= other.blue && self.green >= other.green
    }

    fn power(&self) -> usize {
        self.red * self.green * self.blue
    }
}

#[derive(Default, Debug, Clone)]
struct Game {
    number: usize,
    sets: Vec<Set>,
}

impl Game {
    fn from_raw(number: usize, sets: Vec<Set>) -> Self {
        Game { number, sets }
    }

    fn is_possible(&self, test_set: &Set) -> bool {
        self.sets.iter().all(|game_set| test_set.contains(game_set))
    }

    fn min_set(&self) -> Set {
        self.sets.iter().fold(Set::default(), |acc, cur| {
            Set::from_raw(
                max(acc.red, cur.red),
                max(acc.green, cur.green),
                max(acc.blue, cur.blue),
            )
        })
    }
}

/// ```rust
/// assert_eq!(true, true);
/// ```
fn parse_red(input: &str) -> IResult<&str, Color> {
    let (remainder, (red, _, _)) = tuple((digit1, space0, tag("red")))(input)?;
    Ok((remainder, Color::Red(red.parse().unwrap())))
}

fn parse_green(input: &str) -> IResult<&str, Color> {
    let (remainder, (green, _, _)) = tuple((digit1, space0, tag("green")))(input)?;
    Ok((remainder, Color::Green(green.parse().unwrap())))
}

fn parse_blue(input: &str) -> IResult<&str, Color> {
    let (remainder, (blue, _, _)) = tuple((digit1, space0, tag("blue")))(input)?;
    Ok((remainder, Color::Blue(blue.parse().unwrap())))
}

fn parse_color(input: &str) -> IResult<&str, Color> {
    alt((parse_red, parse_green, parse_blue))(input)
}

fn parse_set(input: &str) -> IResult<&str, Set> {
    let (remainder, colors) = separated_list1(tag(", "), parse_color)(input)?;
    Ok((remainder, colors.into()))
}

fn parse_game_number(input: &str) -> IResult<&str, usize> {
    let (remainder, (_, num, _)) = tuple((tag("Game "), digit1, tag(": ")))(input)?;
    Ok((remainder, num.parse().unwrap()))
}

fn parse_game(input: &str) -> IResult<&str, Game> {
    let (remainder, (numnber, colors)) =
        tuple((parse_game_number, separated_list1(tag("; "), parse_set)))(input)?;
    Ok((remainder, Game::from_raw(numnber, colors)))
}

pub fn part1(input: &str) -> String {
    let test_set = Set::from_raw(12, 13, 14);
    input
        .lines()
        .map(|line| parse_game(line).unwrap().1)
        .filter(|game| game.is_possible(&test_set))
        .map(|game| game.number)
        .sum::<usize>()
        .to_string()
}

pub fn part2(input: &str) -> String {
    input
        .lines()
        .map(|line| parse_game(line).unwrap().1)
        .map(|game| game.min_set())
        .map(|set| set.power())
        .sum::<usize>()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_game_is_possible() {
        let set_1 = Set::from_raw(1, 1, 1);
        let set_2 = Set::from_raw(1, 2, 3);
        let set_3 = Set::from_raw(3, 2, 1);

        let game_1 = Game::from_raw(1, vec![set_1]);
        assert!(game_1.is_possible(&set_2));

        let game_2 = Game::from_raw(2, vec![set_1, set_2]);
        assert!(game_2.is_possible(&set_2));

        // Not possible for game 3 to been made with set 3
        let game_3 = Game::from_raw(3, vec![set_1, set_2]);
        assert!(!game_3.is_possible(&set_3));
    }

    #[test]
    fn test_parse_color() {
        let red = "3 red";
        let green = "2 green";
        let blue = "1 blue";

        assert_eq!(parse_color(red), Ok(("", Color::Red(3))));
        assert_eq!(parse_color(green), Ok(("", Color::Green(2))));
        assert_eq!(parse_color(blue), Ok(("", Color::Blue(1))));
    }

    #[test]
    fn test_parse_set() {
        let set_1 = "3 red, 2 green, 1 blue";
        let set_2 = "3 red, 2 green";
        let set_3 = "2 green, 3 red";

        assert_eq!(parse_set(set_1), Ok(("", Set::from_raw(3, 2, 1))));
        assert_eq!(parse_set(set_2), Ok(("", Set::from_raw(3, 2, 0))));
        assert_eq!(parse_set(set_3), Ok(("", Set::from_raw(3, 2, 0))));
    }

    #[test]
    fn test_parse_game() {
        let game = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green";
        let (_, parsed_game) = parse_game(game).unwrap();

        let game_number = 1;
        let set_1 = Set::from_raw(4, 0, 3);
        let set_2 = Set::from_raw(1, 2, 6);
        let set_3 = Set::from_raw(0, 2, 0);

        assert_eq!(parsed_game.number, game_number);
        assert!(parsed_game.sets.contains(&set_1));
        assert!(parsed_game.sets.contains(&set_2));
        assert!(parsed_game.sets.contains(&set_3));
    }

    #[test]
    fn test_part1() {
        let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";
        assert_eq!(part1(input), "8".to_string());
    }

    #[test]
    fn test_part2() {
        let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";
        assert_eq!(part2(input), "2286".to_string());
    }
}
