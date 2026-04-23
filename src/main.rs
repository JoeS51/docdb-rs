use serde_json;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct DocumentKey {
    partition_key: String,
    collection: String,
    id: String,
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
            .open("db.log")
            .unwrap(); // consider removing this unwrap if this is bad practice
        let map: HashMap<DocumentKey, serde_json::Value> = HashMap::new();

        Database {
            documents: map,
            commit_log: db_file,
        }
    }

    fn add_document(db: &mut Self, key: &DocumentKey, value: &serde_json::Value) {
        db.documents.insert(key.clone(), value.clone());
    }

    fn get_document<'a>(db: &'a mut Self, key: &DocumentKey) -> Option<&'a serde_json::Value> {
        db.documents.get(key)
    }

    fn delete_document(db: &mut Self, key: &DocumentKey) {
        db.documents.remove(key);
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
