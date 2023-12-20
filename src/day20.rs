use std::collections::{HashMap, VecDeque};

use derive_more::{Deref, DerefMut, From};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, newline};
use nom::combinator::{into, map};
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;

use Pulse::*;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Pulse {
    High,
    Low,
}

impl Pulse {
    fn flip(&self) -> Self {
        match self {
            High => Low,
            Low => High,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Broadcaster {
    label: String,
    outputs: Vec<String>,
}

impl Broadcaster {
    fn process_message(&mut self, message: Message) -> Vec<Message> {
        assert_eq!(self.label, message.to);
        self.outputs
            .iter()
            .map(|to| Message {
                to: to.clone(),
                from: self.label.clone(),
                pulse: message.pulse,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FlipFlop {
    label: String,
    is_on: bool,
    outputs: Vec<String>,
}

impl FlipFlop {
    fn process_message(&mut self, message: Message) -> Vec<Message> {
        assert_eq!(self.label, message.to);

        if message.pulse == High {
            return vec![];
        }

        self.is_on = !self.is_on;

        let pulse = if self.is_on { High } else { Low };

        self.outputs
            .iter()
            .map(|to| Message {
                to: to.clone(),
                from: self.label.clone(),
                pulse,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Conjunction {
    label: String,
    inputs: HashMap<String, Pulse>,
    outputs: Vec<String>,
}

impl Conjunction {
    fn connect_input(&mut self, input: &str) {
        self.inputs.insert(input.to_string(), Low);
    }

    fn process_message(&mut self, message: Message) -> Vec<Message> {
        assert_eq!(self.label, message.to);

        self.inputs.insert(message.from, message.pulse);

        let pulse = self
            .inputs
            .values()
            .find(|pulse| *pulse == &Low)
            .unwrap_or(&High)
            .flip();

        self.outputs
            .iter()
            .map(|to| Message {
                to: to.clone(),
                from: self.label.clone(),
                pulse,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, From)]
enum Module {
    Broadcaster(Broadcaster),
    FlipFlop(FlipFlop),
    Conjunction(Conjunction),
}

impl Module {
    fn broadcaster(&mut self) -> Option<&mut Broadcaster> {
        match self {
            Module::Broadcaster(module) => Some(module),
            _ => None,
        }
    }

    fn flip_flop(&mut self) -> Option<&mut FlipFlop> {
        match self {
            Module::FlipFlop(module) => Some(module),
            _ => None,
        }
    }

    fn conjunction(&mut self) -> Option<&mut Conjunction> {
        match self {
            Module::Conjunction(module) => Some(module),
            _ => None,
        }
    }

    fn get_label(&self) -> &str {
        match self {
            Module::Broadcaster(broadcaster) => &broadcaster.label,
            Module::FlipFlop(flip_flop) => &flip_flop.label,
            Module::Conjunction(conjunction) => &conjunction.label,
        }
    }

    fn get_outputs(&self) -> &Vec<String> {
        match self {
            Module::Broadcaster(b) => &b.outputs,
            Module::FlipFlop(f) => &f.outputs,
            Module::Conjunction(c) => &c.outputs,
        }
    }

    fn get_connections(&self) -> Vec<(String, String)> {
        self.get_outputs()
            .iter()
            .map(|output| (self.get_label().to_string(), output.to_string()))
            .collect()
    }

    fn process_message(&mut self, message: Message) -> Vec<Message> {
        match self {
            Module::Broadcaster(b) => b.process_message(message),
            Module::FlipFlop(f) => f.process_message(message),
            Module::Conjunction(c) => c.process_message(message),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deref, DerefMut, From)]
struct Modules(Vec<Module>);

impl Modules {
    fn connect_conjunctions(&mut self) {
        let connections = self
            .iter()
            .flat_map(|module| module.get_connections())
            .collect_vec();
        self.iter_mut()
            .filter_map(|module| module.conjunction())
            .for_each(|conjunction| {
                let label = conjunction.label.clone();
                connections
                    .iter()
                    .filter(|(_from, to)| &label == to)
                    .for_each(|(from, _to)| conjunction.connect_input(from))
            })
    }

    fn process_message(&mut self, message: Message) -> Vec<Message> {
        self.iter_mut()
            .find(|module| module.get_label() == message.to)
            .map(|module| module.process_message(message.clone()))
            .unwrap_or_else(|| {
                eprintln!("unable to find module {}", message.to);
                vec![]
            })
    }
}

struct Communications {
    modules: Modules,
    message_queue: VecDeque<Message>,
    low_counter: usize,
    high_counter: usize,
}

impl Communications {
    fn new(mut modules: Modules) -> Self {
        modules.connect_conjunctions();
        Self {
            modules,
            message_queue: VecDeque::new(),
            low_counter: 0,
            high_counter: 0,
        }
    }

    fn push_button(&mut self) {
        self.message_queue.push_back(Message {
            to: "broadcaster".to_string(),
            from: "button".to_string(),
            pulse: Low,
        });

        while let Some(message) = self.message_queue.pop_front() {
            match message.pulse {
                High => self.high_counter = self.high_counter + 1,
                Low => self.low_counter = self.low_counter + 1,
            }

            let messages = self.modules.process_message(message);
            self.message_queue.extend(messages);
        }
    }

    fn value(&self) -> usize {
        self.high_counter * self.low_counter
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Message {
    to: String,
    from: String,
    pulse: Pulse,
}

fn parse_broadcaster(input: &str) -> IResult<&str, Broadcaster> {
    map(
        separated_pair(
            tag("broadcaster"),
            tag(" -> "),
            separated_list1(tag(", "), alpha1),
        ),
        |(label, outputs): (&str, Vec<&str>)| Broadcaster {
            label: label.to_string(),
            outputs: outputs.into_iter().map(|o| o.to_string()).collect(),
        },
    )(input)
}

fn parse_flip_flop(input: &str) -> IResult<&str, FlipFlop> {
    map(
        separated_pair(
            preceded(tag("%"), alpha1),
            tag(" -> "),
            separated_list1(tag(", "), alpha1),
        ),
        |(label, outputs): (&str, Vec<&str>)| FlipFlop {
            label: label.to_string(),
            is_on: false,
            outputs: outputs.into_iter().map(|o| o.to_string()).collect(),
        },
    )(input)
}

fn parse_conjunction(input: &str) -> IResult<&str, Conjunction> {
    map(
        separated_pair(
            preceded(tag("&"), alpha1),
            tag(" -> "),
            separated_list1(tag(", "), alpha1),
        ),
        |(label, outputs): (&str, Vec<&str>)| Conjunction {
            label: label.to_string(),
            inputs: HashMap::new(),
            outputs: outputs.into_iter().map(|o| o.to_string()).collect(),
        },
    )(input)
}

fn parse_module(input: &str) -> IResult<&str, Module> {
    alt((
        into(parse_broadcaster),
        into(parse_flip_flop),
        into(parse_conjunction),
    ))(input)
}

fn parse_modules(input: &str) -> IResult<&str, Modules> {
    into(separated_list1(newline, parse_module))(input)
}

pub fn part1(input: &str) -> String {
    let modules = parse_modules(input).unwrap().1;
    let mut communications = Communications::new(modules);
    for _ in 0..1000 {
        communications.push_button();
    }
    communications.value().to_string()
}

pub fn part2(_input: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    mod parsers {
        use super::*;

        #[test]
        fn test_parse_broadcaster() {
            let input = "broadcaster -> a, b, c\n";
            let result = parse_broadcaster(input);
            assert_eq!(
                result,
                Ok((
                    "\n",
                    Broadcaster {
                        label: "broadcaster".to_string(),
                        outputs: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                    }
                ))
            )
        }

        #[test]
        fn test_parse_flip_flop() {
            let input = "%a -> b\n";
            let result = parse_flip_flop(input);
            assert_eq!(
                result,
                Ok((
                    "\n",
                    FlipFlop {
                        label: "a".to_string(),
                        is_on: false,
                        outputs: vec!["b".to_string()],
                    }
                ))
            )
        }

        #[test]
        fn test_parse_conjunction() {
            let input = "&inv -> a\n";
            let result = parse_conjunction(input);
            assert_eq!(
                result,
                Ok((
                    "\n",
                    Conjunction {
                        label: "inv".to_string(),
                        inputs: HashMap::new(),
                        outputs: vec!["a".to_string()],
                    }
                ))
            )
        }

        #[test]
        fn test_parse_modules() {
            let input = "broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a";
            let mut mods = parse_modules(input).unwrap().1;
            assert_eq!(mods.len(), 5);
            assert_eq!(mods.iter_mut().filter_map(|m| m.broadcaster()).count(), 1);
            assert_eq!(mods.iter_mut().filter_map(|m| m.flip_flop()).count(), 3);
            assert_eq!(mods.iter_mut().filter_map(|m| m.conjunction()).count(), 1);
        }
    }

    #[test]
    fn test_part1() {
        let input = "broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a";
        assert_eq!(part1(input), "32000000");

        let input = "broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output";
        assert_eq!(part1(input), "11687500");
    }

    #[ignore]
    #[test]
    fn test_part2() {
        let input = "";
        assert_eq!(part2(input), "");
    }
}
