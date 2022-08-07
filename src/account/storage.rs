use std::{
    collections::{hash_map::Iter, HashMap},
    sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;

use crate::Account;

pub static mut ACCOUNTS: Lazy<Mutex<AccountStorage>> =
    Lazy::new(|| Mutex::new(AccountStorage::new()));

pub struct AccountStorage {
    accounts: Arc<Mutex<HashMap<u16, Account>>>,
}

impl AccountStorage {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn read_account<F, R>(&self, id: u16, f: F) -> R
    where
        F: FnOnce(Option<&Account>) -> R,
    {
        f(self.accounts.lock().unwrap().get(&id))
    }

    pub fn read_accounts<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Iter<u16, Account>) -> R,
    {
        f(self.accounts.lock().unwrap().iter())
    }

    pub fn insert_account(&self, account: Account) -> bool {
        let acc = self
            .accounts
            .lock()
            .unwrap()
            .insert(account.client, account.clone());

        if let Some(_acct) = acc {
            return true;
        }

        false
    }

    pub fn modify_account<F, R>(&self, id: u16, f: F) -> R
    where
        F: FnOnce(Option<&mut Account>) -> R,
    {
        f(self.accounts.lock().unwrap().get_mut(&id))
    }

    pub fn account_exists(&self, id: u16) -> bool {
        self.accounts.lock().unwrap().contains_key(&id)
    }

    pub fn clear_accounts(&self) {
        self.accounts.lock().unwrap().clear();
    }
}

/// Tests

#[test]
fn test_insert_account() {
    let account = Account {
        client: 1,
        available: 20.0,
        held: 0.0,
        total: 0.0,
        locked: false,
    };

    let db = AccountStorage::new();
    let acct_exist = db.account_exists(1);
    assert!(!acct_exist, "account should be empty");

    db.insert_account(account.clone());

    let acct: Account = db.read_account(1, |acct| acct.unwrap().clone());
    println!("{:?}", acct);

    assert!(
        acct.client == account.client,
        "created client id and fetched client id are not equal"
    );
}

#[test]
fn test_modify_account() {
    unsafe {
        ACCOUNTS.lock().unwrap().clear_accounts();
    }
    let account = Account {
        client: 1,
        available: 20.0,
        held: 0.0,
        total: 0.0,
        locked: false,
    };

    let db = AccountStorage::new();
    let acct_exist = db.account_exists(1);
    assert!(!acct_exist, "account should be empty");

    db.insert_account(account.clone());

    let acct: Account = db.read_account(1, |acct| acct.unwrap().clone());
    println!("{:?}", acct);

    assert!(
        acct.available == account.available,
        "created client id and fetched client id are not equal"
    );

    let updated_account = db.modify_account(acct.client, |acc| {
        let a = acc.unwrap();
        a.available = 25.0;

        db.insert_account(*a);

        return *a;
    });

    assert!(
        updated_account.available == account.available,
        "account not equal after modification; expect {}, got {}",
        account.available,
        updated_account.available,
    );
}
