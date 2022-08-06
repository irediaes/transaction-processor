extern crate csv;

use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    r#type: String,
    client: u16,
    tx: u32,
    amount: f32,
}

fn main() {
    if let Err(err) = read_csv_file() {
        println!("{}", err);
        process::exit(1);
    }
}

fn read_csv_file() -> Result<(), Box<dyn Error>> {
    let file_path = read_arg()?;
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);

    for result in rdr.deserialize() {
        let record: Transaction = result?;
        println!("{:?}", record);
    }

    Ok(())
}

fn read_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected a csv file argument")),
        Some(file_path) => Ok(file_path),
    }
}
