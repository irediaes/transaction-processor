use std::io;
extern crate csv;

use crate::acct::storage;
use crate::tx::transaction::Dispute;
use crate::tx::{storage as TxStore, transaction::Transaction};
use serde::Serialize;

#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub struct Account {
    pub client: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl Account {
    pub fn new(client: u16, available: f32, held: f32) -> Self {
        let total = available + held;
        let locked = false;

        Self {
            client,
            available,
            held,
            total,
            locked,
        }
    }

    pub fn deposit(&mut self, tranx: &Transaction) {
        self.available += tranx.amount;
        self.total += tranx.amount;
    }

    pub fn withdraw(&mut self, tranx: &Transaction) {
        if tranx.amount <= self.available {
            self.available -= tranx.amount;
            self.total -= tranx.amount;
        }
    }

    pub fn dispute(&mut self, tranx: &Transaction) {
        if tranx.amount <= self.available {
            self.available -= tranx.amount;
            self.held += tranx.amount;
        }
    }

    pub fn resolve(&mut self, tranx: &Transaction) {
        if tranx.amount <= self.held {
            self.available += tranx.amount;
            self.held -= tranx.amount;
        }
    }
}

pub fn print() {
    let mut csv_writer = csv::Writer::from_writer(io::stdout());
    unsafe {
        storage::ACCOUNTS.lock().unwrap().reads(|iter| {
            for (_, acc) in iter {
                csv_writer.serialize(acc).unwrap();
                println!("{:?}", acc)
            }
        })
    }
}

pub fn process_deposit(tranx: &Transaction) {
    if tranx.r#type != "deposit" {
        return;
    }

    insert_transaction(tranx);

    let account_exists: bool;

    unsafe {
        account_exists = storage::ACCOUNTS.lock().unwrap().exists(tranx.client);
    }

    if !account_exists {
        let new_account = Account::new(tranx.client, tranx.amount, 0.0);
        unsafe {
            storage::ACCOUNTS.lock().unwrap().insert(new_account);
        }

        return;
    }
    let u_account: Option<Account>;

    unsafe {
        u_account = storage::ACCOUNTS
            .lock()
            .unwrap()
            .modify(tranx.client, |acct| {
                let a = if let Some(acc) = acct {
                    acc.deposit(tranx);
                    Some(*acc)
                } else {
                    None
                };
                return a;
            });

        if let Some(acct) = u_account {
            storage::ACCOUNTS.lock().unwrap().insert(acct);
        }
    }
}

pub fn process_withdrawal(tranx: &Transaction) {
    if tranx.r#type != "withdrawal" {
        return;
    }

    insert_transaction(tranx);

    let account_exists: bool;

    unsafe {
        account_exists = storage::ACCOUNTS.lock().unwrap().exists(tranx.client);
    }

    if !account_exists {
        let new_account = Account::new(tranx.client, 0.0, 0.0);
        unsafe {
            storage::ACCOUNTS.lock().unwrap().insert(new_account);
        }

        return;
    }
    let u_account: Option<Account>;

    unsafe {
        u_account = storage::ACCOUNTS
            .lock()
            .unwrap()
            .modify(tranx.client, |acct| {
                let a = if let Some(acc) = acct {
                    acc.withdraw(tranx);
                    Some(*acc)
                } else {
                    None
                };
                return a;
            });

        if let Some(acct) = u_account {
            storage::ACCOUNTS.lock().unwrap().insert(acct);
        }
    }
}

pub fn process_dispute(tranx: &Transaction) {
    if tranx.r#type != "dispute" {
        return;
    }

    let tx_exists: bool;

    unsafe {
        tx_exists = TxStore::TRANSACTIONS.lock().unwrap().exists(tranx.tx);
    }

    if !tx_exists {
        return;
    }

    let stored_tranx: Transaction;

    unsafe {
        stored_tranx = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .read(tranx.tx, |trx| trx.unwrap().clone());

        let dispute = Dispute::new(tranx.client, tranx.tx, false);

        // Store dispute
        TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .insert_dispute(dispute);
    }

    let account_exists: bool;

    unsafe {
        account_exists = storage::ACCOUNTS.lock().unwrap().exists(tranx.client);
    }

    if !account_exists {
        let new_account = Account::new(tranx.client, 0.0, 0.0);
        unsafe {
            storage::ACCOUNTS.lock().unwrap().insert(new_account);
        }
    }

    let u_account: Option<Account>;

    unsafe {
        u_account = storage::ACCOUNTS
            .lock()
            .unwrap()
            .modify(tranx.client, |acct| {
                let a = if let Some(acc) = acct {
                    acc.dispute(&stored_tranx);
                    Some(*acc)
                } else {
                    None
                };
                return a;
            });

        if let Some(acct) = u_account {
            storage::ACCOUNTS.lock().unwrap().insert(acct);
        }
    }
}

pub fn process_resolve(tranx: &Transaction) {
    if tranx.r#type != "resolve" {
        return;
    }

    let tx_exists: bool;
    let dispute_exists: bool;

    unsafe {
        tx_exists = TxStore::TRANSACTIONS.lock().unwrap().exists(tranx.tx);
        dispute_exists = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .dispute_exists(tranx.tx);
    }

    if !tx_exists || !dispute_exists {
        return;
    }

    let stored_tranx: Transaction;
    let stored_dispute: Dispute;

    unsafe {
        stored_tranx = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .read(tranx.tx, |trx| trx.unwrap().clone());

        stored_dispute = TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .dispute(tranx.tx, |trx| trx.unwrap().clone());
    }

    if stored_dispute.resolved {
        return;
    }

    let account_exists: bool;

    unsafe {
        account_exists = storage::ACCOUNTS.lock().unwrap().exists(tranx.client);
    }

    if !account_exists {
        let new_account = Account::new(tranx.client, 0.0, 0.0);
        unsafe {
            storage::ACCOUNTS.lock().unwrap().insert(new_account);
        }
    }

    let u_account: Option<Account>;

    unsafe {
        let updated_dispute =
            TxStore::TRANSACTIONS
                .lock()
                .unwrap()
                .modify_dispute(stored_dispute.tx, |dis| {
                    let disp = dis.unwrap();
                    disp.resolved = true;

                    return *disp;
                });

        u_account = storage::ACCOUNTS
            .lock()
            .unwrap()
            .modify(tranx.client, |acct| {
                let a = if let Some(acc) = acct {
                    acc.resolve(&stored_tranx);
                    Some(*acc)
                } else {
                    None
                };
                return a;
            });

        if let Some(acct) = u_account {
            storage::ACCOUNTS.lock().unwrap().insert(acct);
            TxStore::TRANSACTIONS
                .lock()
                .unwrap()
                .insert_dispute(updated_dispute);
        }
    }
}

fn insert_transaction(tranx: &Transaction) {
    let tx_exists: bool;

    unsafe {
        tx_exists = TxStore::TRANSACTIONS.lock().unwrap().exists(tranx.tx);
    }

    if !tx_exists {
        let new_tranx = Transaction::new(
            tranx.r#type.to_string(),
            tranx.client,
            tranx.tx,
            tranx.amount,
        );
        unsafe {
            TxStore::TRANSACTIONS.lock().unwrap().insert(new_tranx);
        }
    }
}
