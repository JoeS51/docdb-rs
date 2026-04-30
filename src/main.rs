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
        println!("hashmap");
        println!("{:?}", map);

        Ok(Database {
            documents: map,
            commit_log: db_file,
        })
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

    fn scan_collection<'a>(
        db: &'a Self,
        partition_key: &str,
        collection: &str,
    ) -> Vec<(&'a DocumentKey, &'a serde_json::Value)> {
        db.documents
            .iter()
            .filter(|(k, _)| k.partition_key == partition_key && k.collection == collection)
            .collect()
    }

    fn scan_collection_where<'a>(
        db: &'a Self,
        partition_key: &str,
        collection: &str,
        field: &str,
        expected_value: &serde_json::Value,
    ) -> Vec<(&'a DocumentKey, &'a serde_json::Value)> {
        Database::scan_collection(db, partition_key, collection)
            .into_iter()
            .filter(|(_, value)| value.get(field) == Some(expected_value))
            .collect()
    }

    fn execute_query<'a>(
        db: &'a Self,
        query: &Query,
    ) -> Vec<(&'a DocumentKey, &'a serde_json::Value)> {
        let res: Vec<(&'a DocumentKey, &'a serde_json::Value)> = db
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
    let mut db = Database::open(Path::new("db.log"))?;
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

    // Database::delete_document(&mut db, &doc_key);

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
        let mut test_db = Database::open(Path::new("test_get.log")).unwrap();
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
        let mut test_db = Database::open(Path::new("test_missing.log")).unwrap();
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
        let mut test_db = Database::open(Path::new("test_delete.log")).unwrap();
        Database::add_document(&mut test_db, &doc_key, &val);
        Database::delete_document(&mut test_db, &doc_key);
        let get_result = Database::get_document(&mut test_db, &doc_key);
        assert_eq!(None, get_result);
    }

    #[test]
    fn test_scan_collection() {
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
        let mut test_db = Database::open(Path::new("test_scan.log")).unwrap();
        Database::add_document(&mut test_db, &doc_key, &val);
        Database::add_document(&mut test_db, &doc_key2, &val2);
        let scan_result = Database::scan_collection(&test_db, "partition1", "collection1");
        assert!(scan_result.contains(&(&doc_key, &val)));
        assert!(scan_result.contains(&(&doc_key2, &val2)));
    }
}
