use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::Transaction;

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    client: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
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
}
