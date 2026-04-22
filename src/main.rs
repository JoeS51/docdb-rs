use serde_json;
use serde_json::json;
use std::collections::HashMap;

struct Document {
    key: String,
    value: serde_json::Value,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
struct DocumentKey {
    partition_key: String,
    collection: String,
    id: String,
}

fn main() {
    let mut db: HashMap<DocumentKey, serde_json::Value> = HashMap::new();
    let john = json!({
        "name": "John Doe",
        "age": 43,
    });
    let doc_key = DocumentKey {
        partition_key: "partition1".to_string(),
        collection: "collection1".to_string(),
        id: "id1".to_string(),
    };
    add_document(&mut db, &doc_key, &john);
    println!("Insert document: {:?}", db);

    let result = get_document(&db, &doc_key);
    println!("Get document: {:?}", result);

    delete_document(&mut db, &doc_key);
    println!("Delete document: {:?}", db);
}

fn add_document(
    db: &mut HashMap<DocumentKey, serde_json::Value>,
    key: &DocumentKey,
    value: &serde_json::Value,
) {
    db.insert(key.clone(), value.clone());
}

fn get_document<'a>(
    db: &'a HashMap<DocumentKey, serde_json::Value>,
    key: &DocumentKey,
) -> Option<&'a serde_json::Value> {
    db.get(key)
}

fn delete_document(db: &mut HashMap<DocumentKey, serde_json::Value>, key: &DocumentKey) {
    db.remove(key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_document() {
        let mut db: HashMap<DocumentKey, serde_json::Value> = HashMap::new();
        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        add_document(&mut db, &doc_key, &val);
        let get_result = get_document(&db, &doc_key);
        assert_eq!(&val, get_result.unwrap());
    }

    #[test]
    fn test_delete_document() {
        let mut db: HashMap<DocumentKey, serde_json::Value> = HashMap::new();
        let val = json!({
            "name": "John Doe",
            "age": 43,
        });
        let doc_key = DocumentKey {
            partition_key: "partition1".to_string(),
            collection: "collection1".to_string(),
            id: "id1".to_string(),
        };
        add_document(&mut db, &doc_key, &val);
        delete_document(&mut db, &doc_key);
        let get_result = get_document(&db, &doc_key);
        assert_eq!(None, get_result);
    }
}
