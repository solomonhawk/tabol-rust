use nom_supreme::error::GenericErrorTree;
use nom_supreme::final_parser::{Location, RecreateContext};
use rand::distributions::{Uniform, WeightedIndex};
use rand::prelude::*;
use std::error::Error;
use std::{collections::HashMap, fmt};

use crate::nom_parser;

type TableId<'a> = &'a str;

#[derive(Debug)]
pub enum TableError<'a> {
    ParseError(
        &'a str,
        GenericErrorTree<&'static str, &'static str, &'static str, Box<dyn Error + Send + Sync>>,
    ),
    CallError(String),
}

impl<'a> Error for TableError<'a> {}

impl<'a> fmt::Display for TableError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TableError::ParseError(source, e) => {
                writeln!(f, "Table syntax is invalid")?;
                writeln!(f, "-----------------------------")?;

                match e {
                    GenericErrorTree::Base { location, kind } => {
                        // XXX: why do we get only Base sometimes, and why does it contain no information about the problem? "Expected eof"
                        write_base_error(f, source, location, format!("{}", kind).as_ref())?;
                    }
                    GenericErrorTree::Stack { base, contexts } => {
                        // XXX: just grab the "most recent" error right now
                        for context in contexts.iter().take(1) {
                            write_base_error(f, source, context.0, context.1.to_string().as_ref())?;
                            writeln!(f, "-----------------------------")?;
                        }
                    }
                    _ => (),
                }
            }
            TableError::CallError(msg) => {
                write!(f, "invalid table call: {}", msg)?;
            }
        }

        Ok(())
    }
}

fn write_base_error(
    f: &mut fmt::Formatter,
    source: &str,
    location: &str,
    msg: &str,
) -> fmt::Result {
    let Location { line, column } = Location::recreate_context(source, location);

    for (i, l) in contextual_lines(source, line, 3) {
        writeln!(f, "{}", l)?;

        // line is 1-indexed, i is 0-indexed
        let indent = column - 1;
        if i == line - 1 {
            writeln!(f, "{:indent$}^-- {}", "", msg)?;
        }
    }

    Ok(())
}

fn contextual_lines(
    text: &str,
    line: usize,
    n_lines: usize,
) -> impl Iterator<Item = (usize, &str)> {
    let start = line - n_lines;
    let end = line + n_lines;
    let skip = start.max(0);

    text.lines()
        .enumerate()
        .map(|(i, line)| (i, line))
        .skip(skip)
        .take(end - start)
}

#[derive(Debug)]
pub struct Tabol<'a> {
    table_map: HashMap<&'a str, Table<'a>>,
}

impl<'a> Tabol<'a> {
    pub fn new(table_definitions: &'static str) -> Result<Self, TableError<'a>> {
        let mut table_map = HashMap::new();
        let tables = nom_parser::parse_tables(table_definitions)
            .map_err(|e| TableError::ParseError(table_definitions, e))?;

        for table in tables {
            table_map.insert(table.id, table);
        }

        Ok(Self { table_map })
    }

    pub fn table_ids(&self) -> Vec<&str> {
        self.table_map.keys().copied().collect()
    }

    pub fn gen(&self, id: &str) -> Result<String, TableError> {
        if let Some(table) = self.table_map.get(id) {
            return table.gen(self);
        }

        Err(TableError::CallError(format!(
            "No table found with id {}",
            id
        )))
    }

    pub fn gen_many(&self, id: &str, count: usize) -> Result<Vec<String>, TableError> {
        if let Some(table) = self.table_map.get(id) {
            let mut results = Vec::with_capacity(count);

            for _ in 0..count {
                results.push(table.gen(self));
            }

            return results.into_iter().collect();
        }

        Err(TableError::CallError(format!(
            "No table found with id {}",
            id
        )))
    }
}

#[derive(Debug)]
pub struct Table<'a> {
    pub title: &'a str,
    pub id: TableId<'a>,
    pub rules: Vec<Rule<'a>>,
    pub weights: Vec<f32>,
    pub distribution: WeightedIndex<f32>,
}

impl<'a> Table<'a> {
    pub fn new(title: &'a str, id: &'a str, rules: Vec<Rule<'a>>) -> Self {
        let weights: Vec<f32> = rules.iter().map(|rule| rule.weight).collect();

        Self {
            title,
            id,
            rules,
            weights: weights.to_owned(),
            distribution: WeightedIndex::new(&weights).unwrap(),
        }
    }

    pub fn gen(&self, tables: &'a Tabol) -> Result<String, TableError> {
        let mut rng = rand::thread_rng();
        let rule = &self.rules[self.distribution.sample(&mut rng)];

        rule.resolve(tables)
    }
}

#[derive(Debug, Clone)]
pub struct Rule<'a> {
    pub raw: &'a str,
    pub weight: f32,
    pub parts: Vec<RuleInst<'a>>,
}

impl<'a> Rule<'a> {
    pub fn resolve(&self, tables: &'a Tabol) -> Result<String, TableError<'a>> {
        // keep track of context
        // forward pass to resolve all interpolations
        // backwards pass to resolve built-ins (e.g. article)
        let resolved: Result<Vec<String>, TableError<'a>> = self
            .parts
            .iter()
            .map(|part| match part {
                RuleInst::DiceRoll(count, sides) => Ok(roll_dice(*count, *sides).to_string()),
                RuleInst::Literal(str) => Ok(str.to_string()),
                RuleInst::Interpolation(id, opts) => {
                    let mut resolved = tables.gen(id)?;

                    for opt in opts {
                        opt.apply(&mut resolved);
                    }

                    Ok(resolved)
                }
            })
            .collect();

        Ok(resolved?.join(""))
    }
}

#[derive(Debug, Clone)]
pub enum RuleInst<'a> {
    DiceRoll(usize, usize), // (count, sides)
    Literal(&'a str),
    Interpolation(TableId<'a>, Vec<FilterOp>),
}

#[derive(Debug, Clone)]
pub enum FilterOp {
    DefiniteArticle,
    IndefiniteArticle,
    Capitalize,
}

impl FilterOp {
    pub fn apply(&self, value: &mut String) {
        match self {
            FilterOp::DefiniteArticle => {
                value.insert_str(0, "the ");
            }
            FilterOp::IndefiniteArticle
                if value.starts_with('a')
                    || value.starts_with('e')
                    || value.starts_with('i')
                    || value.starts_with('o')
                    || value.starts_with('u') =>
            {
                value.insert_str(0, "an ");
            }
            FilterOp::IndefiniteArticle => {
                value.insert_str(0, "a ");
            }
            FilterOp::Capitalize => {
                let mut chars = value.chars();
                if let Some(first) = chars.next() {
                    *value = format!("{}{}", first.to_uppercase(), chars.as_str());
                }
            }
        }
    }
}

pub fn roll_dice(count: usize, sides: usize) -> usize {
    let mut rng = rand::thread_rng();
    let mut total = 0;

    for _ in 0..count {
        total += rng.sample(Uniform::new(1, sides + 1));
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_dice() {
        for _ in 0..10000 {
            let roll = roll_dice(1, 6);
            assert!(roll >= 1);
            assert!(roll <= 6);
        }

        for _ in 0..10000 {
            let roll = roll_dice(5, 10);
            assert!(roll >= 5);
            assert!(roll <= 50);
        }
    }
}
