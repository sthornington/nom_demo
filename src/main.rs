use std::{env, io};
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, multispace0, not_line_ending};
use nom::combinator::map;
use nom::{IResult, Parser};
use nom::multi::{many0, separated_list0, separated_list1};
use nom::sequence::{delimited, terminated};

#[derive(Debug, PartialEq)]
pub struct Comment<'a> {
    text: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct CSV<'a> {
    vals: Vec<&'a str>,
}

#[derive(Debug, PartialEq)]
pub enum Line<'a> {
    Comment(Comment<'a>),
    Data(CSV<'a>),
}

impl<'a> From<Comment<'a>> for Line<'a> {
    fn from(x: Comment<'a>) -> Self {
        Line::Comment(x)
    }
}

impl<'a> From<CSV<'a>> for Line<'a> {
    fn from(x: CSV<'a>) -> Self {
        Line::Data(x)
    }
}

// a lot of this is deliberately imperative.  could make it much "tighter" using more combinators
// like 'and'
// found most of these combinators at https://github.com/Geal/nom/blob/main/doc/choosing_a_combinator.md
// but it didn't include and_then ???
fn parse_comment(input: &str) -> IResult<&str, Comment> {
    let (input, _) = tag("#")(input)?;
    let (input, text) = not_line_ending(input)?;
    Ok((input, Comment { text }))
}

// like this
fn parse_comment_2(input: &str) -> IResult<&str, Comment> {
    map(tag("#").and_then(not_line_ending), |text| Comment { text })(input)
}

fn parse_csv(input: &str) -> IResult<&str, CSV> {
    let (input, v) = separated_list1(tag(","), alphanumeric1)(input)?;
    Ok((input, CSV { vals: v}))
}

fn parse_line(input: &str) -> IResult<&str, Line> {
    let line_comment = map(parse_comment, |x| Line::from(x));
    let line_data = map(parse_csv, |x| Line::from(x));
    alt((line_comment, line_data))(input)
}

fn parse_lines(input: &str) -> IResult<&str, Vec<Line>> {
    // terminated ...., multispace0 from
    // https://github.com/Geal/nom/blob/main/doc/nom_recipes.md#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser
    terminated(separated_list0(tag("\n"), parse_line), multispace0)(input)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() > 1);
    let data = std::fs::read_to_string(&args[1]).unwrap();
    let (tail, lines) = parse_lines(&data).unwrap();
    assert_eq!(tail, "");
    let data_lines: Vec<&CSV> = lines.iter().filter_map(|x| {
        if let Line::Data(x) = x {
            Some(x)
        } else {
            None
        }
    }).collect();
    println!("{:?}", data_lines);
}

#[test]
fn test_parse_comment() {
    assert_eq!(parse_comment("#woob"), Ok(("", Comment { text: "woob" })));
}

#[test]
fn test_parse_comment_2() {
    assert_eq!(parse_comment("#woob"), Ok(("", Comment { text: "woob" })));
}

#[test]
fn test_parse_comment2() {
    assert_eq!(parse_comment("#woob\n#foob"), Ok(("\n#foob", Comment { text: "woob" })));
}

#[test]
fn test_parse_not_comment() {
    assert!(parse_comment("woob").is_err());
}

#[test]
fn test_parse_csv() {
    assert_eq!(parse_csv("x,y"), Ok(("", CSV { vals: vec!["x", "y"] })));
}

#[test]
fn test_parse_line_csv() {
    assert_eq!(parse_line("x,y"), Ok(("", Line::Data(CSV { vals: vec!["x", "y"] }))));
}

#[test]
fn test_parse_line_comment() {
    assert_eq!(parse_line("#xy"), Ok(("", Line::Comment(Comment { text: "xy" }))));
}

#[test]
fn test_parse_lines() {
    let src = "#xy\n6,7";
    let expected = vec![Comment { text: "xy"}.into(), CSV { vals: vec!["6","7"] }.into()];
    assert_eq!(parse_lines(src), Ok(("", expected)));
}

#[test]
fn test_parse_lines_trailing_newlines() {
    let src = "#xy\n6,7\n\n";
    let expected = vec![Comment { text: "xy"}.into(), CSV { vals: vec!["6","7"] }.into()];
    assert_eq!(parse_lines(src), Ok(("", expected)));
}

