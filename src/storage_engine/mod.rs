pub mod lsm;
pub mod memory;
<<<<<<< HEAD
pub mod lsm;
=======
>>>>>>> 9170c425d9abd9cd11f65c7c01682ee17a22cba1

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
}

pub trait StorageEngine {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StorageError>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError>;
}
