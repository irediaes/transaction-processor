extern crate csv;

mod account;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;
use std::{env, sync::Mutex};

use crate::account::{account::Account, storage::AccountStorage};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    r#type: String,
    client: u16,
    tx: u32,
    amount: f32,
}

pub static mut ACCOUNTS: Lazy<Mutex<AccountStorage>> =
    Lazy::new(|| Mutex::new(AccountStorage::new()));

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
        process_deposit(record);
    }

    unsafe {
        ACCOUNTS.lock().unwrap().read_accounts(|iter| {
            for ac in iter {
                println!("{:?}", ac)
            }
        })
    }

    Ok(())
}

fn read_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected a csv file argument")),
        Some(file_path) => Ok(file_path),
    }
}

fn process_deposit(tranx: Transaction) {
    if tranx.r#type != "deposit" {
        return;
    }

    let account_exists: bool;

    unsafe {
        account_exists = ACCOUNTS.lock().unwrap().account_exists(tranx.client);
    }

    if !account_exists {
        let new_account = Account::new(tranx.client, tranx.amount, 0.0);
        unsafe {
            ACCOUNTS.lock().unwrap().insert_account(new_account);
        }

        return;
    }
    let u_account: Option<Account>;

    unsafe {
        u_account = ACCOUNTS
            .lock()
            .unwrap()
            .modify_account(tranx.client, |acct| {
                let a = if let Some(acc) = acct {
                    acc.deposit(tranx);
                    Some(*acc)
                } else {
                    None
                };
                return a;
            });

        if let Some(acct) = u_account {
            ACCOUNTS.lock().unwrap().insert_account(acct);
        }
    }
}

#[test]
fn test_process_deposit() {
    let tranx_1 = Transaction {
        r#type: "deposit".to_string(),
        client: 1,
        tx: 1,
        amount: 10.0,
    };

    let tranx_2 = Transaction {
        r#type: "deposit".to_string(),
        client: 1,
        tx: 1,
        amount: 15.0,
    };

    process_deposit(tranx_1.clone());

    unsafe {
        let acct = ACCOUNTS
            .lock()
            .unwrap()
            .read_account(tranx_1.client, |acc| acc.unwrap().clone());

        assert!(
            acct.available == tranx_1.amount,
            "invalid available funds; expected {}, got {}",
            tranx_1.amount,
            acct.available
        );
    }

    process_deposit(tranx_2.clone());

    unsafe {
        let acct = ACCOUNTS
            .lock()
            .unwrap()
            .read_account(tranx_2.client, |acc| acc.unwrap().clone());

        assert!(
            acct.available == (tranx_1.amount + tranx_2.amount),
            "invalid available funds; expected {}, got {}",
            tranx_1.amount,
            acct.available
        );
    }
}
