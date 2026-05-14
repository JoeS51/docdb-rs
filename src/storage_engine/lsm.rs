// LSM-based storage engine
use super::{StorageEngine, StorageError};
use std::collections::BTreeMap;
<<<<<<< HEAD
use std::path::PathBuf;
=======
>>>>>>> 9170c425d9abd9cd11f65c7c01682ee17a22cba1
use std::fs::{self, File};
use std::path::{Path, PathBuf};

enum ValueEntry {
    Put(Vec<u8>),
    Tombstone,
}

<<<<<<< HEAD
struct SsTable {
=======
struct Sstable {
>>>>>>> 9170c425d9abd9cd11f65c7c01682ee17a22cba1
    file_path: PathBuf,
}

pub struct LsmStorage {
    mem_table: BTreeMap<Vec<u8>, ValueEntry>,
    sstables: Vec<Sstable>,
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
<<<<<<< HEAD
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
=======
        todo!()
    }
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        todo!()
    }
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError> {
        todo!()
>>>>>>> 9170c425d9abd9cd11f65c7c01682ee17a22cba1
    }
}
