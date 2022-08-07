use std::io;
extern crate csv;

use crate::account::storage;
use crate::Transaction;
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
        if tranx.amount < self.available {
            self.available -= tranx.amount;
            self.total -= tranx.amount;
        }
    }
}

pub fn print_accounts() {
    let mut csv_writer = csv::Writer::from_writer(io::stdout());
    unsafe {
        storage::ACCOUNTS.lock().unwrap().read_accounts(|iter| {
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

    let account_exists: bool;

    unsafe {
        account_exists = storage::ACCOUNTS
            .lock()
            .unwrap()
            .account_exists(tranx.client);
    }

    if !account_exists {
        let new_account = Account::new(tranx.client, tranx.amount, 0.0);
        unsafe {
            storage::ACCOUNTS
                .lock()
                .unwrap()
                .insert_account(new_account);
        }

        return;
    }
    let u_account: Option<Account>;

    unsafe {
        u_account = storage::ACCOUNTS
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
            storage::ACCOUNTS.lock().unwrap().insert_account(acct);
        }
    }
}

pub fn process_withdrawal(tranx: &Transaction) {
    if tranx.r#type != "withdrawal" {
        return;
    }

    let account_exists: bool;

    unsafe {
        account_exists = storage::ACCOUNTS
            .lock()
            .unwrap()
            .account_exists(tranx.client);
    }

    if !account_exists {
        let new_account = Account::new(tranx.client, 0.0, 0.0);
        unsafe {
            storage::ACCOUNTS
                .lock()
                .unwrap()
                .insert_account(new_account);
        }

        return;
    }
    let u_account: Option<Account>;

    unsafe {
        u_account = storage::ACCOUNTS
            .lock()
            .unwrap()
            .modify_account(tranx.client, |acct| {
                let a = if let Some(acc) = acct {
                    acc.withdraw(tranx);
                    Some(*acc)
                } else {
                    None
                };
                return a;
            });

        if let Some(acct) = u_account {
            storage::ACCOUNTS.lock().unwrap().insert_account(acct);
        }
    }
}

/// Tests

#[test]
fn test_process_deposit() {
    // Clear the account data so it doesn't conflict with other tests
    unsafe {
        storage::ACCOUNTS.lock().unwrap().clear_accounts();
    }

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

    process_deposit(&tranx_1);

    unsafe {
        let acct = storage::ACCOUNTS
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

    process_deposit(&tranx_2);

    unsafe {
        let acct = storage::ACCOUNTS
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

#[test]
fn test_process_withdrawal() {
    // Clear the account data so it doesn't conflict with other tests
    unsafe {
        storage::ACCOUNTS.lock().unwrap().clear_accounts();
    }
    let tranx_1 = Transaction {
        r#type: "withdrawal".to_string(),
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

    process_withdrawal(&tranx_1);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read_account(tranx_1.client, |acc| acc.unwrap().clone());

        assert!(
            acct.available == 0.0,
            "invalid available funds; expected {}, got {}",
            0.0,
            acct.available
        );
    }

    process_deposit(&tranx_2);
    process_withdrawal(&tranx_1);

    unsafe {
        let acct = storage::ACCOUNTS
            .lock()
            .unwrap()
            .read_account(tranx_2.client, |acc| acc.unwrap().clone());
        let amount_diff = tranx_2.amount - tranx_1.amount;
        assert!(
            acct.available == amount_diff,
            "invalid available funds; expected {}, got {}",
            amount_diff,
            acct.available
        );
    }
}

#[test]
fn test_deposit() {
    let mut account = Account::new(1, 20.0, 0.0);
    let tranx = Transaction {
        r#type: "deposit".to_string(),
        client: 1,
        tx: 1,
        amount: 15.0,
    };

    // Test initial funds
    assert!(
        account.available == 20.0,
        "wrong available funds; expect {}, got {}",
        20.0,
        account.available
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    account.deposit(&tranx);

    // Test deposited funds
    assert!(
        account.available == 35.0,
        "wrong available funds; expect {}, got {}",
        35.0,
        account.available
    );

    assert!(
        account.total == 35.0,
        "wrong total funds; expect {}, got {}",
        35.0,
        account.total
    );
}

#[test]
fn test_withdraw() {
    let mut account = Account::new(1, 20.0, 0.0);
    let mut tranx = Transaction {
        r#type: "deposit".to_string(),
        client: 1,
        tx: 1,
        amount: 15.0,
    };

    // Test initial funds
    assert!(
        account.available == 20.0,
        "wrong available funds; expect {}, got {}",
        20.0,
        account.available
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    // Test withdrawing excess funds

    tranx.amount = 50.0;

    account.withdraw(&tranx);

    assert!(
        account.available == 20.0,
        "wrong available funds; expect {}, got {}",
        20.0,
        account.available
    );

    assert!(
        account.total == 20.0,
        "wrong total funds; expect {}, got {}",
        20.0,
        account.total
    );

    // Test withdrawn funds

    tranx.amount = 5.0;
    account.withdraw(&tranx);

    assert!(
        account.available == 15.0,
        "wrong available funds; expect {}, got {}",
        15.0,
        account.available
    );

    assert!(
        account.total == 15.0,
        "wrong total funds; expect {}, got {}",
        15.0,
        account.total
    );
}
