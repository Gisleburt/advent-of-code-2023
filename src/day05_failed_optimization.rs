use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{digit1, space1};
use nom::multi::{many1, separated_list1};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use std::ops::Range;

// Just making one place for all number types I can change later
type Number = i64;

#[derive(Debug, Default, PartialEq, Clone)]
struct Map {
    range: Range<Number>,
    modifier: Number,
}

impl Map {
    fn new(adjustment: Number, range_start: Number, range_size: Number) -> Self {
        Map {
            range: range_start..(range_start + range_size),
            modifier: adjustment - range_start,
        }
    }

    fn contains(&self, number: Number) -> bool {
        self.range.start <= number && self.range.end > number
    }

    fn could_append(&self, other: &Map) -> bool {
        self.range.end == other.range.start && self.modifier == other.modifier
    }

    fn attempt_append(self, other: Map) -> Vec<Map> {
        if self.could_append(&other) {
            vec![Map {
                range: other.range.start..self.range.end,
                modifier: self.modifier,
            }]
        } else {
            vec![self, other]
        }
    }

    fn is_contained_within(&self, other: &Map) -> bool {
        self.range.start >= other.range.start && self.range.end <= other.range.end
    }

    fn is_contained_within_edges(&self, other: &Map) -> bool {
        self.range.start > other.range.start && self.range.end < other.range.end
    }

    fn overlaps(&self, other: &Map) -> bool {
        self.range.end > other.range.start && self.range.start < other.range.end
    }

    fn starts_before(&self, other: &Map) -> bool {
        self.range.start < other.range.start
    }

    fn overlaps_and_is_first(&self, other: &Map) -> bool {
        self.overlaps(other) && self.starts_before(other)
    }

    fn overlaps_and_is_second(&self, other: &Map) -> bool {
        self.overlaps(other) && !self.starts_before(other)
    }

    /// R1 is container within R2
    /// ```text
    ///    [r1]
    /// + [ r2 ]
    /// = [ r2 ]
    /// ```
    /// _and_
    /// ```text
    ///   [ r1 ]
    /// + [ r2 ]
    /// = [ r2 ]
    /// ```
    /// R2 is fully inside the edges of R1
    /// ```text
    ///   [    r1    ]
    /// +     [r2]
    /// = [r1][r2][r1]
    /// ```
    /// R1 overlaps R2 and is first
    /// ```text
    ///   [ r1 ]
    /// +     [ r2 ]
    /// = [r1][ r2 ]
    /// ```
    /// _and_
    /// ```text
    ///   [   r1   ]
    /// +     [ r2 ]
    /// = [r1][ r2 ]
    /// ```
    /// R1 overlaps R2 and is second
    /// ```text
    ///       [ r1 ]
    /// + [ r2 ]
    /// = [ r2 ][r1]
    /// ```
    /// _and_
    /// ```text
    ///   [     r1 ]
    /// + [ r2 ]
    /// = [ r2 ][r1]
    /// ```
    /// I think ordering might be important here
    fn combine(self, other: Map) -> Vec<Map> {
        if self.is_contained_within(&other) {
            vec![other]
        } else if other.is_contained_within_edges(&self) {
            vec![
                Map {
                    range: self.range.start..other.range.start,
                    modifier: self.modifier,
                },
                Map {
                    range: other.range.start..other.range.end,
                    modifier: other.modifier,
                },
                Map {
                    range: other.range.end..self.range.end,
                    modifier: self.modifier,
                },
            ]
        } else if self.overlaps_and_is_first(&other) {
            vec![
                Map {
                    range: self.range.start..other.range.start,
                    modifier: self.modifier,
                },
                Map {
                    range: other.range.start..other.range.end,
                    modifier: other.modifier,
                },
            ]
        } else if self.overlaps_and_is_second(&other) {
            vec![
                Map {
                    range: other.range.start..other.range.end,
                    modifier: other.modifier,
                },
                Map {
                    range: other.range.end..self.range.end,
                    modifier: self.modifier,
                },
            ]
        } else if self.starts_before(&other) {
            vec![self, other]
        } else {
            vec![other, self]
        }
    }
}

struct CombinedMap {
    maps: Option<Vec<Map>>,
}

impl CombinedMap {
    fn new() -> Self {
        CombinedMap {
            maps: Some(Vec::with_capacity(0)),
        }
    }

    fn add_map(&mut self, new_map: Map) {
        let maps = self.maps.take().unwrap(); // Leaves None behind, always put something back

        let new_map_ref = &new_map;
        let (mut unaffected_maps, mut maps_to_process): (Vec<_>, Vec<_>) = maps
            .into_iter()
            .partition(|existing_map| !new_map_ref.overlaps(existing_map));

        maps_to_process.reverse();
        while let Some(map_to_handle) = maps_to_process.pop() {
            if new_map.overlaps(&map_to_handle) {
                maps_to_process.extend(map_to_handle.combine(new_map.clone()));
            } else {
                unaffected_maps.push(map_to_handle)
            }
        }

        unaffected_maps.sort_by_key(|map| map.range.start);

        self.maps = Some(unaffected_maps)
    }

    fn optimise(&mut self) {
        let max_size = self.maps.as_ref().unwrap().len();
        self.maps = Some(self.maps.take().unwrap().into_iter().fold(
            Vec::with_capacity(max_size),
            |mut acc, cur| {
                if let Some(last) = acc.pop() {
                    acc.extend(last.attempt_append(cur))
                } else {
                    acc.push(cur);
                }
                acc
            },
        ));
    }

    fn map_position(&self, pos: Number) -> Number {
        self.maps
            .as_ref()
            .unwrap()
            .iter()
            .find(|map| map.contains(pos))
            .map(|map| pos + map.modifier)
            .unwrap_or(pos)
    }
}

