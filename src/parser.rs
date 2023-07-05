// #![allow(unused)]

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till1, take_until, take_while1},
    character::{
        complete::{alphanumeric1, anychar, digit1, line_ending, space0},
        is_alphabetic, is_space,
    },
    combinator::{all_consuming, eof, map, map_res},
    error::make_error,
    multi::{fold_many1, many1, many_till},
    sequence::{delimited, pair, separated_pair, terminated, tuple},
    IResult,
};
use rand::prelude::*;
use std::error::Error;
use std::{collections::HashMap, fmt, vec};

use crate::{Rule, RuleInst, Table};

type Indices = (usize, usize);

#[derive(Debug)]
pub struct ParsedRule {
    indices: Indices,
    rule: Rule,
}

// --------- Tabol ---------
pub fn parse_tables(input: &str) -> IResult<&str, Vec<Table>> {
    let (remaining, (frontmatter, rules)) =
        all_consuming(tuple((parse_frontmatter, parse_rules)))(input)?;

    let mut choices = vec![];
    let rules = rules
        .iter()
        .enumerate()
        .map(|(i, parsed_rule)| {
            let (min, max) = parsed_rule.indices;

            for _ in min..=max {
                choices.push(i);
            }

            parsed_rule.rule.clone()
        })
        .collect();

    Ok((
        remaining,
        vec![Table {
            title: frontmatter.title,
            id: frontmatter.id,
            rules,
            choices,
        }],
    ))
}

struct Frontmatter {
    pub title: String,
    pub id: String,
}

fn parse_frontmatter(input: &str) -> IResult<&str, Frontmatter> {
    let (remaining, attrs) = delimited(
        pair(tag("---"), line_ending),
        fold_many1(
            parse_frontmatter_attr,
            HashMap::new,
            |mut acc: HashMap<_, _>, (k, v)| {
                acc.insert(k, v.to_string());
                acc
            },
        ),
        pair(tag("---"), line_ending),
    )(input)?;

    // arbitary frontmatter???
    let id = attrs.get("id").ok_or(nom::Err::Failure(make_error(
        input,
        nom::error::ErrorKind::Many1,
    )))?;

    let title = attrs.get("title").ok_or(nom::Err::Failure(make_error(
        input,
        nom::error::ErrorKind::Many1,
    )))?;

    Ok((
        remaining,
        Frontmatter {
            id: id.to_string(),
            title: title.to_string(),
        },
    ))
}

fn parse_frontmatter_attr(input: &str) -> IResult<&str, (&str, &str)> {
    terminated(
        separated_pair(alphanumeric1, tag(": "), alphanumeric1),
        line_ending,
    )(input)
}

/**
 * input:
 *    1: foo
 *    2-5: bar
 *    6-10: baz
 */
fn parse_rules(input: &str) -> IResult<&str, Vec<ParsedRule>> {
    many1(terminated(parse_one_rule, alt((eof, line_ending))))(input)
}

pub fn parse_one_rule(input: &str) -> IResult<&str, ParsedRule> {
    map_res(
        separated_pair(parse_indices, tag(": "), words),
        |(indices, raw)| {
            let rule = Rule::new(raw.to_string()).unwrap();
            // let rule = Rule::new(raw.to_string())
            //     .map_err(|_| nom::error::Error::new(input, nom::error::ErrorKind::Many1))?;
            // .map_err(|err| nom::error::Error::new(input, nom::error::ErrorKind::MapRes))?;

            Ok::<ParsedRule, nom::error::Error<nom::error::ErrorKind>>(ParsedRule { indices, rule })
        },
    )(input)
}

fn parse_indices(input: &str) -> IResult<&str, Indices> {
    alt((
        separated_pair(parse_int, tag("-"), parse_int),
        map_res(parse_int, |n: usize| {
            Ok::<Indices, nom::error::Error<nom::error::ErrorKind>>((n, n)) // this turbofish seems _incredibly_ unnecessary, but rust makes me specify it
        }),
    ))(input)
}

fn parse_int(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| str::parse::<usize>(s))(input)
}

// --------- Rule ---------
pub fn parse_rule(input: &str) -> IResult<&str, Vec<RuleInst>> {
    all_consuming(many1(alt((parse_rule_literal, parse_rule_interpolation))))(input)
}

fn parse_rule_literal(input: &str) -> IResult<&str, RuleInst> {
    map(alt((take_until("{{"), words)), |s: &str| {
        RuleInst::Literal(s.to_string())
    })(input)
}

fn parse_rule_interpolation(input: &str) -> IResult<&str, RuleInst> {
    map(delimited(tag("{{"), ident, tag("}}")), |s: &str| {
        RuleInst::Interpolation(s.to_string())
    })(input)
}

fn words(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == ' ')(input)
}

fn ident(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}
