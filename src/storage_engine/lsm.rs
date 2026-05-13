// LSM-based storage engine
use super::{StorageEngine, StorageError};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

enum ValueEntry {
    Put(Vec<u8>),
    Tombstone,
}

struct Sstable {
    file_path: PathBuf,
}

pub struct LsmStorage {
    mem_table: BTreeMap<Vec<u8>, ValueEntry>,
    sstables: Vec<Sstable>,
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
        todo!()
    }
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        todo!()
    }
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError> {
        todo!()
    }
}
