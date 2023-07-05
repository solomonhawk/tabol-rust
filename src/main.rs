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
mod parser;

use rand::prelude::*;
use std::error::Error;
use std::{collections::HashMap, fmt, vec};

type TableId = String;

#[derive(Debug, Clone)]
pub enum TableError {
    ParseError(String),
    CallError(String),
}

impl Error for TableError {}

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

#[derive(Debug)]
pub struct Tabol {
    table_map: HashMap<String, Table>,
}

impl Tabol {
    pub fn new(table_definitions: &str) -> Result<Self, TableError> {
        let mut table_map = HashMap::new();
        let (_, tables) = parser::parse_tables(table_definitions).map_err(|e| {
            // TODO: better error handling, convert nom errors to TableError
            // nom has some pretty bad errors, maybe use nom-supreme?
            TableError::ParseError(format!("failed to parse table definitions: {}", e))
        })?;

        for table in tables {
            table_map.insert(table.id.clone(), table);
        }

        Ok(Self { table_map })
    }

    pub fn table_ids(&self) -> Vec<&str> {
        self.table_map.keys().map(|s| s.as_str()).collect()
    }

    pub fn gen(&self, id: &str) -> Result<String, TableError> {
        if let Some(table) = self.table_map.get(id) {
            return table.gen(&self);
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

#[derive(Debug)]
pub struct Table {
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

#[derive(Debug, Clone)]
pub struct Rule {
    raw: String,
    parts: Vec<RuleInst>,
}

impl Rule {
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

        Ok(resolved?.join(""))
    }
}

#[derive(Debug, Clone)]
pub enum RuleInst {
    Literal(String),
    Interpolation(TableId), // parameters?
}

fn main() -> Result<(), Box<dyn Error>> {
    let table_def = include_str!("example.tbl");
    // let table_defs = vec![table_def.to_string()];

    let tabol = match Tabol::new(table_def.trim()) {
        Err(error) => return Err(Box::new(error)),
        Ok(tabol) => {
            println!("{:#?}", tabol);
            println!("Tabol Ids: {:#?}", tabol.table_ids());
            // println!("{:#?}", tabol.gen("class"));
            // println!("{:#?}", tabol.gen("race"));
            // println!("{:#?}", tabol.gen("alignment"));
        }
    };

    // match tabol.gen("color") {
    //     Ok(color) => println!("{}", color),
    //     Err(error) => return Err(error.to_string()),
    // }

    // match tabol.gen_many("color", 10) {
    //     Ok(colors) => println!("{:?}", colors),
    //     Err(error) => return Err(error.to_string()),
    // }

    // let (_, result) = parser::parse_tables(table_def)?;

    // println!("{:?}", parser::parse_one_rule("2-4: lol")?);
    Ok(())
}
