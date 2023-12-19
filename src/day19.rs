use derive_more::{Deref, From};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{alpha1, newline};
use nom::combinator::{map, value};
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use nom::IResult;

use Category::*;
use Outcome::*;
use RuleType::*;

#[derive(Debug, Clone, PartialEq)]
enum Outcome {
    Accepted,
    Rejected,
    ContinueTo(String),
}

fn parse_outcome(input: &str) -> IResult<&str, Outcome> {
    alt((
        value(Accepted, complete::char('A')),
        value(Rejected, complete::char('R')),
        map(alpha1, |s: &str| ContinueTo(s.to_string())),
    ))(input)
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Category {
    Cool,
    Musical,
    Aerodynamic,
    Shiny,
}

fn parse_category(input: &str) -> IResult<&str, Category> {
    alt((
        value(Cool, complete::char('x')),
        value(Musical, complete::char('m')),
        value(Aerodynamic, complete::char('a')),
        value(Shiny, complete::char('s')),
    ))(input)
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum RuleType {
    GreaterThan,
    LessThan,
}

fn parse_rule_type(input: &str) -> IResult<&str, RuleType> {
    alt((
        value(GreaterThan, complete::char('>')),
        value(LessThan, complete::char('<')),
    ))(input)
}

#[derive(Debug, Clone, PartialEq)]
struct Rule {
    category: Category,
    rule_type: RuleType,
    value: u64,
    outcome: Outcome,
}

impl Rule {
    fn process_part(&self, part: Part) -> Option<Outcome> {
        let value = part.value_for_category(self.category);
        match self.rule_type {
            GreaterThan => (value > self.value).then_some(self.outcome.clone()),
            LessThan => (value < self.value).then_some(self.outcome.clone()),
        }
    }
}

fn parse_rule(input: &str) -> IResult<&str, Rule> {
    map(
        tuple((
            parse_category,
            parse_rule_type,
            complete::u64,
            complete::char(':'),
            parse_outcome,
        )),
        |(category, rule_type, value, _, outcome)| Rule {
            category,
            rule_type,
            value,
            outcome,
        },
    )(input)
}

#[derive(Debug, Clone, PartialEq)]
enum RuleOrOutcome {
    Rule(Rule),
    Outcome(Outcome),
}

fn parse_rule_or_outcome(input: &str) -> IResult<&str, RuleOrOutcome> {
    alt((
        map(parse_rule, RuleOrOutcome::Rule),
        map(parse_outcome, RuleOrOutcome::Outcome),
    ))(input)
}

#[derive(Debug, Clone, PartialEq)]
struct Workflow {
    label: String,
    rules: Vec<RuleOrOutcome>,
}

impl Workflow {
    fn process_part(&self, part: Part) -> Outcome {
        self.rules
            .iter()
            .find_map(|rule_or_outcome| match rule_or_outcome {
                RuleOrOutcome::Rule(rule) => rule.process_part(part),
                RuleOrOutcome::Outcome(outcome) => Some(outcome.clone()),
            })
            .unwrap_or_else(|| panic!("Workflow {self:?} did not match part {part:?}"))
    }
}

fn parse_workflow(input: &str) -> IResult<&str, Workflow> {
    map(
        tuple((
            alpha1,
            delimited(
                complete::char('{'),
                separated_list1(complete::char(','), parse_rule_or_outcome),
                complete::char('}'),
            ),
        )),
        |(label, rules)| Workflow {
            label: label.to_string(),
            rules,
        },
    )(input)
}

#[derive(Debug, Clone, PartialEq, From, Deref)]
struct Workflows(Vec<Workflow>);

impl Workflows {
    fn process_part(&self, part: Part, label: &str) -> Outcome {
        let workflow = self
            .iter()
            .find(|workflow| workflow.label == label)
            .unwrap_or_else(|| panic!("Could not find {label}"));
        workflow.process_part(part)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Part {
    x: u64,
    m: u64,
    a: u64,
    s: u64,
}

impl Part {
    fn value_for_category(&self, category: Category) -> u64 {
        match category {
            Cool => self.x,
            Musical => self.m,
            Aerodynamic => self.a,
            Shiny => self.s,
        }
    }

    fn total_value(&self) -> u64 {
        self.x + self.m + self.a + self.s
    }
}

fn parse_part(input: &str) -> IResult<&str, Part> {
    map(
        delimited(
            complete::char('{'),
            tuple((
                delimited(tag("x="), complete::u64, tag(",")),
                delimited(tag("m="), complete::u64, tag(",")),
                delimited(tag("a="), complete::u64, tag(",")),
                preceded(tag("s="), complete::u64),
            )),
            complete::char('}'),
        ),
        |(x, m, a, s)| Part { x, m, a, s },
    )(input)
}

fn parse_input(input: &str) -> IResult<&str, (Workflows, Vec<Part>)> {
    separated_pair(
        map(separated_list1(newline, parse_workflow), Workflows),
        pair(newline, newline),
        separated_list1(newline, parse_part),
    )(input)
}

pub fn part1(input: &str) -> String {
    let (workflows, parts) = parse_input(input).unwrap().1;

    let mut accepted: Vec<Part> = vec![];
    for part in parts.into_iter() {
        let mut workflow_label = "in".to_string();
        loop {
            let outcome = workflows.process_part(part, &workflow_label);
            match outcome {
                Accepted => {
                    accepted.push(part);
                    break;
                }
                Rejected => break,
                ContinueTo(label) => workflow_label = label,
            }
        }
    }

    accepted
        .into_iter()
        .map(|part| part.total_value())
        .sum::<u64>()
        .to_string()
}

pub fn part2(_input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    mod workflow {
        use super::*;

        #[test]
        fn test_parse_workflow() {
            let input = "ex{x>10:one,m<20:two,a>30:R,A}";
            let workflow = parse_workflow(input).unwrap().1;
            assert_eq!(
                workflow,
                Workflow {
                    label: "ex".to_string(),
                    rules: vec![
                        RuleOrOutcome::Rule(Rule {
                            category: Cool,
                            rule_type: GreaterThan,
                            value: 10,
                            outcome: ContinueTo("one".to_string()),
                        }),
                        RuleOrOutcome::Rule(Rule {
                            category: Musical,
                            rule_type: LessThan,
                            value: 20,
                            outcome: ContinueTo("two".to_string()),
                        }),
                        RuleOrOutcome::Rule(Rule {
                            category: Aerodynamic,
                            rule_type: GreaterThan,
                            value: 30,
                            outcome: Rejected,
                        }),
                        RuleOrOutcome::Outcome(Accepted),
                    ],
                }
            )
        }
    }

    mod part {
        use super::*;

        #[test]
        fn test_parse_part() {
            let input = "{x=787,m=2655,a=1222,s=2876}";
            let part = parse_part(input).unwrap().1;
            assert_eq!(
                part,
                Part {
                    x: 787,
                    m: 2655,
                    a: 1222,
                    s: 2876,
                }
            )
        }
    }

    #[test]
    fn test_part1() {
        let input = "px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}";
        assert_eq!(part1(input), "19114");
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "");
    }
}
