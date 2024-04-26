#![feature(lazy_cell)]

mod nom_parser;
mod tabol;

use clap::Parser;
use std::sync::LazyLock;
use std::{error::Error, fs};

use crate::tabol::TableError;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    definition: String,

    #[arg(short, long)]
    table: Option<String>,

    #[arg(short, long, default_value_t = 10)]
    count: u8,

    #[arg(long, default_value_t = false)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    static TABLE_DEF: LazyLock<String> = LazyLock::new(|| {
        let args = Args::parse();
        let file_path = format!("./src/tables/{}.tbl", args.definition);
        println!("filepath: {}", file_path);
        fs::read_to_string(file_path).expect("Should have been able to read the file")
    });

    let args = Args::parse();
    let tabol = tabol::Tabol::new(TABLE_DEF.trim());
    let table_name = args.table.unwrap_or(args.definition);

    if let Ok(tabol) = tabol {
        if !tabol.contains_table(table_name.as_str()) {
            return Err(TableError::CallError(format!(
                "Table definition does not have a table with id \"{}\"",
                table_name
            ))
            .into());
        }

        if args.debug {
            println!("[DEBUG] Table IDs: {:?}", tabol.table_ids());
        }

        if let Ok(results) = tabol.gen_many(table_name.as_str(), args.count) {
            for result in results {
                println!("{}\n", result);
            }
        } else {
            return Err(TableError::CallError(format!(
                "Could not generate \"{}\" x {}!",
                table_name, args.count
            ))
            .into());
        }
    } else {
        return Err(tabol.unwrap_err().into());
    }

    Ok(())
}
