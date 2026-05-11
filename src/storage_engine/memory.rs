// The simplest in memory DB
use super::{StorageEngine, StorageError};
use std::Collections::HashMap;

pub struct MemoryStorage {
    map: HashMap<Vec<u8>, Vec<u8>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl StorageEngine for MemoryStorage {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StorageError> {
        self.map.insert(key, value);
        Ok(())
    }
    fn get(&self, key: Vec<u8>) -> Result<Vec<u8>, StorageError> {
        todo!();
    }
}
