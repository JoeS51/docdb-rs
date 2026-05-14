// LSM-based storage engine
use super::{StorageEngine, StorageError};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
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
            next_table_id: 0, // TODO: need to dynamically find latest sstable
        }
    }

    pub fn create_next_sstable_path(&mut self) -> Result<PathBuf, StorageError> {
        let path = Path::new("sstables").join(format!("{}.sst", self.next_table_id));
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
        if self.mem_table.len() >= 2 {
            true
        } else {
            false
        }
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
        Ok(match self.mem_table.get(key) {
            Some(ValueEntry::Put(value)) => Some(value.clone()),
            Some(ValueEntry::Tombstone) => None,
            None => None,
        })
    }
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError> {
        self.mem_table.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_then_get_returns_value() {
        let mut storage = LsmStorage::new();
        storage.put(b"hello".to_vec(), b"world".to_vec()).unwrap();
        storage.put(b"test".to_vec(), b"world".to_vec()).unwrap();
        storage.put(b"test2".to_vec(), b"world".to_vec()).unwrap();
        assert_eq!(storage.get(b"hello").unwrap(), Some(b"world".to_vec()));
    }
}
