mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day05_failed_optimization;
mod day06;
mod day07;
mod day08;
mod day09;
mod day10;
mod day11;

use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
    #[structopt(short = "d", long = "day")]
    day: usize,
    #[structopt(short = "p", long = "part")]
    part: usize,
}

fn main() {
    let opt = Opt::from_args();
    let input_path = opt
        .input
        .unwrap_or_else(|| PathBuf::from(format!("inputs/d{:0>2}.txt", opt.day)));

    let input = read_to_string(input_path).expect("input not found");

    let start = Instant::now();
    let result = match (opt.day, opt.part) {
        (1, 1) => day01::part1(&input),
        (1, 2) => day01::part2(&input),
        (2, 1) => day02::part1(&input),
        (2, 2) => day02::part2(&input),
        (3, 1) => day03::part1(&input),
        (3, 2) => day03::part2(&input),
        (4, 1) => day04::part1(&input),
        (4, 2) => day04::part2(&input),
        (5, 1) => day05::part1(&input),
        (5, 2) => day05::part2(&input),
        (105, 1) => day05_failed_optimization::part1(&input),
        (105, 2) => day05_failed_optimization::part2(&input),
        (6, 1) => day06::part1(&input),
        (6, 2) => day06::part2(&input),
        (7, 1) => day07::part1(&input),
        (7, 2) => day07::part2(&input),
        (8, 1) => day08::part1(&input),
        (8, 2) => day08::part2(&input),
        (9, 1) => day09::part1(&input),
        (9, 2) => day09::part2(&input),
        (10, 1) => day10::part1(&input),
        (10, 2) => day10::part2(&input),
        (11, 1) => day11::part1(&input),
        (11, 2) => day11::part2(&input),
        // (12, 1) => day12::part1(&input),
        // (12, 2) => day12::part2(&input),
        // (13, 1) => day13::part1(&input),
        // (13, 2) => day13::part2(&input),
        // (14, 1) => day14::part1(&input),
        // (14, 2) => day14::part2(&input),
        // (15, 1) => day15::part1(&input),
        // (15, 2) => day15::part2(&input),
        // (16, 1) => day16::part1(&input),
        // (16, 2) => day16::part2(&input),
        // (17, 1) => day17::part1(&input),
        // (17, 2) => day17::part2(&input),
        // (18, 1) => day18::part1(&input),
        // (18, 2) => day18::part2(&input),
        // (19, 1) => day19::part1(&input),
        // (19, 2) => day19::part2(&input),
        // (20, 1) => day20::part1(&input),
        // (20, 2) => day20::part2(&input),
        // (21, 1) => day21::part1(&input),
        // (21, 2) => day21::part2(&input),
        // (22, 1) => day22::part1(&input),
        // (22, 2) => day22::part2(&input),
        // (23, 1) => day23::part1(&input),
        // (23, 2) => day23::part2(&input),
        // (24, 1) => day24::part1(&input),
        // (24, 2) => day24::part2(&input),
        // (25, 1) => day25::part1(&input),
        // (25, 2) => day25::part2(&input),
        _ => {
            eprintln!("Day {} part {} not found", opt.day, opt.part);
            exit(1);
        }
    };
    let end = Instant::now();
    let duration = end - start;
    let seconds = duration.as_secs();
    let sub_millis = duration.subsec_millis();
    let sub_micros = duration.subsec_micros() - (sub_millis * 1000);
    let sub_nanos = (duration.subsec_nanos() - (sub_millis * 1_000_000)) - (sub_micros * 1000);
    println!("Answer for day {} part {} is:", opt.day, opt.part);
    println!("{result}");
    println!("Time taken: {seconds}s {sub_millis}ms {sub_micros}µs {sub_nanos}ns");
}
