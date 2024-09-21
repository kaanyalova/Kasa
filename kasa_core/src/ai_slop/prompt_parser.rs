use std::hash::Hash;
use std::{cmp::max, vec};

use anyhow::Ok as fuck_off;
use anyhow::Result;
use log::{error, trace};
use nom::bytes::complete::tag;
use nom::combinator::fail;
use nom::error::VerboseError;
use nom::Err;
use nom::{
    branch::alt,
    bytes::complete::{take, take_till, take_until, take_until1, take_while},
    character::{complete::char, is_alphabetic, is_alphanumeric},
    combinator::map,
    error::context,
    multi::{many0, separated_list0, separated_list1},
    sequence::separated_pair,
    Finish, IResult,
};
use serde::Deserialize;
use serde::Serialize;

use crate::db;

use super::SlopTag;

//-------------------------
#[derive(Debug, Clone)]
struct ParenthesisExpr {
    count: u32,
    inner: String,
    _type: ParenthesisType,
}

#[derive(Debug, Clone, Copy)]
enum ParenthesisType {
    /// ()
    Normal,

    /// []
    Square,

    /// {}
    Curly,

    /// <>
    Angled,
}

enum Tag {
    Single(String),
}

/// Parses the various types of parenthesis between expressions
/// returns the type and count
fn parse_parenthesis_expr_outer(input: &str) -> IResult<&str, ParenthesisExpr> {
    match input.chars().nth(0) {
        Some('(') => {
            let (input, parentheses) = many0(tag("("))(input)?;
            let (input, output) = take_until(")".repeat(parentheses.len()).as_str())(input)?;
            let (input, _) =
                take::<usize, &str, nom::error::Error<_>>(parentheses.len())(input).unwrap();

            let out = ParenthesisExpr {
                count: parentheses.len() as u32,
                inner: output.to_string(),
                _type: ParenthesisType::Normal,
            };

            return Ok((input, out));
        }

        Some('[') => {
            let (input, parentheses) = many0(tag("["))(input)?;
            let (input, output) = take_until("]".repeat(parentheses.len()).as_str())(input)?;
            let (input, _) = take(parentheses.len())(input)?;

            let out = ParenthesisExpr {
                count: parentheses.len() as u32,
                inner: output.to_string(),
                _type: ParenthesisType::Square,
            };

            return Ok((input, out));
        }

        Some('{') => {
            let (input, parentheses) = many0(tag("{"))(input)?;
            let (input, output) = take_until("}".repeat(parentheses.len()).as_str())(input)?;
            let (input, _) = take(parentheses.len())(input)?;

            let out = ParenthesisExpr {
                count: parentheses.len() as u32,
                inner: output.to_string(),
                _type: ParenthesisType::Curly,
            };

            return Ok((input, out));
        }

        Some('<') => {
            let (input, _) = tag("<")(input)?;
            let (input, output) = take_until(">")(input)?;
            let (input, _) = take(1usize)(input)?;

            let out = ParenthesisExpr {
                count: 1,
                inner: output.to_string(),
                _type: ParenthesisType::Angled,
            };

            return Ok((input, out));
        }
        // Implement custom errors
        // why are they so annoying to implement
        // https://github.com/rust-bakery/nom/blob/main/examples/custom_error.rs
        None => return fail(input),

        Some(_) => return fail(input),
    }
}
//-------------------------
/*
fn parse_tag(input: &str) -> IResult<String, (Vec<String>, f64)> {
    let (input, output) = take_until(",")(input)?;
    Ok((input.to_string(), ))
}
*/

/// Parses expression separated by commas
/// strips the tags of leading and trailing whitespace
/// `one,two,three` -> (_, ["one", "two", "three"])
fn parse_tags_list(input: &str) -> IResult<&str, Vec<&str>> {
    let (input, tags) = separated_list1(tag(","), take_while(|c| c != ','))(input)?;
    let stripped = tags.into_iter().map(|t| t.trim()).collect();
    Ok((input, stripped))
}

/// Parses a tags block that was inside parenthesis with powers into a vec and power
/// `one,two,three:1.3` -> (_, (["one", "two", "three"], 1.3))
///
/// Returns an error if no power separator (the `:` char) is present
/// `one,two,three` -> Err(...)
fn parse_tag_inner_with_power(input: &str) -> IResult<&str, (Vec<&str>, f64)> {
    let (input, (tags, power)) = separated_pair(take_until1(":"), char(':'), parse_float)(input)?;

    let (_, tags_parsed) = parse_tags_list(tags)?;

    Ok((input, (tags_parsed, power)))
}

