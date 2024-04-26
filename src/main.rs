#![feature(lazy_cell)]

extern crate pretty_env_logger;

#[macro_use]
extern crate log;

mod nom_parser;
mod tabol;

use clap::Parser;
use std::sync::LazyLock;
use std::{error::Error, fs};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    definition: String,

    #[arg(short, long)]
    table: Option<String>,

    #[arg(short, long, default_value_t = 10)]
    count: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    static TABLE_DEF: LazyLock<String> = LazyLock::new(|| {
        let args = Args::parse();
        let file_path = format!("./src/tables/{}.tbl", args.definition);

        debug!("Filepath: \"{file_path}\"");

        fs::read_to_string(file_path).expect("Should have been able to read the file")
    });

    let args = Args::parse();
    let tabol = tabol::Tabol::new(TABLE_DEF.trim())?;
    let table_name = args.table.unwrap_or(args.definition);

    debug!("Table IDs: {:?}", tabol.table_ids());

    let results = tabol.gen_many(table_name.as_str(), args.count)?;

    for result in results {
        println!("{}\n", result);
    }

    Ok(())
}
