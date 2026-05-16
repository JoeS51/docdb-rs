pub mod lsm;
pub mod memory;

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
}

pub trait StorageEngine {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StorageError>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn delete(&mut self, key: Vec<u8>) -> Result<(), StorageError>;
}
