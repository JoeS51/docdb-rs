// LSM-based storage engine
use super::{StorageEngine, StorageError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const MEMTABLE_CAPACITY: usize = 2;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    next_table_id: u64,
}

impl Sstable {
    pub fn new(fp: PathBuf) -> Self {
        Self { file_path: fp }
    }
}

impl LsmStorage {
    pub fn new() -> Self {
        Self {
            mem_table: BTreeMap::new(),
            sstables: Vec::new(),
            next_table_id: 1, // TODO: need to dynamically find latest sstable
        }
    }

    pub fn create_next_sstable_path(&mut self) -> Result<PathBuf, StorageError> {
        let path = Self::sstable_path_for_id(self.next_table_id);
        self.next_table_id += 1;
        Ok(path)
    }

    pub fn add_sstable(&mut self) -> Result<PathBuf, StorageError> {
        let path = self.create_next_sstable_path()?;
        self.sstables.push(Sstable::new(path.clone()));
        Ok(path)
    }

    pub fn flush_memtable(&mut self) -> Result<(), StorageError> {
        // Flushes memtable after it reaches some capacity threshold
        let path = self.add_sstable()?;
        let content = self
            .mem_table
            .iter()
            .map(|(k, v)| format!("{:?}: {:?}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&path, content).unwrap();
        self.mem_table.clear();
        Ok(())
    }

    pub fn is_full(&self) -> bool {
        if self.mem_table.len() >= MEMTABLE_CAPACITY {
            true
        } else {
            false
        }
    }

    pub fn search_sstable(
        &self,
        key: &[u8],
        sstable_id: u64,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        let path = Self::sstable_path_for_id(sstable_id);
        let file = File::open(path).map_err(StorageError::Io)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.map_err(StorageError::Io)?;
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 && parts[0].trim().as_bytes() == key {
                return Ok(Some(parts[1].to_string().into_bytes()));
            }
        }
        Ok(None)
    }

    fn sstable_path_for_id(id: u64) -> PathBuf {
        Path::new("sstables").join(format!("{}.sst", id))
    }
}

impl StorageEngine for LsmStorage {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StorageError> {
        if self.is_full() {
            self.flush_memtable()?;
        }
        self.mem_table.insert(key, ValueEntry::Put(value));
        Ok(())
    }
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        if let Some(entry) = self.mem_table.get(key) {
            match entry {
                ValueEntry::Put(value) => return Ok(Some(value.clone())),
                ValueEntry::Tombstone => return Ok(None),
            }
        } else {
            // Search sstables since key wasn't in memtable
            let mut curr_table_id = self.next_table_id - 1;
            while curr_table_id > 0 {
                println!("iterating down");
                let res = self.search_sstable(key, curr_table_id)?;
                if let Some(value) = res {
                    return Ok(Some(value));
                }
                curr_table_id -= 1;
            }
            return Ok(None);
        }
    }
    fn delete(&mut self, key: Vec<u8>) -> Result<(), StorageError> {
        self.mem_table.insert(key, ValueEntry::Tombstone);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_then_get_returns_value() {
        let mut storage = LsmStorage::new();
        storage
            .put(b"sstable1".to_vec(), b"world".to_vec())
            .unwrap();
        storage
            .put(b"sstable1.1".to_vec(), b"sstable1.1".to_vec())
            .unwrap();
        // storage
        //     .put(b"sstable2".to_vec(), b"world".to_vec())
        //     .unwrap();
        // storage
        //     .put(b"sstable2.1".to_vec(), b"world".to_vec())
        //     .unwrap();
        storage
            .put(b"sstable3".to_vec(), b"world".to_vec())
            .unwrap();
        assert_eq!(storage.get(b"sstable3").unwrap(), Some(b"world".to_vec()));
        assert_eq!(storage.get(b"sstable1").unwrap(), Some(b"world".to_vec()));
    }
}
