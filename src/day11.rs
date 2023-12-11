use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::{char, newline};
use nom::combinator::{map, value};
use nom::multi::{many1, separated_list1};
use nom::IResult;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, Div};

struct Image(Vec<Vec<Option<usize>>>);

impl Image {
    fn width(&self) -> usize {
        self[0].len()
    }

    fn height(&self) -> usize {
        self.len()
    }

    fn is_row_empty(&self, row: usize) -> bool {
        self[row].iter().all(|galaxy| galaxy.is_none())
    }

    fn is_column_empty(&self, column: usize) -> bool {
        self.iter()
            .map(|row| row[column])
            .all(|galaxy| galaxy.is_none())
    }

    fn expand_row(&mut self, row: usize) {
        self.0.insert(row, self[row].clone());
    }

    fn expand_column(&mut self, column: usize) {
        self.0.iter_mut().for_each(|row| {
            row.insert(column, row[column]);
        })
    }

    fn expand(&mut self) {
        // Expand rows
        let rows_to_expand: Vec<_> = (0..self.height())
            .into_iter()
            .rev() // Don't forget to work backwards
            .filter(|row| self.is_row_empty(*row))
            .collect();
        rows_to_expand
            .into_iter()
            .for_each(|row| self.expand_row(row));
        // Expand columns
        let columns_to_expand: Vec<_> = (0..self.width())
            .into_iter()
            .rev() // The expansion of space is no joke
            .filter(|column| self.is_column_empty(*column))
            .collect();
        columns_to_expand
            .into_iter()
            .for_each(|column| self.expand_column(column))
    }

    fn get_galaxies(&self) -> Vec<GalaxyLocation> {
        self.iter()
            .enumerate()
            .flat_map(|(row, data)| {
                data.iter()
                    .enumerate()
                    .filter_map(|(column, galaxy)| galaxy.map(|g| (column, g)))
                    .map(move |(column, galaxy)| GalaxyLocation::new(galaxy, row, column))
            })
            .collect()
    }
}

impl From<Vec<Vec<bool>>> for Image {
    fn from(value: Vec<Vec<bool>>) -> Self {
        let mut count = 0;
        Self(
            value
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|galaxy| {
                            galaxy.then(|| {
                                count += 1;
                                count
                            })
                        })
                        .collect()
                })
                .collect(),
        )
    }
}

impl Deref for Image {
    type Target = Vec<Vec<Option<usize>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|row| row
                    .iter()
                    .map(|galaxy| if galaxy.is_some() { "#" } else { "." })
                    .collect::<String>())
                .join("\n")
        )
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
struct GalaxyLocation {
    name: usize,
    row: usize,
    column: usize,
}

impl GalaxyLocation {
    fn new(name: usize, row: usize, column: usize) -> Self {
        Self { name, row, column }
    }

    fn distance_to(&self, other: &GalaxyLocation) -> usize {
        self.row.abs_diff(other.row) + self.column.abs_diff(other.column)
    }

    fn distances_to(&self, others: &Vec<GalaxyLocation>) -> GalacticDistances {
        GalacticDistances::new(*self, others)
    }
}

struct GalacticDistances {
    from: GalaxyLocation,
    distances: VecDeque<(usize, GalaxyLocation)>,
}

impl GalacticDistances {
    fn new(from: GalaxyLocation, galaxies: &Vec<GalaxyLocation>) -> Self {
        Self {
            from: from,
            distances: galaxies
                .iter()
                .copied()
                .filter(|other| other != &from)
                .map(|other| (from.distance_to(&other), other))
                .sorted_by_key(|pair| pair.0)
                .collect(),
        }
    }

    fn remove_pair(self, pair: &GalacticPair) -> Option<Self> {
        if pair.includes(&self.from) {
            None
        } else {
            Some(Self {
                from: self.from,
                distances: self
                    .distances
                    .into_iter()
                    .filter(|(_, location)| !pair.includes(&location))
                    .collect(),
            })
        }
    }

    fn distance_to_all_galaxies(&self) -> usize {
        self.distances.iter().map(|(distance, _)| distance).sum()
    }

    fn to_closest_pair(mut self) -> Option<GalacticPair> {
        self.distances
            .pop_front()
            .map(|(_, closest)| GalacticPair(closest, self.from))
    }
}

