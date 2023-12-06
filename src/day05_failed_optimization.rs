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

    #[cfg(test)]
    fn new_raw(start: Number, end: Number, modifier: Number) -> Self {
        Map {
            range: start..end,
            modifier,
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

    fn overlaps(&self, other: &Map) -> bool {
        self.range.end > other.range.start && self.range.start < other.range.end
    }

    fn starts_before(&self, other: &Map) -> bool {
        self.range.start < other.range.start
    }

    fn ends_after(&self, other: &Map) -> bool {
        self.range.end > other.range.end
    }

    fn get_before_inside_after(&self, other: &Map) -> (Option<Map>, Map, Option<Map>) {
        assert!(self.overlaps(other));
        let modifier = self.modifier;
        let before = match self.starts_before(other) {
            true => Some(Map {
                range: self.range.start..self.range.end.min(other.range.start),
                modifier,
            }),
            false => None,
        };
        let inside = Map {
            range: self.range.start.max(other.range.start)..self.range.end.min(other.range.end),
            modifier,
        };
        let after = match self.ends_after(other) {
            true => Some(Map {
                range: self.range.start.max(other.range.end)..self.range.end,
                modifier,
            }),
            false => None,
        };
        (before, inside, after)
    }
}

trait OptionalPush<T> {
    fn optional_push(&mut self, opt: Option<T>);
}

impl<T> OptionalPush<T> for Vec<T> {
    fn optional_push(&mut self, opt: Option<T>) {
        if let Some(value) = opt {
            self.push(value)
        }
    }
}

#[derive(Debug)]
struct CombinedMap {
    maps: Option<Vec<Map>>,
}

impl CombinedMap {
    fn new() -> Self {
        CombinedMap { maps: None }
    }

    fn add_map(&mut self, new_map: Map) {
        let Some(maps) = self.maps.take() else {
            self.maps = Some(vec![new_map]);
            return;
        };

        let (mut unaffected_maps, maps_to_process): (Vec<_>, Vec<_>) = maps
            .into_iter()
            .partition(|existing_map| !&new_map.overlaps(existing_map));

        let mut map_to_add = Some(new_map);
        for old_map in maps_to_process.into_iter() {
            let new_map = map_to_add.take().unwrap();
            let (new_before, mut new_inside, new_after) = new_map.get_before_inside_after(&old_map);
            let (old_before, old_inside, old_after) = old_map.get_before_inside_after(&new_map);

            unaffected_maps.optional_push(new_before);
            unaffected_maps.optional_push(old_before);

            // The middle needs modifying
            new_inside.modifier += old_inside.modifier;
            unaffected_maps.push(new_inside);

            // If there's a leftover old map its not being overlapped so check it straight in
            unaffected_maps.optional_push(old_after);
            // Leftover new map should replace the "map to add" though
            map_to_add = new_after;
        }
        unaffected_maps.optional_push(map_to_add);

        unaffected_maps.sort_by_key(|map| map.range.start);

        self.maps = Some(unaffected_maps);
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
    // dbg!(&combined);
    combined.optimise();
    seeds
        .into_iter()
        // .inspect(|s| {
        //     dbg!(s);
        // })
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
    fn test_combined_map_add_map() {
        let mut combined = CombinedMap::new();

        let map1 = Map::new_raw(5, 10, 1);
        combined.add_map(map1);
        assert_eq!(combined.maps, Some(vec![Map::new_raw(5, 10, 1)]));

        let map2 = Map::new_raw(8, 12, 2);
        combined.add_map(map2);
        assert_eq!(
            combined.maps,
            Some(vec![
                Map::new_raw(5, 8, 1),
                Map::new_raw(8, 10, 3),
                Map::new_raw(10, 12, 2),
            ])
        );

        let map3 = Map::new_raw(2, 4, 3);
        combined.add_map(map3);
        assert_eq!(
            combined.maps,
            Some(vec![
                Map::new_raw(2, 4, 3),
                Map::new_raw(5, 8, 1),
                Map::new_raw(8, 10, 3),
                Map::new_raw(10, 12, 2),
            ])
        );

        let map4 = Map::new_raw(3, 6, 5);
        combined.add_map(map4);
        assert_eq!(
            combined.maps,
            Some(vec![
                Map::new_raw(2, 3, 3),
                Map::new_raw(3, 4, 8),
                Map::new_raw(4, 5, 5),
                Map::new_raw(5, 6, 6),
                Map::new_raw(6, 8, 1),
                Map::new_raw(8, 10, 3),
                Map::new_raw(10, 12, 2),
            ])
        );
    }

    #[test]
    fn test_combined_map_add_map_no_overlap() {
        let mut combined = CombinedMap::new();

        let middle = Map::new_raw(5, 10, 1);
        combined.add_map(middle);
        assert_eq!(combined.maps, Some(vec![Map::new_raw(5, 10, 1)]));

        let first = Map::new_raw(1, 5, 2);
        combined.add_map(first.clone());
        assert_eq!(
            combined.maps,
            Some(vec![Map::new_raw(1, 5, 2), Map::new_raw(5, 10, 1)])
        );

        let last = Map::new_raw(10, 15, 3);
        combined.add_map(last.clone());
        assert_eq!(
            combined.maps,
            Some(vec![
                Map::new_raw(1, 5, 2),
                Map::new_raw(5, 10, 1),
                Map::new_raw(10, 15, 3),
            ])
        );
    }

    #[test]
    fn test_combined_map_add_map_simple_overlap() {
        let mut combined = CombinedMap::new();

        let middle = Map::new_raw(4, 11, 1);
        combined.add_map(middle);
        assert_eq!(combined.maps, Some(vec![Map::new_raw(4, 11, 1)]));

        let first = Map::new_raw(1, 6, 2);
        combined.add_map(first);
        assert_eq!(
            combined.maps,
            Some(vec![
                Map::new_raw(1, 4, 2),
                Map::new_raw(4, 6, 3),
                Map::new_raw(6, 11, 1)
            ])
        );

        let last = Map::new_raw(9, 15, 3);
        combined.add_map(last);
        assert_eq!(
            combined.maps,
            Some(vec![
                Map::new_raw(1, 4, 2),
                Map::new_raw(4, 6, 3),
                Map::new_raw(6, 9, 1),
                Map::new_raw(9, 11, 4),
                Map::new_raw(11, 15, 3),
            ])
        );
    }

    #[test]
    fn test_combined_map_add_map_middle_overlap() {
        let mut combined = CombinedMap::new();

        let first = Map::new_raw(5, 10, 1);
        combined.add_map(first);
        assert_eq!(combined.maps, Some(vec![Map::new_raw(5, 10, 1)]));

        let second = Map::new_raw(5, 10, 2);
        combined.add_map(second);
        assert_eq!(combined.maps, Some(vec![Map::new_raw(5, 10, 3)]));

        let third = Map::new_raw(1, 15, 3);
        combined.add_map(third);
        assert_eq!(
            combined.maps,
            Some(vec![
                Map::new_raw(1, 5, 3),
                Map::new_raw(5, 10, 6),
                Map::new_raw(10, 15, 3),
            ])
        );

        let fourth = Map::new_raw(3, 13, 4);
        combined.add_map(fourth);
        assert_eq!(
            combined.maps,
            Some(vec![
                Map::new_raw(1, 3, 3),
                Map::new_raw(3, 5, 7),
                Map::new_raw(5, 10, 10),
                Map::new_raw(10, 13, 7),
                Map::new_raw(13, 15, 3),
            ])
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
