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
        let available = self.available + tranx.amount;
        let total = self.total + tranx.amount;
        self.available = round_up(available);
        self.total = round_up(total);
    }

    pub fn withdraw(&mut self, tranx: &Transaction) {
        if tranx.amount <= self.available {
            let available = self.available - tranx.amount;
            let total = self.total - tranx.amount;
            self.available = round_up(available);
            self.total = round_up(total);
        }
    }

    pub fn dispute(&mut self, tranx: &Transaction) {
        if tranx.amount <= self.available {
            let available = self.available - tranx.amount;
            let held = self.held + tranx.amount;

            self.available = round_up(available);
            self.held = round_up(held);
        }
    }

    pub fn resolve(&mut self, tranx: &Transaction) {
        if tranx.amount <= self.held {
            let available = self.available + tranx.amount;
            let held = self.held - tranx.amount;

            self.available = round_up(available);
            self.held = round_up(held);
        }
    }

    pub fn chargeback(&mut self, tranx: &Transaction) {
        if tranx.amount <= self.held {
            let held = self.held - tranx.amount;
            let total = self.total - tranx.amount;

            self.held = round_up(held);
            self.total = round_up(total);
            self.locked = true;
        }
    }
}

fn round_up(value: f32) -> f32 {
    (value * 10000.0).floor() / 10000.0
}

pub fn print() {
    let mut csv_writer = csv::Writer::from_writer(io::stdout());
    storage::ACCOUNTS.lock().unwrap().reads(|iter| {
        for (_, acc) in iter {
            csv_writer.serialize(acc).unwrap();
            // println!("{:?}", acc)
        }
    })
}

pub fn process_deposit(tranx: &Transaction) {
    if tranx.r#type != "deposit" {
        return;
    }

    let tx_exists: bool;
    tx_exists = TxStore::TRANSACTIONS.lock().unwrap().exists(tranx.tx);

    // handle duplicates
    if tx_exists {
        return;
    }

    let acct = get_account(tranx.client);
    // ignore if account is frozen
    if acct.locked {
        return;
    }

    let u_account: Option<Account> =
        storage::ACCOUNTS
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
        save_transaction(tranx);
    }
}

pub fn process_withdrawal(tranx: &Transaction) {
    if tranx.r#type != "withdrawal" {
        return;
    }

    let tx_exists: bool;

    tx_exists = TxStore::TRANSACTIONS.lock().unwrap().exists(tranx.tx);

    // handle duplicates
    if tx_exists {
        return;
    }

    let acct = get_account(tranx.client);

    // ignore if account is frozen
    if acct.locked {
        return;
    }

    let u_account: Option<Account> =
        storage::ACCOUNTS
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
        save_transaction(tranx);
    }
}

pub fn process_dispute(tranx: &Transaction) {
    if tranx.r#type != "dispute" {
        return;
    }

    let tx_exists: bool = TxStore::TRANSACTIONS.lock().unwrap().exists(tranx.tx);
    let dispute_exists: bool = TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .dispute_exists(tranx.tx);

    if !tx_exists || dispute_exists {
        return;
    }

    let stored_tranx: Transaction = TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .read(tranx.tx, |trx| trx.unwrap().clone());

    let acct = get_account(tranx.client);

    // ignore if account is frozen
    if acct.locked {
        return;
    }

    let u_account: Option<Account> =
        storage::ACCOUNTS
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
        let dispute = Dispute::new(tranx.client, tranx.tx, false);

        storage::ACCOUNTS.lock().unwrap().insert(acct);
        // Store dispute
        TxStore::TRANSACTIONS
            .lock()
            .unwrap()
            .insert_dispute(dispute);
    }
}

pub fn process_resolve(tranx: &Transaction) {
    if tranx.r#type != "resolve" {
        return;
    }

    let tx_exists: bool;
    let dispute_exists: bool;

    tx_exists = TxStore::TRANSACTIONS.lock().unwrap().exists(tranx.tx);
    dispute_exists = TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .dispute_exists(tranx.tx);

    if !tx_exists || !dispute_exists {
        return;
    }

    let stored_tranx: Transaction = TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .read(tranx.tx, |trx| trx.unwrap().clone());

    let stored_dispute: Dispute = TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .dispute(tranx.tx, |trx| trx.unwrap().clone());

    // ignore if dispute has been resolved
    if stored_dispute.resolved {
        return;
    }

    let acct = get_account(tranx.client);

    // ignore if account is frozen
    if acct.locked {
        return;
    }

    let u_account: Option<Account> =
        storage::ACCOUNTS
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
        resolve_dispute(stored_dispute.tx);
    }
}

pub fn process_chargeback(tranx: &Transaction) {
    if tranx.r#type != "chargeback" {
        return;
    }

    let tx_exists: bool = TxStore::TRANSACTIONS.lock().unwrap().exists(tranx.tx);
    let dispute_exists = TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .dispute_exists(tranx.tx);

    if !tx_exists || !dispute_exists {
        return;
    }

    let stored_tranx: Transaction = TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .read(tranx.tx, |trx| trx.unwrap().clone());

    let stored_dispute: Dispute = TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .dispute(tranx.tx, |trx| trx.unwrap().clone());

    // ignore if dispute has been resolved
    if stored_dispute.resolved {
        return;
    }

    let acct = get_account(tranx.client);

    // ignore if account is frozen
    if acct.locked {
        return;
    }

    let u_account: Option<Account> =
        storage::ACCOUNTS
            .lock()
            .unwrap()
            .modify(tranx.client, |acct| {
                let a = if let Some(acc) = acct {
                    acc.chargeback(&stored_tranx);
                    Some(*acc)
                } else {
                    None
                };
                return a;
            });

    if let Some(acct) = u_account {
        storage::ACCOUNTS.lock().unwrap().insert(acct);
        resolve_dispute(stored_dispute.tx);
    }
}

fn get_account(client: u16) -> Account {
    let account_exists: bool;
    account_exists = storage::ACCOUNTS.lock().unwrap().exists(client);

    if !account_exists {
        let new_account = Account::new(client, 0.0, 0.0);
        storage::ACCOUNTS.lock().unwrap().insert(new_account);
    }

    storage::ACCOUNTS
        .lock()
        .unwrap()
        .read(client, |acct| acct.unwrap().clone())
}

fn save_transaction(tranx: &Transaction) {
    let new_tranx = Transaction::new(
        tranx.r#type.to_string(),
        tranx.client,
        tranx.tx,
        tranx.amount,
    );

    TxStore::TRANSACTIONS.lock().unwrap().insert(new_tranx);
}

fn resolve_dispute(tx: u32) {
    let updated_dispute = TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .modify_dispute(tx, |dis| {
            let disp = dis.unwrap();
            disp.resolved = true;

            return *disp;
        });

    TxStore::TRANSACTIONS
        .lock()
        .unwrap()
        .insert_dispute(updated_dispute);
}
