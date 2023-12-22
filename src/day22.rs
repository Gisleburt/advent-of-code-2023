use std::cmp::{max, min};

use derive_more::{Deref, DerefMut, From};
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::newline;
use nom::combinator::{into, map};
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair, tuple};
use nom::IResult;

#[derive(Debug, Copy, Clone, PartialEq, From)]
struct Coordinate {
    x: u64,
    y: u64,
    z: u64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Area {
    top: u64,
    bottom: u64,
    left: u64,
    right: u64,
}

impl Area {
    fn point_inside(&self, x: u64, y: u64) -> bool {
        self.left <= x && self.right >= x && self.top <= y && self.bottom >= y
    }

    fn overlaps(&self, other: &Area) -> bool {
        self.point_inside(other.left, other.top)
            || self.point_inside(other.right, other.top)
            || self.point_inside(other.left, other.bottom)
            || self.point_inside(other.right, other.bottom)
            || other.point_inside(self.left, self.top)
            || other.point_inside(self.right, self.top)
            || other.point_inside(self.left, self.bottom)
            || other.point_inside(self.right, self.bottom)
    }
}

impl From<Brick> for Area {
    fn from(brick: Brick) -> Self {
        Self {
            top: min(brick.0.y, brick.1.y),
            bottom: max(brick.0.y, brick.1.y),
            left: min(brick.0.x, brick.1.x),
            right: max(brick.0.x, brick.1.x),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, From)]
struct Brick(Coordinate, Coordinate);

impl Brick {
    fn lowest_point(&self) -> u64 {
        min(self.0.z, self.1.z)
    }

    fn highest_point(&self) -> u64 {
        max(self.0.z, self.1.z)
    }

    fn move_down_to(&mut self, lowest_point: u64) {
        let distance = self.lowest_point() - lowest_point;
        self.0.z -= distance;
        self.1.z -= distance;
    }

    fn footprint_overlaps(&self, other: &Brick) -> bool {
        Area::from(*self).overlaps(&Area::from(*other))
    }

    fn is_resting_on(&self, other: &Brick) -> bool {
        self.lowest_point() == other.highest_point() + 1 && self.footprint_overlaps(other)
    }

    fn held_by(&self, bricks: &[Brick]) -> Vec<Brick> {
        bricks
            .iter()
            .filter(|other| self.is_resting_on(other))
            .copied()
            .collect()
    }

    fn held_by_only(&self, bricks: &[Brick], other: &Brick) -> bool {
        let held_by = self.held_by(bricks);
        held_by.contains(other) && held_by.len() == 1
    }
}

#[derive(Debug, Clone, PartialEq, From, Deref, DerefMut)]
struct Bricks(Vec<Brick>);

impl Bricks {
    fn sort(&mut self) {
        self.sort_by_key(|brick| brick.lowest_point())
    }

    fn collapse(&mut self) {
        self.sort();
        for i in 0..self.len() {
            let mut current_brick = *self.get(i).unwrap();
            let mut bricks_below = self[0..i].iter().rev();
            let new_z = bricks_below
                .filter_map(|other| {
                    current_brick
                        .footprint_overlaps(other)
                        .then_some(other.highest_point() + 1)
                })
                .max()
                .unwrap_or(1);
            self.get_mut(i).map(|brick| brick.move_down_to(new_z));
        }
    }

