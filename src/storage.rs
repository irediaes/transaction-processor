use std::{
    collections::{hash_map::Iter, HashMap},
    hash::Hash,
    sync::{Arc, Mutex},
};

pub trait StoreKey {
    type Key: Hash + Eq;

    fn key(&self) -> Self::Key;
}

pub struct Storage<K, D> {
    data: Arc<Mutex<HashMap<K, D>>>,
}

impl<K: Hash + Eq, D: StoreKey<Key = K>> Storage<K, D> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn read<F, R>(&self, id: K, f: F) -> R
    where
        F: FnOnce(Option<&D>) -> R,
    {
        f(self.data.lock().unwrap().get(&id))
    }

    pub fn reads<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Iter<K, D>) -> R,
    {
        f(self.data.lock().unwrap().iter())
    }

    pub fn insert(&self, item: D) -> bool {
        let acc = self.data.lock().unwrap().insert(item.key(), item);

        if let Some(_acct) = acc {
            return true;
        }

        false
    }

    pub fn modify<F, R>(&self, id: K, f: F) -> R
    where
        F: FnOnce(Option<&mut D>) -> R,
    {
        f(self.data.lock().unwrap().get_mut(&id))
    }

    pub fn exists(&self, id: K) -> bool {
        self.data.lock().unwrap().contains_key(&id)
    }
}

// Tests
#[cfg(test)]
mod tests {
    use crate::storage::Storage;

    use super::StoreKey;

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct Dummy {
        id: u16,
    }

    impl StoreKey for Dummy {
        type Key = u16;
        fn key(&self) -> Self::Key {
            self.id
        }
    }

    #[test]

    fn test_storage_insert() {
        let dummy = Dummy { id: 1 };

        let db = Storage::<u16, Dummy>::new();
        let acct_exist = db.exists(1);
        assert!(!acct_exist, "account should be empty");

        db.insert(dummy.clone());

        let dumb: Dummy = db.read(1, |dumm| dumm.unwrap().clone());
        // println!("{:?}", acct);

        assert!(
            dumb.id == dummy.id,
            "created id and fetched id are not equal; expected {}, got {}",
            dummy.id,
            dumb.id
        );
    }

    #[test]
    fn test_storage_modify() {
        let dummy = Dummy { id: 1 };

        let db = Storage::<u16, Dummy>::new();
        let exists = db.exists(1);
        assert!(!exists, "account should be empty");

        db.insert(dummy.clone());

        let dumb: Dummy = db.read(1, |acct| acct.unwrap().clone());

        assert!(
            dumb.id == dummy.id,
            "created id and fetched id are not equal; expected {}, got {}",
            dummy.id,
            dumb.id
        );

        let updated = db.modify(dummy.id, |dumm| {
            let dmy = dumm.unwrap();
            dmy.id = 25;

            return *dmy;
        });

        db.insert(updated);

        assert!(
            updated.id == 25,
            "dummy not equal after modification; expect {}, got {}",
            25,
            updated.id,
        );
    }
}
