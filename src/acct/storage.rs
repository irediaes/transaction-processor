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
