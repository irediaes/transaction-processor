use std::{
    collections::{hash_map::Iter, HashMap},
    sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;

use crate::Account;

pub static ACCOUNTS: Lazy<Mutex<AccountStorage>> = Lazy::new(|| Mutex::new(AccountStorage::new()));

pub struct AccountStorage {
    accounts: Arc<Mutex<HashMap<u16, Account>>>,
}

impl AccountStorage {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn read<F, R>(&self, id: u16, f: F) -> R
    where
        F: FnOnce(Option<&Account>) -> R,
    {
        f(self.accounts.lock().unwrap().get(&id))
    }

    pub fn reads<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Iter<u16, Account>) -> R,
    {
        f(self.accounts.lock().unwrap().iter())
    }

    pub fn insert(&self, account: Account) -> bool {
        let acc = self
            .accounts
            .lock()
            .unwrap()
            .insert(account.client, account);

        if let Some(_acct) = acc {
            return true;
        }

        false
    }

    pub fn modify<F, R>(&self, id: u16, f: F) -> R
    where
        F: FnOnce(Option<&mut Account>) -> R,
    {
        f(self.accounts.lock().unwrap().get_mut(&id))
    }

    pub fn exists(&self, id: u16) -> bool {
        self.accounts.lock().unwrap().contains_key(&id)
    }
}

/// Tests

#[test]
fn test_account_storage_insert() {
    let account = Account {
        client: 1,
        available: 20.0,
        held: 0.0,
        total: 0.0,
        locked: false,
    };

    let db = AccountStorage::new();
    let acct_exist = db.exists(1);
    assert!(!acct_exist, "account should be empty");

    db.insert(account.clone());

    let acct: Account = db.read(1, |acct| acct.unwrap().clone());
    // println!("{:?}", acct);

    assert!(
        acct.client == account.client,
        "created client id and fetched client id are not equal"
    );
}

#[test]
fn test_account_storage_modify() {
    let account = Account {
        client: 1,
        available: 20.0,
        held: 0.0,
        total: 0.0,
        locked: false,
    };

    let db = AccountStorage::new();
    let exists = db.exists(1);
    assert!(!exists, "account should be empty");

    db.insert(account.clone());

    let acct: Account = db.read(1, |acct| acct.unwrap().clone());

    assert!(
        acct.available == account.available,
        "created client id and fetched client id are not equal"
    );

    let updated = db.modify(acct.client, |acc| {
        let a = acc.unwrap();
        a.available = 25.0;

        return *a;
    });

    db.insert(updated);

    assert!(
        updated.available == 25.0,
        "account not equal after modification; expect {}, got {}",
        25.0,
        updated.available,
    );
}