/// The main parsing function for expressions inside parenthesis combines `parse_tag_inner_with_power`
/// and `parse_tags_list`, first tries `parse_tag_inner_with_power` if it fails falls back to `parse_tags_list`
///
/// `one,two,three` -> (_, (["one", "two", "three"], 1))
/// `one,two,three:1.3` -> (_, ["one", "two", "three", 1.3])
///
fn parse_parenthesis_inner(input: &str) -> IResult<String, (Vec<String>, f64)> {
    // First try to parse it with power
    if let Ok((input, (tags, power))) = parse_tag_inner_with_power(input) {
        return Ok((
            input.to_string(),
            (tags.into_iter().map(|tag| tag.to_string()).collect(), power),
        ));
    }

    //  if it fails simply parse regular comma separated tags
    if let Ok((input, tags)) = parse_tags_list(input) {
        return Ok((
            (input.to_string()),
            (tags.into_iter().map(|tag| tag.to_string()).collect(), 1f64),
        ));
    } else {
        Err(nom::Err::Failure(nom::error::Error::new(
            input.to_string(),
            nom::error::ErrorKind::Fail,
        )))
    }
}

fn parse_float(input: &str) -> IResult<&str, f64> {
    let (input, decimal) = take_while(|c: char| c.is_numeric())(input)?;

    let decimal: f64 = match decimal.parse::<f64>() {
        Ok(f) => f,
        Err(_) => {
            return Err(nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Fail,
            )))
        }
    };

    if input.chars().nth(0) == Some('.') {
        let (input, _) = tag(".")(input)?;
        let (input, rest) = take_while(|c: char| c.is_numeric())(input)?;
        let num_len = rest.len();
        let num: u64 = rest.parse().unwrap();

        let num_rest = num as f64 / 10_f64.powf(num_len as f64);

        return Ok((input, decimal + num_rest));
    } else {
        return Ok((input, decimal));
    }
}

/// Parses an expression surrounded by parenthesis
/// `(one,two,three:1.3)` -> `(_, (["one", "two","three"], 1.3))`
/// `((one,two,three))` -> (_, (["one", "two", "three"], 1.3))
fn parse_parenthesis_expr(input: &str) -> IResult<String, (Vec<String>, f64)> {
    let (input, parenthesis_expr) =
        parse_parenthesis_expr_outer(&input).map_err(|e| e.to_owned())?;

    match parenthesis_expr._type {
        ParenthesisType::Normal => {
            let (_input_inner, (tags, power)) = parse_parenthesis_inner(&parenthesis_expr.inner)?;

            // exception when an expression like `(tuxedo)` equals to 1.1 power
            // this also breaks something like `((tuxedo:1.0))` wrong, but nobody sane would use that
            if parenthesis_expr.count == 1 && power == 1.0 {
                return Ok((parenthesis_expr.inner, (tags, 1.1f64)));
            }

            // each parenthesis multiplies the power by 1.1
            // ignore the first parenthesis that is covered by previous exception
            let parenthesis_pow = parenthesis_expr.count - 1;
            let pow = power * 1.1f64.powf(parenthesis_pow as f64);

            return Ok((input.to_string(), (tags, pow)));
        }
        ParenthesisType::Square => {
            // check if the insides contains the characters `|`, `:` and just skip the parsing for now,
            // i might add support for the "alternating between prompts" later but it is rarely used
            if parenthesis_expr.inner.contains("|") || parenthesis_expr.inner.contains(":") {
                trace!("Parsing AI metadata for alternating prompts not supported yet.");
                return Ok((input.to_string(), (vec![], 1f64)));
            }

            let (input, (tags, power)) = parse_parenthesis_inner(&parenthesis_expr.inner)?;

            let pow = 1f64 - 1.1f64.powf(parenthesis_expr.count as f64);
            // make sure it isn't lower than 0
            let pow = pow.max(0.01);

            return Ok(((input), (tags, pow)));
        }
        ParenthesisType::Curly => {
            // Same as `()`, used by NAI?, but with lower power

            let (input, (tags, power)) = parse_parenthesis_inner(&parenthesis_expr.inner)?;

            // exception when an expression like `(tuxedo)` equals to 1.1 power
            // this also breaks something like `((tuxedo:1.0))` wrong, but nobody sane would use that
            if parenthesis_expr.count == 1 && power == 1.0 {
                return Ok((input, (tags, 1.1f64)));
            }

            // each parenthesis multiplies the power by 1.1
            // ignore the first parenthesis that is covered by previous exception
            let parenthesis_pow = parenthesis_expr.count - 1;
            let pow = power * 1.05f64.powf(parenthesis_pow as f64);

            return Ok((input, (tags, pow)));
        }
        ParenthesisType::Angled => {
            trace!("Parsing Lora metadata not implemented yet.");
            // Lora parsing unimplemented
            return Ok((input.to_string(), (vec![], 1f64)));
        }
    }
}

