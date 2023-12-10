use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;

const FIGURES: [(&str, u8); 9] = [
    ("one", 1), ("two", 2), ("three", 3), ("four", 4),
    ("five", 5), ("six", 6), ("seven", 7), ("eight", 8), ("nine", 9)
];
const R_FIGURES: [(&str, u8); 9] = [
    ("eno", 1), ("owt", 2), ("eerht", 3), ("ruof", 4),
    ("evif", 5), ("xis", 6), ("neves", 7), ("thgie", 8), ("enin", 9)
];

struct Node {
    leaf: Option<u8>,
    children: HashMap<char, Node>,
}

impl Node {
    fn new(leaf: Option<u8>) -> Node {
        Node {
            leaf,
            children: HashMap::new(),
        }
    }

    /// build trie from figure
    fn add_path(figure: &str, value: u8, node: &mut Node) {
        let mut child = node;
        for c in figure.chars() {
            child = child.add_child(c, None);
        }
        child.leaf = Some(value);
    }

    /// Adds child or returns existing child.
    fn add_child(&mut self, key: char, leaf: Option<u8>) -> &mut Node {
        self.children.entry(key).or_insert_with(|| Node::new(leaf))
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, required = true, value_name = "FILE")]
    cal_doc: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let config_path = cli.cal_doc.as_deref().expect("cal_doc is required");

    let cal_doc = fs::read_to_string(config_path).expect("failed to read cal_doc");

    let t0 = Instant::now();
    let result = parse_cal_doc(&cal_doc);
    let t1 = Instant::now();
    let elapsed_time = t1 - t0;

    println!("sum of calibration values: {}", result);
    println!("took: {} µs", elapsed_time.as_micros())
}

fn parse_cal_doc(cal_doc: &str) -> u32 {
    let mut tree = Node::new(None);
    FIGURES.iter().for_each(|(fig, val)| Node::add_path(fig, *val, &mut tree));

    let mut rtree = Node::new(None);
    R_FIGURES.iter().for_each(|(fig, val)| Node::add_path(fig, *val, &mut rtree));

    cal_doc.lines().map(|line| parse_cal_doc_line(line, &tree, &rtree)).sum()
}

fn parse_cal_doc_line(cal_doc_line: &str, tree: &Node, rtree: &Node) -> u32 {
    let first = find_digit(cal_doc_line.chars(), tree);
    if first.is_none() { return 0 }
    let last = find_digit(cal_doc_line.chars().rev(), rtree);

    first.unwrap_or(0) as u32 * 10 + last.unwrap_or(0) as u32
}

fn find_digit(cal_doc_line_chars: impl Iterator<Item=char>, tree: &Node) -> Option<u8> {
    let mut nodes: Vec<&Node> = vec![];
    for c in cal_doc_line_chars {
        if c.is_numeric() {
            return Some(c.to_digit(10).unwrap_or(0) as u8);
        }
        let mut new_nodes = vec![];
        for node in nodes {
            if node.children.contains_key(&c) {
                let new_node = node.children.get(&c).unwrap();
                if new_node.leaf.is_some() {
                    return new_node.leaf;
                }
                new_nodes.push(new_node)
            }
        }
        nodes = new_nodes;

        // a new figure might be starting from this character on
        if tree.children.contains_key(&c) {
            let new_node = tree.children.get(&c).unwrap();
            nodes.push(new_node);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cal_doc_line_test() {
        let result = parse_cal_doc_line("1abc2", &get_tree(), &get_rtree());
        assert_eq!(result, 12);

        let result = parse_cal_doc_line("pqr3stu8vwx", &get_tree(), &get_rtree());
        assert_eq!(result, 38);

        let result = parse_cal_doc_line("eightwothree", &get_tree(), &get_rtree());
        assert_eq!(result, 83);

        let result = parse_cal_doc_line("twoeighthree", &get_tree(), &get_rtree());
        assert_eq!(result, 23);

        let result = parse_cal_doc_line("eightwothree", &get_tree(), &get_rtree());
        assert_eq!(result, 83);

        let result = parse_cal_doc_line("fifour", &get_tree(), &get_rtree());
        assert_eq!(result, 44);

        let result = parse_cal_doc_line("onine", &get_tree(), &get_rtree());
        assert_eq!(result, 99);
    }

    #[test]
    fn parse_cal_doc_test() {
        let result = parse_cal_doc(&cal_doc_fixture());
        assert_eq!(result, 885);
    }

    fn cal_doc_fixture() -> String {
        String::from(
            "two1nine\n\
                eightwothree\n\
                abcone2threexyz\n\
                xtwone3four\n\
                4nineeightseven2\n\
                zoneight234\n\
                7pqrstsixteen\n\
                oneight\n\
                one\n\
                twone\n\
                eightwo\n\
                nineight\n\
                eighthree\n\
                nineeight\n\
                eeeight\n\
                oooneeone\n\
                1\n\
                eightwothree\n\
                "
        )
    }

    #[test]
    fn find_digit_test() {
        let val = find_digit("oneabs".chars(), &get_tree());

        assert!(val.is_some());
        assert_eq!(1, val.unwrap())
    }

    fn get_tree() -> Node {
        let mut tree = Node::new(None);
        FIGURES.iter().for_each(|(fig, val)| Node::add_path(fig, *val, &mut tree));
        tree
    }

    fn get_rtree() -> Node {
        let mut rtree = Node::new(None);
        R_FIGURES.iter().for_each(|(fig, val)| Node::add_path(fig, *val, &mut rtree));
        rtree
    }
}
