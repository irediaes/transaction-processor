extern crate csv;

mod ac;
pub mod storage;
mod tx;
use async_stream::stream;
use futures_util::{pin_mut, StreamExt};

use crate::ac::account;
use crate::tx::transaction::Transaction;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

#[tokio::main]
async fn main() {
    if let Err(err) = read_csv_file().await {
        println!("{}", err);
        process::exit(1);
    }
}

async fn read_csv_file() -> Result<(), Box<dyn Error>> {
    let file_path = read_arg()?;
    let file = File::open(file_path)?;

    let tx_stream = stream! {
        let mut rdr = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(file);

        for result in rdr.deserialize() {
            let record: Transaction = result.unwrap();
            yield record;
        }
    };

    pin_mut!(tx_stream);

    while let Some(record) = tx_stream.next().await {
        // println!("{:?}", record);
        account::process_deposit(&record);
        account::process_withdrawal(&record);
        account::process_dispute(&record);
        account::process_resolve(&record);
        account::process_chargeback(&record);
    }

    account::export();

    Ok(())
}

fn read_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected a csv file argument")),
        Some(file_path) => Ok(file_path),
    }
}
