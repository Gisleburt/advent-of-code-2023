use nom::character::complete;
use nom::character::complete::{newline, space1};
use nom::multi::separated_list1;
use nom::IResult;

type Number = i64;

fn next_sequence(v: &[Number]) -> Vec<Number> {
    let mut output = Vec::with_capacity(v.len() - 1);
    let mut iter = v.iter().peekable();
    while let (Some(a), Some(b)) = (iter.next(), iter.peek()) {
        output.push(*b - a)
    }
    output
}

fn next_sequences_rec(mut v: Vec<Vec<Number>>) -> Vec<Vec<Number>> {
    let last = v
        .last()
        .and_then(|last| (!is_end_sequence(last)).then_some(last));

    if let Some(last) = last {
        v.push(next_sequence(last));
        next_sequences_rec(v)
    } else {
        v
    }
}

fn is_end_sequence(v: &[Number]) -> bool {
    v.iter().all(|i| *i == 0)
}

fn add_predictions(v: &mut [Vec<Number>]) {
    let mut iter = v.iter_mut().rev();
    let mut prev = iter.next().expect("Need at least one entry");
    if !is_end_sequence(prev) {
        panic!("Last entry must be end sequencxe")
    }
    prev.push(0);

    for next in iter {
        let prev_last = prev.last().expect("Did not expect empty sequence");
        let next_last = next.last().expect("Did not expect empty sequence");
        next.push(prev_last + next_last);
        prev = next;
    }
}

fn add_predictions_back(v: &mut [Vec<Number>]) {
    let mut iter = v.iter_mut().rev();
    let mut prev = iter.next().expect("Need at least one entry");
    if !is_end_sequence(prev) {
        panic!("Last entry must be end sequencxe")
    }
    prev.push(0);

    for next in iter {
        let prev_last = prev.first().expect("Did not expect empty sequence");
        let next_last = next.first().expect("Did not expect empty sequence");
        next.insert(0, next_last - prev_last);
        prev = next;
    }
}

fn parse_input(input: &str) -> IResult<&str, Vec<Vec<Number>>> {
    separated_list1(newline, separated_list1(space1, complete::i64))(input)
}

pub fn part1(input: &str) -> String {
    let vectors = parse_input(input).expect("invalid input").1;
    vectors
        .into_iter()
        .map(|line| next_sequences_rec(vec![line]))
        .map(|mut sequence| {
            add_predictions(&mut sequence);
            sequence
        })
        .map(|predictions| {
            *predictions
                .first()
                .expect("Empty predictions")
                .last()
                .expect("Empty prediction")
        })
        .sum::<Number>()
        .to_string()
}

pub fn part2(input: &str) -> String {
    let vectors = parse_input(input).expect("invalid input").1;
    vectors
        .into_iter()
        .map(|line| next_sequences_rec(vec![line]))
        .map(|mut sequence| {
            add_predictions_back(&mut sequence);
            sequence
        })
        .map(|predictions| {
            *predictions
                .first()
                .expect("Empty predictions")
                .first()
                .expect("Empty prediction")
        })
        .sum::<Number>()
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    mod parts {
        use super::*;

        #[test]
        fn test_next_sequence() {
            let seq1 = vec![1, 3, 6, 10, 15, 21];
            let seq2 = vec![2, 3, 4, 5, 6];
            let seq3 = vec![1, 1, 1, 1];
            let seq4 = vec![0, 0, 0];

            assert_eq!(next_sequence(&seq1), seq2);
            assert_eq!(next_sequence(&seq2), seq3);
            assert_eq!(next_sequence(&seq3), seq4);
        }

        #[test]
        fn test_next_req() {
            let seq1 = vec![1, 3, 6, 10, 15, 21];
            let seq2 = vec![2, 3, 4, 5, 6];
            let seq3 = vec![1, 1, 1, 1];
            let seq4 = vec![0, 0, 0];

            assert_eq!(
                next_sequences_rec(vec![seq1.clone()]),
                vec![seq1, seq2, seq3, seq4]
            );
        }

        #[test]
        fn test_add_predictions() {
            let seq1 = vec![1, 3, 6, 10, 15, 21];
            let seq2 = vec![2, 3, 4, 5, 6];
            let seq3 = vec![1, 1, 1, 1];
            let seq4 = vec![0, 0, 0];

            let p_seq1 = vec![1, 3, 6, 10, 15, 21, 28];
            let p_seq2 = vec![2, 3, 4, 5, 6, 7];
            let p_seq3 = vec![1, 1, 1, 1, 1];
            let p_seq4 = vec![0, 0, 0, 0];

            let mut sequence = vec![seq1, seq2, seq3, seq4];
            let expected_sequence = vec![p_seq1, p_seq2, p_seq3, p_seq4];
            add_predictions(&mut sequence);

            assert_eq!(sequence, expected_sequence);
        }

        #[test]
        fn test_add_predictions_back() {
            let seq1 = vec![10, 13, 16, 21, 30, 45];
            let seq2 = vec![3, 3, 5, 9, 15];
            let seq3 = vec![0, 2, 4, 6];
            let seq4 = vec![2, 2, 2, 2];
            let seq5 = vec![0, 0, 0];

            let p_seq1 = vec![5, 10, 13, 16, 21, 30, 45];
            let p_seq2 = vec![5, 3, 3, 5, 9, 15];
            let p_seq3 = vec![-2, 0, 2, 4, 6];
            let p_seq4 = vec![2, 2, 2, 2, 2];
            let p_seq5 = vec![0, 0, 0, 0];

            let mut sequence = vec![seq1, seq2, seq3, seq4, seq5];
            let expected_sequence = vec![p_seq1, p_seq2, p_seq3, p_seq4, p_seq5];
            add_predictions_back(&mut sequence);

            assert_eq!(sequence, expected_sequence);
        }

        #[test]
        fn text_parse_input() {
            let input = "0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";
            let expected = vec![
                vec![0, 3, 6, 9, 12, 15],
                vec![1, 3, 6, 10, 15, 21],
                vec![10, 13, 16, 21, 30, 45],
            ];

            assert_eq!(parse_input(input).unwrap().1, expected);

            let input = "0 -3 -6 -9 -12 -15
1 -3 -6 -10 -15 -21
10 -13 -16 -21 -30 -45";
            let expected = vec![
                vec![0, -3, -6, -9, -12, -15],
                vec![1, -3, -6, -10, -15, -21],
                vec![10, -13, -16, -21, -30, -45],
            ];

            assert_eq!(parse_input(input).unwrap().1, expected);
        }
    }

    #[test]
    fn test_part1() {
        let input = "0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";
        assert_eq!(part1(input), "114")
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "")
    }
}
