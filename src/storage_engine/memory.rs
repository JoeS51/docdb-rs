// The simplest in memory DB
use super::{StorageEngine, StorageError};
use std::collections::HashMap;

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
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.map.get(key).cloned())
    }
    fn delete(&mut self, key: Vec<u8>) -> Result<(), StorageError> {
        self.map.remove(&key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_then_get_returns_value() {
        let mut storage = MemoryStorage::new();
        storage.put(b"hello".to_vec(), b"world".to_vec()).unwrap();
        assert_eq!(storage.get(b"hello").unwrap(), Some(b"world".to_vec()));
    }

    #[test]
    fn put_then_delete_returns_none() {
        let mut storage = MemoryStorage::new();
        storage.put(b"hello".to_vec(), b"world".to_vec()).unwrap();
        storage.delete(b"hello".to_vec()).unwrap();
        assert_eq!(storage.get(b"hello").unwrap(), None);
    }

    #[test]
    fn get_missing_key() {
        let storage = MemoryStorage::new();
        assert_eq!(storage.get(b"hello").unwrap(), None);
    }
}