    fn find_potentially_removable(&self) -> Vec<Brick> {
        let mut removable = vec![];
        for i in 0..self.len() {
            let current_brick = self.get(i).unwrap();
            let is_holding_brick = self[(i + 1)..]
                .iter()
                .any(|other| other.held_by_only(&self, current_brick));
            if !is_holding_brick {
                removable.push(*current_brick)
            }
        }
        removable
    }
}

fn parse_coordinate(input: &str) -> IResult<&str, Coordinate> {
    map(
        tuple((
            complete::u64,
            preceded(tag(","), complete::u64),
            preceded(tag(","), complete::u64),
        )),
        |(x, y, z)| Coordinate { x, y, z },
    )(input)
}

fn parse_brick(input: &str) -> IResult<&str, Brick> {
    into(separated_pair(parse_coordinate, tag("~"), parse_coordinate))(input)
}

fn parse_bricks(input: &str) -> IResult<&str, Bricks> {
    into(separated_list1(newline, parse_brick))(input)
}

pub fn part1(input: &str) -> String {
    let mut bricks = parse_bricks(input).unwrap().1;
    bricks.collapse();
    bricks.find_potentially_removable().len().to_string()
}

pub fn part2(_input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    mod area {
        use super::*;

        #[test]
        fn test_point_inside() {
            let area = Area {
                top: 1,
                bottom: 2,
                left: 1,
                right: 2,
            };

            assert!(area.point_inside(1, 1));
            assert!(area.point_inside(1, 2));
            assert!(area.point_inside(2, 1));
            assert!(area.point_inside(2, 2));
            assert!(!area.point_inside(0, 2));
        }

        #[test]
        fn test_area_overlaps() {
            let area1 = Area {
                top: 1,
                bottom: 3,
                left: 1,
                right: 3,
            };
            // Perfect overlap
            assert!(area1.overlaps(&Area {
                top: 1,
                bottom: 3,
                left: 1,
                right: 3,
            }));
            // Inside
            assert!(area1.overlaps(&Area {
                top: 2,
                bottom: 2,
                left: 2,
                right: 2,
            }));
            // Outside
            assert!(area1.overlaps(&Area {
                top: 0,
                bottom: 4,
                left: 0,
                right: 4,
            }));
            // TL
            assert!(area1.overlaps(&Area {
                top: 2,
                bottom: 4,
                left: 2,
                right: 4,
            }));
            // TR
            assert!(area1.overlaps(&Area {
                top: 2,
                bottom: 4,
                left: 0,
                right: 2,
            }));
            // BL
            assert!(area1.overlaps(&Area {
                top: 0,
                bottom: 2,
                left: 2,
                right: 4,
            }));
            // BR
            assert!(area1.overlaps(&Area {
                top: 0,
                bottom: 2,
                left: 0,
                right: 2,
            }));
        }

        #[test]
        fn test_from_brick() {
            let brick = Brick(
                Coordinate { x: 2, y: 2, z: 3 },
                Coordinate { x: 1, y: 3, z: 4 },
            );
            assert_eq!(
                Area::from(brick),
                Area {
                    top: 2,
                    bottom: 3,
                    left: 1,
                    right: 2,
                }
            )
        }
    }

    mod brick {
        use super::*;

        #[test]
        fn test_lowest_point() {
            let brick = Brick(
                Coordinate { x: 2, y: 2, z: 3 },
                Coordinate { x: 1, y: 3, z: 4 },
            );
            assert_eq!(brick.lowest_point(), 3);
        }

        #[test]
        fn test_highest_point() {
            let brick = Brick(
                Coordinate { x: 2, y: 2, z: 3 },
                Coordinate { x: 1, y: 3, z: 4 },
            );
            assert_eq!(brick.highest_point(), 4);
        }

        #[test]
        fn test_is_resting_on() {
            let test_brick = Brick(
                Coordinate { x: 1, y: 1, z: 3 },
                Coordinate { x: 2, y: 2, z: 3 },
            );
            let resting_on_1 = Brick(
                Coordinate { x: 0, y: 0, z: 2 },
                Coordinate { x: 1, y: 1, z: 2 },
            );
            let resting_on_2 = Brick(
                Coordinate { x: 2, y: 2, z: 2 },
                Coordinate { x: 3, y: 3, z: 2 },
            );
            let below = Brick(
                Coordinate { x: 0, y: 0, z: 1 },
                Coordinate { x: 1, y: 1, z: 1 },
            );
            let aside = Brick(
                Coordinate { x: 4, y: 4, z: 2 },
                Coordinate { x: 4, y: 4, z: 2 },
            );
            assert!(test_brick.is_resting_on(&resting_on_1));
            assert!(test_brick.is_resting_on(&resting_on_2));
            assert!(!test_brick.is_resting_on(&below));
            assert!(!test_brick.is_resting_on(&aside));
        }

        #[test]
        fn test_held_by() {
            let test_brick = Brick(
                Coordinate { x: 1, y: 1, z: 3 },
                Coordinate { x: 2, y: 2, z: 3 },
            );
            let resting_on_1 = Brick(
                Coordinate { x: 0, y: 0, z: 2 },
                Coordinate { x: 1, y: 1, z: 2 },
            );
            let resting_on_2 = Brick(
                Coordinate { x: 2, y: 2, z: 2 },
                Coordinate { x: 3, y: 3, z: 2 },
            );
            let below = Brick(
                Coordinate { x: 0, y: 0, z: 1 },
                Coordinate { x: 1, y: 1, z: 1 },
            );
            let aside = Brick(
                Coordinate { x: 4, y: 4, z: 2 },
                Coordinate { x: 4, y: 4, z: 2 },
            );
            let bricks = [test_brick, resting_on_1, resting_on_2, below, aside];

            assert_eq!(
                test_brick.held_by(&bricks),
                vec![resting_on_1, resting_on_2]
            );
        }

        #[test]
        fn test_held_by_only() {
            let test_brick = Brick(
                Coordinate { x: 1, y: 1, z: 3 },
                Coordinate { x: 2, y: 2, z: 3 },
            );
            let resting_on_1 = Brick(
                Coordinate { x: 0, y: 0, z: 2 },
                Coordinate { x: 1, y: 1, z: 2 },
            );
            let resting_on_2 = Brick(
                Coordinate { x: 2, y: 2, z: 2 },
                Coordinate { x: 3, y: 3, z: 2 },
            );
            let below = Brick(
                Coordinate { x: 0, y: 0, z: 1 },
                Coordinate { x: 1, y: 1, z: 1 },
            );
            let aside = Brick(
                Coordinate { x: 4, y: 4, z: 2 },
                Coordinate { x: 4, y: 4, z: 2 },
            );
            let two_resting = [test_brick, resting_on_1, resting_on_2, below, aside];
            let one_resting = [test_brick, resting_on_1, below, aside];

            assert!(!test_brick.held_by_only(&two_resting, &resting_on_1));
            assert!(test_brick.held_by_only(&one_resting, &resting_on_1));
        }
    }

    mod bricks {
        use super::*;

        #[test]
        fn test_sort() {
            let mut bricks = Bricks(vec![
                Brick(
                    Coordinate { x: 1, y: 1, z: 3 },
                    Coordinate { x: 2, y: 2, z: 10 },
                ),
                Brick(
                    Coordinate { x: 1, y: 1, z: 1 },
                    Coordinate { x: 2, y: 2, z: 4 },
                ),
            ]);
            bricks.sort();
            assert_eq!(
                &bricks.0,
                &[
                    Brick(
                        Coordinate { x: 1, y: 1, z: 1 },
                        Coordinate { x: 2, y: 2, z: 4 },
                    ),
                    Brick(
                        Coordinate { x: 1, y: 1, z: 3 },
                        Coordinate { x: 2, y: 2, z: 10 },
                    ),
                ]
            )
        }

        #[test]
        fn test_collapse() {
            // ======
            // ||
            // ||  =
            // || ===
            // Brick we want to drop
            let perched_on_top = Brick(
                Coordinate { x: 0, y: 0, z: 20 },
                Coordinate { x: 0, y: 4, z: 20 },
            );
            // Low and flat
            let low_and_flat = Brick(
                Coordinate { x: 1, y: 1, z: 2 },
                Coordinate { x: 1, y: 4, z: 2 },
            );
            // Sits on low and flat
            let sits_on_low_and_flat = Brick(
                Coordinate { x: 1, y: 2, z: 4 },
                Coordinate { x: 1, y: 2, z: 4 },
            );
            // Is really tall but rests on ground
            let really_tall = Brick(
                Coordinate { x: 0, y: 0, z: 5 },
                Coordinate { x: 0, y: 0, z: 10 },
            );

            let mut bricks = Bricks(vec![
                perched_on_top,
                low_and_flat,
                sits_on_low_and_flat,
                really_tall,
            ]);
            bricks.collapse();
            assert_eq!(
                &bricks.0,
                &[
                    // Low and flat
                    Brick(
                        Coordinate { x: 1, y: 1, z: 1 },
                        Coordinate { x: 1, y: 4, z: 1 },
                    ),
                    // Sits on low and flat
                    Brick(
                        Coordinate { x: 1, y: 2, z: 2 },
                        Coordinate { x: 1, y: 2, z: 2 },
                    ),
                    // Is really tall but rests on ground
                    Brick(
                        Coordinate { x: 0, y: 0, z: 1 },
                        Coordinate { x: 0, y: 0, z: 6 },
                    ),
                    // Brick we want to drop
                    Brick(
                        Coordinate { x: 0, y: 0, z: 7 },
                        Coordinate { x: 0, y: 4, z: 7 },
                    ),
                ]
            )
        }

        #[test]
        fn test_find_potentially_removable() {
            // ======
            // ||
            // ||  =
            // || ===
            // Brick we want to drop
            let perched_on_top = Brick(
                Coordinate { x: 0, y: 0, z: 20 },
                Coordinate { x: 0, y: 4, z: 20 },
            );
            // Low and flat
            let low_and_flat = Brick(
                Coordinate { x: 1, y: 1, z: 2 },
                Coordinate { x: 1, y: 4, z: 2 },
            );
            // Sits on low and flat
            let sits_on_low_and_flat = Brick(
                Coordinate { x: 1, y: 2, z: 4 },
                Coordinate { x: 1, y: 2, z: 4 },
            );
            // Is really tall but rests on ground
            let really_tall = Brick(
                Coordinate { x: 0, y: 0, z: 5 },
                Coordinate { x: 0, y: 0, z: 10 },
            );

            let mut bricks = Bricks(vec![
                perched_on_top,
                low_and_flat,
                sits_on_low_and_flat,
                really_tall,
            ]);
            bricks.collapse();
            // ======  <-
            // ||
            // ||  =   <-
            // || ===
            assert_eq!(bricks.find_potentially_removable().len(), 2);
        }
    }

    #[test]
    fn test_part1() {
        let input = "1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9";
        assert_eq!(part1(input), "5");
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "");
    }
}
