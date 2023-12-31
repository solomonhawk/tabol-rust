mod nom_parser;
mod tabol;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let table_def = include_str!("./tables/potion.tbl");

    let tabol = tabol::Tabol::new(table_def.trim());

    if let Ok(tabol) = tabol {
        println!("table ids: {:?}", tabol.table_ids());
        println!("{:#?}", tabol.gen_many("potion", 10));

        Ok(())
    } else {
        println!("{}", tabol.unwrap_err());
        Ok(())
    }
}
