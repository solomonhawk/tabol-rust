// #![allow(unused)]

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till1, take_until, take_while, take_while1},
    character::{
        complete::{alphanumeric1, anychar, digit1, line_ending, not_line_ending, space0},
        is_alphabetic, is_space,
    },
    combinator::{all_consuming, consumed, eof, map, map_res, recognize},
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
    let (remaining, tables) = all_consuming(many1(table))(input)?;
    Ok((remaining, tables))
}

fn table(input: &str) -> IResult<&str, Table> {
    let (remaining, (frontmatter, rules, _)) = tuple((frontmatter, rules, whitespace))(input)?;

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
        Table {
            title: frontmatter.title,
            id: frontmatter.id,
            rules,
            choices,
        },
    ))
}

struct Frontmatter {
    pub title: String,
    pub id: String,
}

fn frontmatter(input: &str) -> IResult<&str, Frontmatter> {
    let (remaining, attrs) = delimited(
        pair(tag("---"), line_ending),
        fold_many1(
            frontmatter_attr,
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

fn frontmatter_attr(input: &str) -> IResult<&str, (&str, &str)> {
    terminated(
        separated_pair(alphanumeric1, tag(": "), not_line_ending),
        line_ending,
    )(input)
}

fn rules(input: &str) -> IResult<&str, Vec<ParsedRule>> {
    many1(terminated(one_rule_entry, alt((line_ending, eof))))(input)
}

fn one_rule_entry(input: &str) -> IResult<&str, ParsedRule> {
    map_res(
        separated_pair(rule_indices, tag(": "), rule),
        |(indices, rule)| {
            // this turbofish seems _incredibly_ unnecessary, but rust makes me specify it
            Ok::<ParsedRule, nom::error::Error<nom::error::ErrorKind>>(ParsedRule { indices, rule })
        },
    )(input)
}

fn rule_indices(input: &str) -> IResult<&str, Indices> {
    alt((
        separated_pair(int, tag("-"), int),
        map_res(int, |n: usize| {
            // this turbofish seems _incredibly_ unnecessary, but rust makes me specify it
            Ok::<Indices, nom::error::Error<nom::error::ErrorKind>>((n, n))
        }),
    ))(input)
}

fn int(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| str::parse::<usize>(s))(input)
}

// --------- Rule ---------
pub fn rule(input: &str) -> IResult<&str, Rule> {
    let (remaining, (raw, parts)) =
        consumed(many1(alt((rule_literal, rule_interpolation))))(input)?;

    Ok((
        remaining,
        Rule {
            raw: raw.to_string(),
            parts,
        },
    ))
}

fn rule_literal(input: &str) -> IResult<&str, RuleInst> {
    println!("rule_literal: {}", input);
    map(alt((take_until("{{"), words)), |s: &str| {
        RuleInst::Literal(s.to_string())
    })(input)
}

fn rule_interpolation(input: &str) -> IResult<&str, RuleInst> {
    println!("rule_interpolation: {}", input);
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

fn whitespace(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_whitespace())(input)
}
