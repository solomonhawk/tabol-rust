mod parser;
mod tabol;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let table_def = include_str!("potion.tbl");

    let tabol = tabol::Tabol::new(table_def.trim())?;

    println!("{:#?}", tabol);
    println!("table ids: {:?}", tabol.table_ids());
    println!("{:#?}", tabol.gen_many("potion", 10));

    Ok(())
}
