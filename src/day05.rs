use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, newline, space1};
use nom::combinator::value;
use nom::multi::separated_list1;
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;
use rayon::prelude::*;
use std::ops::Range;

// Just making one place for all number types I can change later
type Number = u64;

#[derive(Debug, PartialEq, Copy, Clone)]
enum MapType {
    SeedToSoil,
    SoilToFertilizer,
    FertilizerToWater,
    WaterToLight,
    LightToTemperature,
    TemperatureToHumidity,
    HumidityToLocation,
}

#[derive(Debug, Default, PartialEq, Clone)]
struct RangeMap {
    source: Range<Number>,
    destination: Number,
}

impl RangeMap {
    fn new(source_start: Number, destination_start: Number, range: Number) -> Self {
        RangeMap {
            source: source_start..(source_start + range),
            destination: destination_start,
        }
    }

    fn contains(&self, number: Number) -> bool {
        self.source.contains(&number)
    }

    fn apply(&self, number: Number) -> Number {
        if self.contains(number) {
            number - self.source.start + self.destination
        } else {
            number
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct SeedMap {
    map_type: MapType,
    ranges: Vec<RangeMap>,
}

impl SeedMap {
    fn apply(&self, number: Number) -> Number {
        if let Some(range) = self.ranges.iter().find(|r| r.contains(number)) {
            range.apply(number)
        } else {
            number
        }
    }
}

#[derive(Debug, PartialEq)]
struct Almanac {
    seed_to_soil: SeedMap,
    soil_to_fertilizer: SeedMap,
    fertilizer_to_water: SeedMap,
    water_to_light: SeedMap,
    light_to_temperature: SeedMap,
    temperature_to_humidity: SeedMap,
    humidity_to_location: SeedMap,
}

#[derive(Debug, PartialEq)]
struct SeedsV(Vec<Number>);

#[derive(Debug, PartialEq)]
struct SeedsR(Range<Number>);

type NumberIterator = dyn Iterator<Item = Number>;

trait Seeds {
    fn seed_iter(&self) -> Box<NumberIterator>;

    fn nearest_seed_according_to_almanac<'a>(&'a self, almanac: &'a Almanac) -> Number {
        self.seed_iter()
            .map(|seed| almanac.seed_to_soil.apply(seed))
            .map(|seed| almanac.soil_to_fertilizer.apply(seed))
            .map(|seed| almanac.fertilizer_to_water.apply(seed))
            .map(|seed| almanac.water_to_light.apply(seed))
            .map(|seed| almanac.light_to_temperature.apply(seed))
            .map(|seed| almanac.temperature_to_humidity.apply(seed))
            .map(|seed| almanac.humidity_to_location.apply(seed))
            .min()
            .unwrap()
    }
}

impl Seeds for SeedsV {
    fn seed_iter(&self) -> Box<NumberIterator> {
        Box::new(self.0.clone().into_iter())
    }
}

impl Seeds for SeedsR {
    fn seed_iter(&self) -> Box<NumberIterator> {
        Box::new(self.0.clone())
    }
}

impl From<SeedsV> for Vec<SeedsR> {
    fn from(seeds: SeedsV) -> Self {
        seeds
            .0
            .into_iter()
            .chunks(2)
            .into_iter()
            .map(|mut i| {
                let start = i.next().unwrap();
                let size = i.next().unwrap();
                SeedsR(start..(start + size))
            })
            .collect()
    }
}

fn parse_map_type(input: &str) -> IResult<&str, MapType> {
    alt((
        value(MapType::SeedToSoil, tag("seed-to-soil")),
        value(MapType::SoilToFertilizer, tag("soil-to-fertilizer")),
        value(MapType::FertilizerToWater, tag("fertilizer-to-water")),
        value(MapType::WaterToLight, tag("water-to-light")),
        value(MapType::LightToTemperature, tag("light-to-temperature")),
        value(
            MapType::TemperatureToHumidity,
            tag("temperature-to-humidity"),
        ),
        value(MapType::HumidityToLocation, tag("humidity-to-location")),
    ))(input)
}

fn parse_seeds(input: &str) -> IResult<&str, SeedsV> {
    let (remainder, seeds) =
        delimited(tag("seeds: "), separated_list1(space1, digit1), newline)(input)?;
    Ok((
        remainder,
        SeedsV(seeds.into_iter().map(|s| s.parse().unwrap()).collect()),
    ))
}

fn parse_range_map(input: &str) -> IResult<&str, RangeMap> {
    let (remainder, (dest, _, source, _, range)) =
        tuple((digit1, space1, digit1, space1, digit1))(input)?;
    Ok((
        remainder,
        RangeMap::new(
            source.parse().unwrap(),
            dest.parse().unwrap(),
            range.parse().unwrap(),
        ),
    ))
}

fn parse_seed_map(input: &str) -> IResult<&str, SeedMap> {
    let (remainder, (map_type, ranges)) = tuple((
        terminated(parse_map_type, tuple((tag(" map:"), newline))),
        separated_list1(newline, parse_range_map),
    ))(input)?;
    Ok((remainder, SeedMap { map_type, ranges }))
}

fn parse_almanac(input: &str) -> IResult<&str, (SeedsV, Almanac)> {
    let (remainder, (seeds, _, maps)) = tuple((
        parse_seeds,
        newline,
        separated_list1(tuple((newline, newline)), parse_seed_map),
    ))(input)?;

    let get_map = move |map_type: MapType| {
        maps.iter()
            .find(|m| m.map_type == map_type)
            .cloned()
            .expect("map not found")
    };

    Ok((
        remainder,
        (
            seeds,
            Almanac {
                seed_to_soil: get_map(MapType::SeedToSoil),
                soil_to_fertilizer: get_map(MapType::SoilToFertilizer),
                fertilizer_to_water: get_map(MapType::FertilizerToWater),
                water_to_light: get_map(MapType::WaterToLight),
                light_to_temperature: get_map(MapType::LightToTemperature),
                temperature_to_humidity: get_map(MapType::TemperatureToHumidity),
                humidity_to_location: get_map(MapType::HumidityToLocation),
            },
        ),
    ))
}

pub fn part1(input: &str) -> String {
    let (_, (seeds, almanac)) = parse_almanac(input).unwrap();
    seeds
        .nearest_seed_according_to_almanac(&almanac)
        .to_string()
}

pub fn part2(input: &str) -> String {
    let (_, (seeds, almanac)) = parse_almanac(input).unwrap();

    Vec::from(seeds)
        .into_par_iter()
        .map(|seeds| seeds.nearest_seed_according_to_almanac(&almanac))
        .min()
        .unwrap()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn test_parse_map_type() {
        assert_eq!(
            parse_map_type("seed-to-soil map:"),
            Ok((" map:", MapType::SeedToSoil))
        );
        assert_eq!(
            parse_map_type("soil-to-fertilizer map:"),
            Ok((" map:", MapType::SoilToFertilizer))
        );
        assert_eq!(
            parse_map_type("fertilizer-to-water map:"),
            Ok((" map:", MapType::FertilizerToWater))
        );
        assert_eq!(
            parse_map_type("water-to-light map:"),
            Ok((" map:", MapType::WaterToLight))
        );
        assert_eq!(
            parse_map_type("light-to-temperature map:"),
            Ok((" map:", MapType::LightToTemperature))
        );
        assert_eq!(
            parse_map_type("temperature-to-humidity map:"),
            Ok((" map:", MapType::TemperatureToHumidity))
        );
        assert_eq!(
            parse_map_type("humidity-to-location map:"),
            Ok((" map:", MapType::HumidityToLocation))
        );
    }

    #[test]
    fn test_parse_seeds() {
        let input = "seeds: 79 14 55 13
some other stuff";
        assert_eq!(
            parse_seeds(input),
            Ok(("some other stuff", SeedsV(vec![79u64, 14, 55, 13])))
        )
    }

    #[test]
    fn test_parse_range_map() {
        let input = "50 98 2
52 50 48";
        assert_eq!(
            parse_range_map(input),
            Ok(("\n52 50 48", RangeMap::new(98, 50, 2)))
        )
    }

    #[test]
    fn test_parse_seed_map() {
        let input = "light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:";
        assert_eq!(
            parse_seed_map(input),
            Ok((
                "\n\ntemperature-to-humidity map:",
                SeedMap {
                    map_type: MapType::LightToTemperature,
                    ranges: vec![
                        RangeMap::new(77, 45, 23),
                        RangeMap::new(45, 81, 19),
                        RangeMap::new(64, 68, 13),
                    ]
                }
            ))
        )
    }

    #[test]
    fn test_parse_almanac() {
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
        // Theoretically, this either works or panics
        let (remainder, _) = parse_almanac(input).unwrap();
        assert_eq!(remainder, "");
    }

    #[test]
    fn test_range() {
        let range = RangeMap::new(98, 50, 2);

        // In range
        assert!(range.contains(99));
        assert_eq!(range.apply(99), 51);

        // Out of range
        assert!(!range.contains(100));
        assert_eq!(range.apply(100), 100);
    }
}
