mod day1;
mod day2;

use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(short = "d", long = "day")]
    day: usize,
    #[structopt(short = "p", long = "part")]
    part: usize,
}

fn main() {
    let opt = Opt::from_args();
    let input = read_to_string(opt.input).expect("input not found");

    let result = match (opt.day, opt.part) {
        (1, 1) => day1::part1(&input),
        (1, 2) => day1::part2(&input),
        (2, 1) => day2::part1(&input),
        (2, 2) => day2::part2(&input),
        _ => {
            eprintln!("Day {} part {} not found", opt.day, opt.part);
            exit(1);
        }
    };
    println!("Answer for day {} part {} is {}", opt.day, opt.part, result)
}
