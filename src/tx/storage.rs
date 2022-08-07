use std::{
    collections::{hash_map::Iter, HashMap},
    sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;

use crate::tx::transaction::Transaction;

pub static mut TRANSACTIONS: Lazy<Mutex<TransactionStorage>> =
    Lazy::new(|| Mutex::new(TransactionStorage::new()));

pub struct TransactionStorage {
    transactions: Arc<Mutex<HashMap<u32, Transaction>>>,
}

impl TransactionStorage {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn read<F, R>(&self, id: u32, f: F) -> R
    where
        F: FnOnce(Option<&Transaction>) -> R,
    {
        f(self.transactions.lock().unwrap().get(&id))
    }

    pub fn insert(&self, transaction: Transaction) -> bool {
        let acc = self
            .transactions
            .lock()
            .unwrap()
            .insert(transaction.tx, transaction.clone());

        if let Some(_acct) = acc {
            return true;
        }

        false
    }

    pub fn exists(&self, id: u32) -> bool {
        self.transactions.lock().unwrap().contains_key(&id)
    }

    pub fn clear(&self) {
        self.transactions.lock().unwrap().clear();
    }
}

/// Tests

#[test]
fn test_insert() {
    unsafe {
        TRANSACTIONS.lock().unwrap().clear();
    }
    let transaction = Transaction {
        r#type: "deposit".to_string(),
        client: 1,
        tx: 1,
        amount: 10.0,
    };

    let db = TransactionStorage::new();
    let exists = db.exists(1);
    assert!(!exists, "transaction should be empty");

    db.insert(transaction.clone());

    let tranx: Transaction = db.read(1, |acct| acct.unwrap().clone());

    assert!(
        tranx.r#type == transaction.r#type,
        "created client id and fetched client id are not equal"
    );
}
