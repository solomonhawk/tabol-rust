use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::error::Error;
use std::{collections::HashMap, fmt};

use crate::parser;

type TableId<'a> = &'a str;

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
pub struct Tabol<'a> {
    table_map: HashMap<&'a str, Table<'a>>,
}

impl<'a> Tabol<'a> {
    pub fn new(table_definitions: &'a str) -> Result<Self, TableError> {
        let mut table_map = HashMap::new();
        let (_, tables) = parser::parse_tables(table_definitions).map_err(|e| {
            // TODO: better error handling, convert nom errors to TableError
            // nom has some pretty bad errors, maybe use nom-supreme?
            TableError::ParseError(format!("failed to parse table definitions: {}", e))
        })?;

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
pub struct Table<'a> {
    pub title: &'a str,
    pub id: TableId<'a>,
    pub rules: Vec<Rule<'a>>,
    pub weights: Vec<f32>,
    pub distribution: WeightedIndex<f32>,
}

impl<'a> Table<'a> {
    pub fn new(title: &'a str, id: &'a str, rules: Vec<Rule<'a>>, weights: Vec<f32>) -> Self {
        Self {
            title,
            id,
            rules,
            weights: weights.to_owned(),
            distribution: WeightedIndex::new(&weights).unwrap(),
        }
    }

    pub fn gen(&self, tables: &Tabol) -> Result<String, TableError> {
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

impl Rule<'_> {
    pub fn resolve(&self, tables: &Tabol) -> Result<String, TableError> {
        // keep track of context
        // forward pass to resolve all interpolations
        // backwards pass to resolve built-ins (e.g. article)
        let resolved: Result<Vec<String>, TableError> = self
            .parts
            .iter()
            .map(|part| match part {
                RuleInst::Literal(str) => Ok(str.to_string()),
                RuleInst::Interpolation(id) => tables.gen(&id),
            })
            .collect();

        Ok(resolved?.join(""))
    }
}

#[derive(Debug, Clone)]
pub enum RuleInst<'a> {
    Literal(&'a str),
    Interpolation(TableId<'a>), // parameters?
}