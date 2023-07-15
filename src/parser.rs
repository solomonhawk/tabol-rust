use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{alphanumeric1, digit1, line_ending, not_line_ending},
    combinator::{all_consuming, consumed, eof, map, map_parser, map_res},
    error::make_error,
    multi::{fold_many1, many0, many1, many_till},
    number::complete::float,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::collections::HashMap;

use crate::tabol::{FilterOp, Rule, RuleInst, Table};

// --------- Tabol ---------
pub fn parse_tables<'a>(input: &'a str) -> IResult<&'a str, Vec<Table>> {
    all_consuming(many1(table))(input)
}

/**
 * --------- Table ---------
 *
 *   ┌───────────────────┐
 *   │    Frontmatter    │
 *   ├───────────────────┤
 *   │                   │
 *   │       Rules       │
 *   │                   │
 *   └───────────────────┘
 *
 */
fn table<'a>(input: &'a str) -> IResult<&'a str, Table<'a>> {
    let (input, (frontmatter, rules, _)) = tuple((frontmatter, rules, whitespace))(input)?;
    let weights = rules.iter().map(|rule| rule.weight).collect::<Vec<_>>();

    Ok((
        input,
        Table::new(frontmatter.title, frontmatter.id, rules, weights),
    ))
}

struct Frontmatter<'a> {
    pub title: &'a str,
    pub id: &'a str,
}

fn frontmatter(input: &str) -> IResult<&str, Frontmatter> {
    let (input, attrs) = delimited(
        pair(tag("---"), line_ending),
        fold_many1(
            frontmatter_attr,
            HashMap::new,
            |mut acc: HashMap<_, _>, (k, v)| {
                acc.insert(k, v);
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

    Ok((input, Frontmatter { id, title }))
}

fn frontmatter_attr(input: &str) -> IResult<&str, (&str, &str)> {
    terminated(
        separated_pair(alphanumeric1, tag(": "), not_line_ending),
        line_ending,
    )(input)
}

// --------- Rules ---------
fn rules(input: &str) -> IResult<&str, Vec<Rule>> {
    many1(terminated(
        map_parser(not_line_ending, one_rule_entry),
        alt((eof, line_ending)),
    ))(input)
}

fn one_rule_entry(input: &str) -> IResult<&str, Rule> {
    map_res(
        // maybe don't allow both : and .? it got annoying while testing
        separated_pair(float, alt((tag(". "), tag(": "))), rule),
        |(weight, (raw, parts))| {
            // this turbofish seems _incredibly_ unnecessary, but rust makes me specify it
            Ok::<Rule, nom::error::Error<nom::error::ErrorKind>>(Rule { raw, weight, parts })
        },
    )(input)
}

// --------- Rule ---------
pub fn rule(input: &str) -> IResult<&str, (&str, Vec<RuleInst>)> {
    let (input, (raw, (parts, _))) = consumed(many_till(
        alt((rule_dice_roll, rule_interpolation, rule_literal)),
        eof,
    ))(input)?;

    Ok((input, (raw, parts)))
}

fn rule_dice_roll(input: &str) -> IResult<&str, RuleInst> {
    map(
        delimited(
            tag("{{"),
            // should throw error if no sides
            alt((
                tuple((
                    map_res(digit1, str::parse),
                    preceded(tag("d"), map_res(digit1, str::parse)),
                )),
                map(
                    tuple((tag("d"), map_res(digit1, str::parse))),
                    |(_, sides)| (1, sides),
                ),
            )),
            tag("}}"),
        ),
        |(count, sides)| RuleInst::DiceRoll(count, sides),
    )(input)
}

fn rule_literal(input: &str) -> IResult<&str, RuleInst> {
    map(alt((take_until("{{"), not_line_ending)), |s: &str| {
        RuleInst::Literal(s)
    })(input)
}

fn rule_interpolation(input: &str) -> IResult<&str, RuleInst> {
    map(delimited(tag("{{"), pipeline, tag("}}")), |(s, filters)| {
        RuleInst::Interpolation(s, filters)
    })(input)
}

fn pipeline(input: &str) -> IResult<&str, (&str, Vec<FilterOp>)> {
    pair(
        ident,
        map(many0(preceded(tag("|"), ident)), |filters: Vec<&str>| {
            filters
                .iter()
                .map(|&filter| match filter {
                    "definite" => FilterOp::DefiniteArticle,
                    "indefinite" => FilterOp::IndefiniteArticle,
                    "capitalize" => FilterOp::Capitalize,
                    // better way to return error from `map` parser?
                    _ => panic!("unknown filter: {}", filter),
                })
                .collect::<Vec<_>>()
        }),
    )(input)
}

fn ident(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn whitespace(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_whitespace())(input)
}
