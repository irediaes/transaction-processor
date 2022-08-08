extern crate csv;

mod acct;
mod tx;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

use crate::acct::account::{self, Account};
use crate::tx::transaction::Transaction;

fn main() {
    if let Err(err) = read_csv_file() {
        println!("{}", err);
        process::exit(1);
    }
}

fn read_csv_file() -> Result<(), Box<dyn Error>> {
    let file_path = read_arg()?;
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(file);

    for result in rdr.deserialize() {
        let record: Transaction = result?;
        // println!("{:?}", record);
        account::process_deposit(&record);
        account::process_withdrawal(&record);
        account::process_dispute(&record);
        account::process_resolve(&record);
        account::process_chargeback(&record);
    }

    account::print();

    Ok(())
}

fn read_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected a csv file argument")),
        Some(file_path) => Ok(file_path),
    }
}