fn parse_seeds(input: &str) -> IResult<&str, Vec<Number>> {
    preceded(tag("seeds: "), separated_list1(tag(" "), digit1))(input)
        .map(|(r, l)| (r, l.into_iter().map(|num| num.parse().unwrap()).collect()))
}

fn parse_map(input: &str) -> IResult<&str, Map> {
    let (remainder, (adjustment_amount, _, source, _, range)) =
        tuple((digit1, space1, digit1, space1, digit1))(input)?;
    Ok((
        remainder,
        Map::new(
            adjustment_amount.parse().unwrap(),
            source.parse().unwrap(),
            range.parse().unwrap(),
        ),
    ))
}

fn parse_maps(input: &str) -> IResult<&str, Vec<Map>> {
    many1(preceded(take_till(char::is_numeric), parse_map))(input)
}

fn parse_input(input: &str) -> (Vec<Number>, Vec<Map>) {
    tuple((parse_seeds, parse_maps))(input).unwrap().1
}

pub fn part1(input: &str) -> String {
    let (seeds, maps) = parse_input(input);
    let mut combined = maps
        .into_iter()
        .inspect(|m| {
            dbg!(m);
        })
        .fold(CombinedMap::new(), |mut combined, map| {
            combined.add_map(map);
            combined
        });
    combined.optimise();
    seeds
        .into_iter()
        .map(|seed| combined.map_position(seed))
        .min()
        .unwrap()
        .to_string()
}

pub fn part2(_input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_map_combine() {
        //    [r1]
        // + [ r2 ]
        // = [ r2 ]
        let map1 = Map {
            range: 2..3,
            modifier: 1,
        };
        let map2 = Map {
            range: 1..5,
            modifier: 2,
        };
        assert_eq!(
            map1.combine(map2),
            vec![Map {
                range: 1..5,
                modifier: 2
            }]
        );

        //   [ r1 ]
        // + [ r2 ]
        // = [ r2 ]
        let map1 = Map {
            range: 1..5,
            modifier: 1,
        };
        let map2 = Map {
            range: 1..5,
            modifier: 2,
        };
        assert_eq!(
            map1.combine(map2),
            vec![Map {
                range: 1..5,
                modifier: 2
            }]
        );

        //   [    r1    ]
        // +     [r2]
        // = [r1][r2][r1]
        let map1 = Map {
            range: 1..12,
            modifier: 1,
        };
        let map2 = Map {
            range: 5..9,
            modifier: 2,
        };
        assert_eq!(
            map1.combine(map2),
            vec![
                Map {
                    range: 1..5,
                    modifier: 1
                },
                Map {
                    range: 5..9,
                    modifier: 2
                },
                Map {
                    range: 9..12,
                    modifier: 1
                },
            ]
        );

        //   [ r1 ]
        // +     [ r2 ]
        // = [r1][ r2 ]
        let map1 = Map {
            range: 1..5,
            modifier: 1,
        };
        let map2 = Map {
            range: 4..9,
            modifier: 2,
        };
        assert_eq!(
            map1.combine(map2),
            vec![
                Map {
                    range: 1..4,
                    modifier: 1
                },
                Map {
                    range: 4..9,
                    modifier: 2
                }
            ]
        );

        //   [   r1   ]
        // +     [ r2 ]
        // = [r1][ r2 ]
        let map1 = Map {
            range: 1..9,
            modifier: 1,
        };
        let map2 = Map {
            range: 4..9,
            modifier: 2,
        };
        assert_eq!(
            map1.combine(map2),
            vec![
                Map {
                    range: 1..4,
                    modifier: 1
                },
                Map {
                    range: 4..9,
                    modifier: 2
                }
            ]
        );

        //       [ r1 ]
        // + [ r2 ]
        // = [ r2 ][r1]
        let map1 = Map {
            range: 4..9,
            modifier: 1,
        };
        let map2 = Map {
            range: 1..5,
            modifier: 2,
        };
        assert_eq!(
            map1.combine(map2),
            vec![
                Map {
                    range: 1..5,
                    modifier: 2
                },
                Map {
                    range: 5..9,
                    modifier: 1
                }
            ]
        );

        //   [   r1   ]
        // + [ r2 ]
        // = [ r2 ][r1]
        let map1 = Map {
            range: 1..9,
            modifier: 1,
        };
        let map2 = Map {
            range: 1..5,
            modifier: 2,
        };
        assert_eq!(
            map1.combine(map2),
            vec![
                Map {
                    range: 1..5,
                    modifier: 2
                },
                Map {
                    range: 5..9,
                    modifier: 1
                }
            ]
        );

        //   [r1]
        // +      [r2]
        // = [r1] [r2]
        let map1 = Map {
            range: 1..5,
            modifier: 1,
        };
        let map2 = Map {
            range: 10..15,
            modifier: 2,
        };
        assert_eq!(
            map1.combine(map2),
            vec![
                Map {
                    range: 1..5,
                    modifier: 1
                },
                Map {
                    range: 10..15,
                    modifier: 2
                }
            ]
        );

        //        [r1]
        // + [r2]
        // = [r2] [r1]
        let map1 = Map {
            range: 10..15,
            modifier: 1,
        };
        let map2 = Map {
            range: 1..5,
            modifier: 2,
        };
        assert_eq!(
            map1.combine(map2),
            vec![
                Map {
                    range: 1..5,
                    modifier: 2
                },
                Map {
                    range: 10..15,
                    modifier: 1
                }
            ]
        );
    }

    #[test]
    fn test_part1() {
        let input = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37";
        assert_eq!(part1(input), "35")
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37";
        assert_eq!(part2(input), "46")
    }
}
