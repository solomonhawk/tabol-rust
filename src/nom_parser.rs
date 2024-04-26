use nom::{
    bytes::complete::{take_until, take_while1},
    character::complete::{alphanumeric1, digit1, line_ending, multispace0, not_line_ending},
    combinator::map_parser,
    error::make_error,
    multi::{fold_many1, many0, many1, separated_list1},
    number::complete::float,
    sequence::{pair, separated_pair, tuple},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, tag::complete::tag};
use nom_supreme::{final_parser::final_parser, parser_ext::ParserExt};
use std::collections::HashMap;

use crate::tabol::{FilterOp, Rule, RuleInst, TableDefinition};

// --------- Tabol ---------
pub fn parse_tables(input: &str) -> Result<Vec<TableDefinition>, ErrorTree<&str>> {
    final_parser(many1(table))(input)
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
fn table(input: &str) -> IResult<&str, TableDefinition<'_>, ErrorTree<&str>> {
    tuple((frontmatter, rules))
        .context("Invalid table definition")
        .map(|(frontmatter, rules)| TableDefinition::new(frontmatter.title, frontmatter.id, rules))
        .parse(input)
}

struct Frontmatter<'a> {
    pub title: &'a str,
    pub id: &'a str,
}

fn frontmatter(input: &str) -> IResult<&str, Frontmatter, ErrorTree<&str>> {
    let (input, attrs) = fold_many1(
        frontmatter_attr,
        HashMap::new,
        |mut acc: HashMap<_, _>, (k, v)| {
            acc.insert(k, v);
            acc
        },
    )
    .delimited_by(
        pair(tag("---"), line_ending).context("Table attributes should be enclosed in `---`"),
    )
    .context("Invalid table attributes")
    .parse(input)?;

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

fn frontmatter_attr(input: &str) -> IResult<&str, (&str, &str), ErrorTree<&str>> {
    separated_pair(
        alphanumeric1.context("Table attributes can only contain alphanumeric characters"),
        tag(": ").context("Missing table attribute separator, expected `:`"),
        not_line_ending, // XXX: happily matches nothing ("")
    )
    .context("Table attributes should be formatted like `name: value`")
    .terminated(line_ending)
    .parse(input)
}

// --------- Rules ---------
fn rules(input: &str) -> IResult<&str, Vec<Rule>, ErrorTree<&str>> {
    separated_list1(line_ending, rule_line)
        .terminated(multispace0)
        .parse(input)
}

fn rule_line(input: &str) -> IResult<&str, Rule, ErrorTree<&str>> {
    // the `map_parser(not_line_ending, rule_line)` is important, so that
    // `rule_line` doesn't parse past '\n' at the end of the current line
    map_parser(
        not_line_ending,
        separated_pair(
            float.context("Invalid rule weight, expected an integer or float"),
            tag(": ").context("Missing rule separator, expected `:`"),
            rule,
        )
        .context("Rule should start with a weight, followed by a `:` and then the rule text")
        .map(|(weight, (raw, parts))| Rule { raw, weight, parts }),
    )
    .parse(input)
}

// --------- Rule ---------
pub fn rule(input: &str) -> IResult<&str, (&str, Vec<RuleInst>), ErrorTree<&str>> {
    many1(rule_dice_roll.or(rule_interpolation).or(rule_literal))
        .context("Invalid rule text, expected a dice roll (`2d4`), an interpolation (`{{other}}`) or a literal")
        .with_recognized()
        .parse(input)
}

fn rule_dice_roll(input: &str) -> IResult<&str, RuleInst, ErrorTree<&str>> {
    tuple((
        digit1.parse_from_str(),
        digit1.parse_from_str().preceded_by(tag("d")),
    ))
    .or(digit1
        .parse_from_str()
        .preceded_by(tag("d"))
        .map(|sides| (1, sides)))
    .preceded_by(tag("{{"))
    .terminated(tag("}}"))
    .map(|(count, sides)| RuleInst::DiceRoll(count, sides))
    .parse(input)
}

fn rule_literal(input: &str) -> IResult<&str, RuleInst, ErrorTree<&str>> {
    // can't just do `take_until("{{").or(not_line_ending)` or else we'll
    // successfully parse "" which causes many1 to fail
    map_parser(take_until("{{").or(not_line_ending), literal)
        .context("rule literal")
        .map(RuleInst::Literal)
        .parse(input)
}

fn rule_interpolation(input: &str) -> IResult<&str, RuleInst, ErrorTree<&str>> {
    pipeline
        .preceded_by(tag("{{"))
        .terminated(tag("}}"))
        .context("rule interpolation")
        .map(|(s, filters)| RuleInst::Interpolation(s, filters))
        .parse(input)
}

fn pipeline(input: &str) -> IResult<&str, (&str, Vec<FilterOp>), ErrorTree<&str>> {
    pair(ident.cut(), filters)
        .context("interpolation pipeline")
        .parse(input)
}

fn filters(input: &str) -> IResult<&str, Vec<FilterOp>, ErrorTree<&str>> {
    many0(ident.preceded_by(tag("|")))
        .map(|filters| {
            filters
                .iter()
                .map(|&filter| match filter {
                    "definite" => FilterOp::DefiniteArticle,
                    "indefinite" => FilterOp::IndefiniteArticle,
                    "capitalize" => FilterOp::Capitalize,
                    // better way to return error from `map` parser?
                    _ => panic!("unknown filter: {}", filter),
                })
                .collect()
        })
        .parse(input)
}

fn literal(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    take_while1(|c: char| {
        c.is_alphanumeric() || c == '_' || c == '-' || c.is_whitespace() || c.is_ascii_punctuation()
    })
    .context("literal")
    .parse(input)
}

fn ident(input: &str) -> IResult<&str, &str, ErrorTree<&str>> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')
        .context("Invalid identifier, only alphanumeric characters and `_` are allowed")
        .parse(input)
}