impl Eq for GalacticDistances {}

impl PartialEq<Self> for GalacticDistances {
    fn eq(&self, other: &Self) -> bool {
        self.distances
            .iter()
            .zip(&other.distances)
            .all(|(s, o)| s.0 == o.0)
    }
}

impl PartialOrd<Self> for GalacticDistances {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GalacticDistances {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distances
            .iter()
            .zip(&other.distances)
            .find_map(|(s, o)| {
                if s.0 == o.0 {
                    None
                } else {
                    if s.0 > o.0 {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Less)
                    }
                }
            })
            .unwrap_or(Ordering::Equal)
    }
}

#[derive(Debug, Copy, Clone)]
struct GalacticPair(GalaxyLocation, GalaxyLocation);

impl GalacticPair {
    fn includes(&self, location: &GalaxyLocation) -> bool {
        &self.0 == location || &self.1 == location
    }

    fn get_distance(&self) -> usize {
        self.0.distance_to(&self.1)
    }
}

fn parse_image(input: &str) -> IResult<&str, Image> {
    map(
        separated_list1(
            newline,
            many1(alt((value(true, char('#')), value(false, char('.'))))),
        ),
        |raw| raw.into(),
    )(input)
}

fn get_image_from_input(input: &str) -> Image {
    parse_image(input).expect("Image could not be parsed").1
}

// pub fn part1(input: &str) -> String {
//     let mut image = get_image_from_input(input);
//     image.expand();
//
//     let galaxies = image.get_galaxies();
//     let mut distances: VecDeque<_> = galaxies
//         .iter()
//         .map(|galaxy| galaxy.distances_to(&galaxies))
//         .sorted()
//         .collect();
//     let mut found_pairs: Vec<GalacticPair> = Vec::new();
//
//     while let Some(distance) = distances.pop_front() {
//         if let Some(pair) = distance.to_closest_pair() {
//             found_pairs.push(pair);
//             distances = distances
//                 .into_iter()
//                 .filter_map(move |d| d.remove_pair(&pair))
//                 .sorted()
//                 .collect();
//         }
//     }
//
//     found_pairs
//         .iter()
//         .map(|pair| pair.get_distance())
//         .sum::<usize>()
//         .to_string()
// }

pub fn part1(input: &str) -> String {
    let mut image = get_image_from_input(input);
    image.expand();

    let galaxies = image.get_galaxies();
    galaxies
        .iter()
        .map(|galaxy| galaxy.distances_to(&galaxies))
        .map(|distances| distances.distance_to_all_galaxies())
        .sum::<usize>()
        .div(2) // Hacks
        .to_string()
}

pub fn part2(_input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    mod parts {
        use super::*;

        fn get_test_image() -> Image {
            get_image_from_input(
                "...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....",
            )
        }

        #[test]
        fn test_parse_image() {
            let image = get_test_image();
            assert_eq!(image.len(), 10);
            assert!(image.iter().all(|row| row.len() == 10));
            assert_eq!(
                image
                    .iter()
                    .flatten()
                    .filter_map(|g| *g)
                    .collect::<Vec<_>>(),
                vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
            )
        }

        #[test]
        fn test_image_expand() {
            let expected_expansion = "....#........
.........#...
#............
.............
.............
........#....
.#...........
............#
.............
.............
.........#...
#....#.......";
            let mut image = get_test_image();
            image.expand();
            assert_eq!(image.to_string(), expected_expansion);
        }

        #[test]
        fn test_get_galaxies() {
            let galaxies = get_test_image().get_galaxies();
            assert_eq!(galaxies.len(), 9)
        }

        #[test]
        fn test_distance_to() {
            let g1 = GalaxyLocation::new(1, 6, 12);
            let g2 = GalaxyLocation::new(1, 4, 8);
            assert_eq!(g1.distance_to(&g2), 6);
        }

        #[test]
        fn test_distances_to() {
            let image = get_test_image();
            let galaxies = image.get_galaxies();
            assert_eq!(galaxies.len(), 9);
            let distances = galaxies[0].distances_to(&galaxies);
            assert_eq!(distances.distances.len(), 8);
        }
    }

    #[test]
    fn test_part1() {
        let input = "...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";
        assert_eq!(part1(input), "374")
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "")
    }
}
