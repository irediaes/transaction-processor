use serde::{Deserialize, Serialize};

use crate::tx::storage;

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
