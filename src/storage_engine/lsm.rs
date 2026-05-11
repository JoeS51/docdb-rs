// LSM-based storage engine
use super::{StorageEngine, StorageError};
use std::collections::BTreeMap;
use std::path::Path;
use std::fs::{self, File};

enum ValueEntry {
    Put(Vec<u8>),
    Tombstone,
}

struct Sstable {
    file_path: Path,
}

pub struct LsmStorage {
    mem_table: BTreeMap<Vec<u8>, ValueEntry>,
    sstables: Vec<SsTable>,
}

impl LsmStorage {
    pub fn new() -> Self {
    }
}

impl StorageEngine for LsmStorage {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StorageError> {
    }
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
    }
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError> {
    }
}
