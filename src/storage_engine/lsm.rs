// LSM-based storage engine
use super::{StorageEngine, StorageError};

pub struct LSM {
    map: HashMap<Vec<u8>, Vec<u8>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
    }
}

impl StorageEngine for MemoryStorage {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StorageError> {
    }
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
    }
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError> {
    }
}
