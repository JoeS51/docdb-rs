use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::json;
use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::error::Error;
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};

#[derive(Eq, Hash, PartialEq, Debug, Clone, Serialize, Deserialize)]
struct DocumentKey {
    partition_key: String,
    collection: String,
    id: String,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Serialize, Deserialize)]
enum LogEntry {
    Add {
        key: DocumentKey,
        value: serde_json::Value,
    },
    Delete {
        key: DocumentKey,
    },
}

struct Database {
    documents: HashMap<DocumentKey, serde_json::Value>,
    commit_log: File,
}

impl Database {
    fn from() -> Self {
        let db_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open("db.log")
            .unwrap(); // consider removing this unwrap if this is bad practice
        let map: HashMap<DocumentKey, serde_json::Value> = HashMap::new();

        // let reader = BufReader::new(&db_file);
        // for line in reader.lines() {
        //     let line = line.unwrap(); // potentially fix unwrap
        //     let parts: Vec<&str> = line.split(':').collect();
        //     println!("{:?}", parts);
        // }

        Database {
            documents: map,
            commit_log: db_file,
        }
    }

    fn add_document(db: &mut Self, key: &DocumentKey, value: &serde_json::Value) {
        let key = key.clone();
        let value = value.clone();
        db.documents.insert(key.clone(), value.clone());
        let log_entry = LogEntry::Add { key, value };
        // TODO: get rid of unwrap
        serde_json::to_writer(&mut db.commit_log, &log_entry).unwrap();
        db.commit_log.write_all(b"\n").unwrap();
        db.commit_log.flush().unwrap();
    }

    fn get_document<'a>(db: &'a mut Self, key: &DocumentKey) -> Option<&'a serde_json::Value> {
        db.documents.get(key)
    }

    fn delete_document(db: &mut Self, key: &DocumentKey) {
        let key = key.clone();
        db.documents.remove(&key);
        let log_entry = LogEntry::Delete { key: key };
        // TODO: get rid of unwrap
        serde_json::to_writer(&mut db.commit_log, &log_entry).unwrap();
        db.commit_log.write_all(b"\n").unwrap();
        db.commit_log.flush().unwrap();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::from();
    let john = json!({
        "name": "John Doe",
        "age": 43,
    });
    let doc_key = DocumentKey {
        partition_key: "partition1".to_string(),
        collection: "collection1".to_string(),
        id: "id1".to_string(),
    };
    Database::add_document(&mut db, &doc_key, &john);

    Database::get_document(&mut db, &doc_key);

    Database::delete_document(&mut db, &doc_key);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_document() {
        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        let mut test_db = Database::from();
        Database::add_document(&mut test_db, &doc_key, &val);
        let get_result = Database::get_document(&mut test_db, &doc_key);
        assert_eq!(&val, get_result.unwrap());
    }

    #[test]
    fn test_get_missing_document() {
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        let mut test_db = Database::from();
        let get_result = Database::get_document(&mut test_db, &doc_key);
        assert_eq!(None, get_result);
    }

    #[test]
    fn test_delete_document() {
        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        let mut test_db = Database::from();
        Database::add_document(&mut test_db, &doc_key, &val);
        Database::delete_document(&mut test_db, &doc_key);
        let get_result = Database::get_document(&mut test_db, &doc_key);
        assert_eq!(None, get_result);
    }
}
