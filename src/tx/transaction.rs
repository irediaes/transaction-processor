use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::storage::{Storage, StoreKey};

pub static TRANSACTIONS: Lazy<Mutex<Storage<u32, Transaction>>> =
    Lazy::new(|| Mutex::new(Storage::new()));

pub static DISPUTES: Lazy<Mutex<Storage<u32, Dispute>>> = Lazy::new(|| Mutex::new(Storage::new()));

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub r#type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: f32,
}

impl Transaction {
    pub fn new(typ: String, client: u16, tx: u32, amount: f32) -> Self {
        Self {
            r#type: typ,
            client,
            tx,
            amount,
        }
    }
}

impl StoreKey for Transaction {
    type Key = u32;

    fn key(&self) -> Self::Key {
        self.tx
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Dispute {
    pub client: u16,
    pub tx: u32,
    pub resolved: bool,
}

impl Dispute {
    pub fn new(client: u16, tx: u32, resolved: bool) -> Self {
        Self {
            client,
            tx,
            resolved,
        }
    }
}

impl StoreKey for Dispute {
    type Key = u32;

    fn key(&self) -> Self::Key {
        self.tx
    }
}
