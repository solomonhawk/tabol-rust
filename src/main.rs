mod parser;

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::error::Error;
use std::{collections::HashMap, fmt};

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
    pub title: String,
    pub id: TableId,
    pub rules: Vec<Rule>,
    pub weights: Vec<f32>,
    pub distribution: WeightedIndex<f32>,
}

impl Table {
    pub fn new(title: String, id: TableId, rules: Vec<Rule>, weights: Vec<f32>) -> Self {
        Self {
            title,
            id,
            rules,
            weights: weights.clone(),
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
pub struct Rule {
    pub raw: String,
    pub weight: f32,
    pub parts: Vec<RuleInst>,
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
    let table_def = include_str!("potion.tbl");

    match Tabol::new(table_def.trim()) {
        Err(error) => return Err(Box::new(error)),
        Ok(tabol) => {
            println!("{:#?}", tabol);
            println!("table ids: {:?}", tabol.table_ids());
            println!("{:#?}", tabol.gen_many("potion", 20));
        }
    };

    Ok(())
}