/// Tries parsing an parenthesis expression, parses a regular value if that fails, tries to parse as much
/// values as possible, does not return errors
pub fn parse_prompt(input: &str) -> Vec<SlopTag> {
    let mut results: Vec<SlopTag> = vec![];

    // cleanup the input from common mistakes
    let remaining = input.to_string();
    let remaining = remaining.trim();
    let remaining = remaining.trim_start_matches(',');
    let mut remaining = remaining.trim().to_string();

    loop {
        match parse_parenthesis_expr(&remaining) {
            Ok((input, (tags, pow))) => {
                let mut packed: Vec<SlopTag> = tags
                    .into_iter()
                    .map(|tag| SlopTag {
                        name: tag,
                        power: pow,
                    })
                    .collect();

                results.append(&mut packed);

                remaining = input;
            }
            std::result::Result::Err(_) => {
                // rustfmt stop removing the braces from match statement every time i save, holy shit
                match parse_tag(&remaining) {
                    Ok((input, (tag, pow))) => {
                        let tag = SlopTag {
                            name: tag,
                            power: pow,
                        };
                        results.push(tag);

                        remaining = input.to_string();
                    }
                    std::result::Result::Err(e) => {
                        error!(
                            "Something went wrong while parsing ai metadata, error details: {}",
                            e.to_string()
                        );
                        // just return all results we can get
                        return results;
                    }
                }
            }
        }

        // no i am not using nom
        remaining = remaining.trim().to_string();
        remaining = remaining.trim_start_matches(",").to_string();
        remaining = remaining.trim().to_string();

        if remaining == "" {
            return results;
        }
    }
}
#[allow(unused)] // but, it is used?? what is going on here
fn parse_tag(input: &str) -> IResult<&str, (String, f64)> {
    // we are at last tag
    match take_until(",")(input) {
        Ok((input, output)) => return Ok((input, (output.to_string(), 1f64))),
        std::result::Result::Err(e) => {
            // end of prompt
            if !input.contains(",") {
                return Ok(("", (input.to_string(), 1f64)));
            } else {
                return Err(e);
            }
        }
    };
}

#[test]
fn test_parser() {
    let tags = parse_prompt("i, (hate, writing:1.3), ((parsers))");

    let excepted = vec![
        SlopTag::new("i", 1.0),
        SlopTag::new("hate", 1.3),
        SlopTag::new("writing", 1.3),
        SlopTag::new("parsers", 1.1),
    ];

    assert_eq!(tags, excepted);
}

#[test]
fn parse_float_test() {
    let (_, pi) = parse_float("3.14").unwrap();
    assert_eq!(pi, 3.14f64);

    let (_, num) = parse_float("31").unwrap();
    assert_eq!(num, 31f64);
}

#[test]
fn parse_tags_test() {
    let (_, out) = parse_tags_list("one,two,three").unwrap();
    assert_eq!(vec!["one", "two", "three"], out);

    let (_, out) = parse_tags_list("one").unwrap();
    assert_eq!(vec!["one"], out);
}

#[test]
fn parse_tags_inner_test() {
    let inp = "one,two,three:1.3";

    let (_, (tags, power)) = parse_parenthesis_inner(&inp).unwrap();
    assert_eq!(tags, vec!["one", "two", "three"]);
    assert_eq!(power, 1.3f64);

    let inp = "one,two,three";
    let (_, (tags, power)) = parse_parenthesis_inner(&inp).unwrap();
    assert_eq!(tags, vec!["one", "two", "three"]);
    assert_eq!(power, 1f64);
}

#[test]
fn parse_parenthesis_expr_test() {
    let inp = "(one,two,three,four:1.3)";
    let (_, (tags, power)) = parse_parenthesis_expr(inp).unwrap();

    assert_eq!(power, 1.3f64);
    assert_eq!(tags, vec!["one", "two", "three", "four"]);
}
