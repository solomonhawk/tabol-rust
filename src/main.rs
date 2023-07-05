// a grammar for defining tables
// a web application for using and editing tables
// a cli for using tables

// parameters
// remapping probabilities?

// build things up starting with 0-parameter tables
// - geography: continents, regions, cities, towns

// const tabol = parse(rules) => Result<Table, ParseError> (throws in wasm)

// tabol.gen('race') => 'Elf'
// tabol.gen('race', 3) => ['Elf', 'Human', 'Dwarf']
// tabol.gen('wrong') => TypeError 'wrong' is not a valid table name

#![allow(unused)]

use rand::prelude::*;
use std::{collections::HashMap, error::Error, fmt};

type TableId = String;

#[derive(Debug, Clone)]
pub enum TableError {
    ParseError(String),
    CallError(String),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TableError::ParseError(msg) => {
                write!(f, "table syntax is invalid: {}", msg)?;
            }
            TableError::CallError(msg) => {
                write!(f, "invalid table call: {}", msg)?;
            }
        }

        Ok(())
    }
}

struct Tabol {
    tables: HashMap<String, Table>,
}

impl Tabol {
    pub fn new(table_definitions: Vec<String>) -> Result<Self, TableError> {
        let mut tables = HashMap::new();

        // temporarily hard-coded since parsing isn't implemented yet
        tables.insert(
            "color".to_string(),
            Table {
                title: "Color".to_string(),
                id: "color".to_string(),
                rules: vec![
                    Rule {
                        raw: "Redish Orange".to_string(),
                        parts: vec![RuleInst::Literal("Redish Orange".to_string())],
                    },
                    Rule {
                        raw: "Greenish Blue".to_string(),
                        parts: vec![RuleInst::Literal("Greenish Blue".to_string())],
                    },
                    Rule {
                        raw: "Purplish Pink".to_string(),
                        parts: vec![RuleInst::Literal("Purplish Pink".to_string())],
                    },
                ],
                choices: vec![0, 1, 2],
            },
        );

        // Err(TableError::ParseError("not implemented".to_string()))
        Ok(Self { tables })
    }

    pub fn gen(&self, id: &str) -> Result<String, TableError> {
        if let Some(table) = self.tables.get(id) {
            return table.gen(&self);
        }

        Err(TableError::CallError(format!(
            "No table found with id {}",
            id
        )))
    }

    pub fn gen_many(&self, id: &str, count: usize) -> Result<Vec<String>, TableError> {
        if let Some(table) = self.tables.get(id) {
            let mut results = Vec::with_capacity(count);

            for _ in 0..count {
                results.push(table.gen(&self));
            }

            return results.into_iter().collect();
        }

        Err(TableError::CallError(format!(
            "No table found with id {}",
            id
        )))
    }
}

struct Table {
    title: String,
    id: TableId,
    rules: Vec<Rule>,
    choices: Vec<usize>, // indices into rules
}

impl Table {
    pub fn gen(&self, tables: &Tabol) -> Result<String, TableError> {
        let mut rng = rand::thread_rng();
        // pick a random number between 0 and len(choices)
        let i = rng.gen_range(0..self.choices.len());
        // use that to select the correct rule
        let rule = &self.rules[self.choices[i]];

        rule.resolve(tables)
    }
}

struct Rule {
    raw: String,
    parts: Vec<RuleInst>,
}

impl Rule {
    pub fn new(&mut self) -> Result<Self, TableError> {
        Err(TableError::ParseError("not implemented".to_string()))
    }

    pub fn resolve(&self, tables: &Tabol) -> Result<String, TableError> {
        let resolved: Result<Vec<String>, TableError> = self
            .parts
            .clone()
            .into_iter()
            .map(|part| match part {
                RuleInst::Literal(str) => Ok(str),
                RuleInst::Interpolation(id) => tables.gen(&id),
            })
            .collect();

        Ok(resolved?.join(" "))
    }
}

#[derive(Clone)]
enum RuleInst {
    Literal(String),
    Interpolation(TableId), // parameters?
}

fn main() -> Result<(), String> {
    let table_def = include_str!("example.tbl");
    let table_defs = vec![table_def.to_string()];

    let tabol = match Tabol::new(table_defs) {
        Ok(tabol) => tabol,
        Err(error) => return Err(error.to_string()),
    };

    match tabol.gen("color") {
        Ok(color) => println!("{}", color),
        Err(error) => return Err(error.to_string()),
    }

    Ok(())
}
