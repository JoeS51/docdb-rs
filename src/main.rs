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
    add_document(&mut db, &doc_key, john);
    println!("{:?}", db);
}

fn add_document(
    db: &mut HashMap<DocumentKey, serde_json::Value>,
    key: &DocumentKey,
    value: serde_json::Value,
) {
    db.insert(key.clone(), value);
}

fn get_document(
    db: &mut HashMap<DocumentKey, serde_json::Value>,
    key: &DocumentKey,
) -> Option<serde_json::Value> {
    let val = db.get(key);
    val.cloned()
}

fn delete_document(db: &mut HashMap<DocumentKey, serde_json::Value>, key: &DocumentKey) {
    db.remove(key);
}
