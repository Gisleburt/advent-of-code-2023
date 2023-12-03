#[derive(Copy, Clone, Debug)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Copy, Clone, Debug)]
struct Number {
    position: Position,
    value: usize,
    len: usize,
}

impl Number {
    pub fn is_adjacent(&self, other: Position) -> bool {
        other.x >= self.position.x.saturating_sub(1)
            && other.x <= self.position.x.saturating_add(self.len)
            && other.y >= self.position.y.saturating_sub(1)
            && other.y <= self.position.y.saturating_add(1)
    }
}

#[derive(Copy, Clone, Debug)]
struct Symbol {
    position: Position,
    symbol: char,
}

#[derive(Default, Debug)]
struct Grid {
    numbers: Vec<Number>,
    symbols: Vec<Symbol>,
}

impl Grid {
    pub fn get_missing_engine_part(&self) -> usize {
        self.numbers
            .iter()
            .filter(|n| self.symbols.iter().any(|s| n.is_adjacent(s.position)))
            .map(|n| n.value)
            .sum()
    }

    pub fn get_gear_ratios(&self) -> Vec<usize> {
        self.symbols
            .iter()
            .filter(|s| s.symbol == '*')
            .map(|s| {
                self.numbers
                    .iter()
                    .filter(|n| n.is_adjacent(s.position))
                    .collect::<Vec<_>>()
            })
            .filter(|n| n.len() == 2)
            .map(|n| n[0].value * n[1].value)
            .collect()
    }
}

fn fill_grid(input: &str) -> Grid {
    let mut grid = Grid::default();

    input.lines().enumerate().for_each(|(y, line)| {
        let mut iter = line.chars().enumerate().peekable();
        while let Some((x, char)) = iter.next() {
            if char == '.' {
                continue;
            }

            let position = Position { x, y };

            if char.is_numeric() {
                let mut number = String::new();
                number.push(char);

                while iter.peek().map(|(_, c)| c.is_numeric()) == Some(true) {
                    number.push(iter.next().map(|(_, c)| c).unwrap())
                }
                let len = number.len();
                let number = Number {
                    position,
                    value: number.parse().unwrap(),
                    len,
                };
                grid.numbers.push(number);
            } else {
                let symbol = Symbol {
                    position,
                    symbol: char,
                };
                grid.symbols.push(symbol);
            }
        }
    });
    grid
}

pub fn part1(input: &str) -> String {
    let grid = fill_grid(input);
    grid.get_missing_engine_part().to_string()
}

pub fn part2(input: &str) -> String {
    let grid = fill_grid(input);
    grid.get_gear_ratios().iter().sum::<usize>().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";
        assert_eq!(part1(input), "4361");
    }

    #[test]
    fn test_part2() {
        let input = "467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";
        assert_eq!(part2(input), "467835")
    }
}
