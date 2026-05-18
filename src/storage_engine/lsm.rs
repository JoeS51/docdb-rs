// LSM-based storage engine
use super::{StorageEngine, StorageError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const MEMTABLE_CAPACITY: usize = 2;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ValueEntry {
    Put(Vec<u8>),
    Tombstone,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SsTableRecord {
    key: Vec<u8>,
    entry: ValueEntry,
}

struct Sstable {
    file_path: PathBuf,
}

pub struct LsmStorage {
    mem_table: BTreeMap<Vec<u8>, ValueEntry>,
    sstables: Vec<Sstable>,
    next_table_id: u64,
    wal_handle: File,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WalEntry {
    Put { key: Vec<u8>, value: ValueEntry },
    Delete { key: Vec<u8> },
}

impl Sstable {
    pub fn new(fp: PathBuf) -> Self {
        Self { file_path: fp }
    }
}

impl LsmStorage {
    pub fn new() -> Self {
        let wal_file = File::options()
            .append(true)
            .read(true)
            .open("wal.log")
            .unwrap();
        Self {
            mem_table: BTreeMap::new(),
            sstables: Vec::new(),
            next_table_id: 1, // TODO: need to dynamically find latest sstable
            wal_handle: wal_file,
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
        let mut out = String::new();
        for (k, v) in &self.mem_table {
            let curr_record = SsTableRecord {
                key: k.clone(),
                entry: v.clone(),
            };
            let line = serde_json::to_string(&curr_record)
                .map_err(|e| StorageError::Io(std::io::Error::other(e)))?;
            out.push_str(&line);
            out.push('\n');
        }
        let path = self.add_sstable()?;
        // let content = self
        //     .mem_table
        //     .iter()
        //     .map(|(k, v)| format!("{:?}: {:?}", k, v))
        //     .collect::<Vec<_>>()
        //     .join("\n");

        // fs::write(&path, content).unwrap();
        // let file = File::create(path).unwrap();
        // let writer = BufWriter::new(file);
        // serde_json::to_writer(writer, &entries).unwrap();
        fs::write(&path, out).unwrap();
        // serde_json::to_writer(&mut self.commit_log, &log_entry)?;
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
            let record: SsTableRecord = serde_json::from_str(&line)
                .map_err(|e| StorageError::Io(std::io::Error::other(e)))?;
            if record.key == key {
                return match record.entry {
                    ValueEntry::Put(value) => Ok(Some(value)),
                    ValueEntry::Tombstone => Ok(None),
                };
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
        let mut out = String::new();
        let wal_entry = WalEntry::Put {
            key: key.clone(),
            value: ValueEntry::Put(value.clone()),
        };
        let wal_entry_serialized = serde_json::to_string(&wal_entry)
            .map_err(|e| StorageError::Io(std::io::Error::other(e)))?;
        out.push_str(&wal_entry_serialized);
        out.push('\n');
        self.wal_handle
            .write_all(out.as_bytes())
            .map_err(StorageError::Io)?;
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
        let mut out = String::new();
        let wal_entry = WalEntry::Delete { key: key.clone() };
        let wal_entry_serialized = serde_json::to_string(&wal_entry)
            .map_err(|e| StorageError::Io(std::io::Error::other(e)))?;
        out.push_str(&wal_entry_serialized);
        out.push('\n');
        self.wal_handle
            .write_all(out.as_bytes())
            .map_err(StorageError::Io)?;
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
        storage
            .put(b"sstable2".to_vec(), b"world".to_vec())
            .unwrap();
        storage
            .put(b"sstable2.1".to_vec(), b"world".to_vec())
            .unwrap();
        storage
            .put(b"sstable3".to_vec(), b"world".to_vec())
            .unwrap();
        storage.delete(b"sstable3".to_vec()).unwrap();
        assert_eq!(storage.get(b"sstable3").unwrap(), Some(b"world".to_vec()));
        assert_eq!(storage.get(b"sstable1").unwrap(), Some(b"world".to_vec()));
        assert_eq!(storage.get(b"sstable2.1").unwrap(), Some(b"world".to_vec()));
    }
}
