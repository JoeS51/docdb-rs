use bson::{Document, doc};
use docdb_rs::storage_engine::StorageEngine;
use docdb_rs::storage_engine::lsm::LsmStorage;
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
use std::path::Path;

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

struct Query {
    partition_key: String,
    collection: String,
    filter: Option<FieldFilter>,
}

struct FieldFilter {
    field: String,
    value: serde_json::Value,
}

impl Database {
    fn open(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let db_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path.as_ref())?;
        let mut map: HashMap<DocumentKey, serde_json::Value> = HashMap::new();

        let reader = BufReader::new(&db_file);
        for line in reader.lines() {
            let line = line?;
            let entry: LogEntry = serde_json::from_str(&line)?;
            match entry {
                LogEntry::Add { key, value } => {
                    map.insert(key, value);
                }
                LogEntry::Delete { key } => {
                    map.remove(&key);
                }
            }
        }

        Ok(Database {
            documents: map,
            commit_log: db_file,
        })
    }

    fn add_document(
        &mut self,
        key: &DocumentKey,
        value: &serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        let key = key.clone();
        let value = value.clone();
        self.documents.insert(key.clone(), value.clone());
        let log_entry = LogEntry::Add { key, value };
        serde_json::to_writer(&mut self.commit_log, &log_entry)?;
        self.commit_log.write_all(b"\n")?;
        self.commit_log.flush()?;
        Ok(())
    }

    fn get_document(&self, key: &DocumentKey) -> Option<&serde_json::Value> {
        self.documents.get(key)
    }

    fn delete_document(&mut self, key: &DocumentKey) -> Result<(), Box<dyn Error>> {
        let key = key.clone();
        self.documents.remove(&key);
        let log_entry = LogEntry::Delete { key: key };
        // TODO: get rid of unwrap
        serde_json::to_writer(&mut self.commit_log, &log_entry)?;
        self.commit_log.write_all(b"\n")?;
        self.commit_log.flush()?;
        Ok(())
    }

    fn scan_collection(
        &self,
        partition_key: &str,
        collection: &str,
    ) -> Vec<(&DocumentKey, &serde_json::Value)> {
        self.documents
            .iter()
            .filter(|(k, _)| k.partition_key == partition_key && k.collection == collection)
            .collect()
    }

    fn scan_collection_where(
        &self,
        partition_key: &str,
        collection: &str,
        field: &str,
        expected_value: &serde_json::Value,
    ) -> Vec<(&DocumentKey, &serde_json::Value)> {
        self.scan_collection(partition_key, collection)
            .into_iter()
            .filter(|(_, value)| value.get(field) == Some(expected_value))
            .collect()
    }

    fn execute_query(&self, query: &Query) -> Vec<(&DocumentKey, &serde_json::Value)> {
        let res: Vec<(&DocumentKey, &serde_json::Value)> = self
            .documents
            .iter()
            .filter(|(k, _)| {
                k.partition_key == query.partition_key && k.collection == query.collection
            })
            .collect();
        if let Some(field_filter) = &query.filter {
            res.into_iter()
                .filter(|(_, value)| value.get(&field_filter.field) == Some(&field_filter.value))
                .collect()
        } else {
            res
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // cli operations:
    // put <key> <value>
    // get <key> <value>
    // clear
    // help
    let mut lsm = LsmStorage::new();
    loop {
        println!("Enter DB operation");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read input");
        parse_cli_input(input, &mut lsm)?;
    }
    // let mut db = Database::open(Path::new("db.log"))?;
    // let john = json!({
    //     "name": "John Doe",
    //     "age": 43,
    // });
    // let doc_key = DocumentKey {
    //     partition_key: "partition1".to_string(),
    //     collection: "collection1".to_string(),
    //     id: "id1".to_string(),
    // };
    // db.add_document(&doc_key, &john)?;

    // db.get_document(&doc_key);

    // Ok(())
}

fn parse_cli_input(input: String, db: &mut LsmStorage) -> Result<(), Box<dyn Error>> {
    let vec: Vec<&str> = input.split_whitespace().collect();
    match vec.as_slice() {
        ["put", key, value] => {
            db.put(key.as_bytes().to_vec(), value.as_bytes().to_vec())
                .unwrap();
            println!("PUT VALUE");
        }
        ["get", key] => {
            let val = db.get(key.as_bytes()).unwrap();
            println!("GOT VALUE {:?}", val);
        }
        ["clear"] => {
            println!("CLEARED")
        }
        ["help"] => {
            println!("------------");
            println!("put <KEY> <VALUE>");
            println!("get <KEY>");
            println!("clear");
            println!("------------");
        }
        _ => {
            println!("invalid input");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_document() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.log");
        let mut test_db = Database::open(&db_path).unwrap();

        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        test_db.add_document(&doc_key, &val).unwrap();
        let get_result = test_db.get_document(&doc_key);
        assert_eq!(&val, get_result.unwrap());
    }

    #[test]
    fn test_get_missing_document() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.log");
        let test_db = Database::open(&db_path).unwrap();

        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        let get_result = test_db.get_document(&doc_key);
        assert_eq!(None, get_result);
    }

    #[test]
    fn test_delete_document() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.log");
        let mut test_db = Database::open(&db_path).unwrap();

        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        test_db.add_document(&doc_key, &val).unwrap();
        test_db.delete_document(&doc_key).unwrap();
        let get_result = test_db.get_document(&doc_key);
        assert_eq!(None, get_result);
    }

    #[test]
    fn test_scan_collection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.log");
        let mut test_db = Database::open(&db_path).unwrap();

        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let val2 = json!({
            "name": "Joe Sluis",
            "age": 22,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        let doc_key2 = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id2".to_string(),
        };
        test_db.add_document(&doc_key, &val).unwrap();
        test_db.add_document(&doc_key2, &val2).unwrap();
        let scan_result = test_db.scan_collection("partition1", "collection1");
        assert!(scan_result.contains(&(&doc_key, &val)));
        assert!(scan_result.contains(&(&doc_key2, &val2)));
    }

    #[test]
    fn test_execute_query() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.log");
        let mut test_db = Database::open(&db_path).unwrap();

        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let val2 = json!({
            "name": "Joe Sluis",
            "age": 22,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        let doc_key2 = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id2".to_string(),
        };
        test_db.add_document(&doc_key, &val).unwrap();
        test_db.add_document(&doc_key2, &val2).unwrap();

        let query_result = test_db.execute_query(&Query {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            filter: Some(FieldFilter {
                field: "age".to_string(),
                value: json!(43),
            }),
        });
        assert!(query_result.contains(&(&doc_key, &val)));
        assert!(!query_result.contains(&(&doc_key2, &val2)));
    }

    #[test]
    fn test_reopen_keeps_document() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.log");
        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        let val2 = json!({
            "name": "Joe Sluis",
            "age": 22,
        });
        let doc_key2 = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id2".to_string(),
        };

        // open the database, add a document, and close the db
        {
            let mut test_db = Database::open(&db_path).unwrap();
            test_db.add_document(&doc_key, &val).unwrap();
        }
        {
            let mut test_db = Database::open(&db_path).unwrap();
            test_db.add_document(&doc_key2, &val2).unwrap();
        }

        let reopened_db = Database::open(&db_path).unwrap();

        assert_eq!(Some(&val), reopened_db.get_document(&doc_key));
        assert_eq!(Some(&val2), reopened_db.get_document(&doc_key2));
    }

    #[test]
    fn test_reopen_keeps_delete() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.log");
        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };

        {
            let mut test_db = Database::open(&db_path).unwrap();
            test_db.add_document(&doc_key, &val).unwrap();
        }
        {
            let mut test_db = Database::open(&db_path).unwrap();
            test_db.delete_document(&doc_key).unwrap();
        }

        let reopened_db = Database::open(&db_path).unwrap();

        assert!(reopened_db.get_document(&doc_key).is_none());
    }
}
