// LSM-based storage engine
use super::{StorageEngine, StorageError};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::fs::{self, File};

enum ValueEntry {
    Put(Vec<u8>),
    Tombstone,
}

struct SsTable {
    file_path: PathBuf,
}

pub struct LsmStorage {
    mem_table: BTreeMap<Vec<u8>, ValueEntry>,
    sstables: Vec<SsTable>,
}

impl SsTable {
    pub fn new() -> Self {
        Self {
            file_path: PathBuf::new(),
        }
    }
}

impl LsmStorage {
    pub fn new() -> Self {
        Self {
            mem_table: BTreeMap::new(),
            sstables: Vec::new(),
        }
    }
}

impl StorageEngine for LsmStorage {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StorageError> {
        self.mem_table.insert(key, ValueEntry::Put(value));
        Ok(())
    }
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(match self.mem_table.get(key) {
            Some(ValueEntry::Put(value)) => Some(value.clone()),
            Some(ValueEntry::Tombstone) => None,
            None => None,
        })
    }
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError> {
        todo!();
    }
}
