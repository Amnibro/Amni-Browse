use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct IdbIndex {
    pub name: String,
    pub key_path: String,
    pub unique: bool,
}

#[derive(Debug, Clone)]
pub struct IdbObjectStore {
    pub name: String,
    pub key_path: Option<String>,
    pub auto_increment: bool,
    pub records: HashMap<String, Value>,
    pub indexes: HashMap<String, IdbIndex>,
}

impl IdbObjectStore {
    pub fn put(&mut self, key: &str, value: Value) {
        self.records.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.records.get(key)
    }

    pub fn delete(&mut self, key: &str) -> bool {
        self.records.remove(key).is_some()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }

    pub fn get_all(&self) -> Vec<(&String, &Value)> {
        self.records.iter().collect()
    }

    pub fn create_index(&mut self, name: &str, key_path: &str, unique: bool) {
        self.indexes.insert(name.to_string(), IdbIndex {
            name: name.to_string(),
            key_path: key_path.to_string(),
            unique,
        });
    }
}

#[derive(Debug, Clone)]
pub struct IdbDatabase {
    pub name: String,
    pub version: u32,
    pub object_stores: HashMap<String, IdbObjectStore>,
}

impl IdbDatabase {
    pub fn create_object_store(&mut self, name: &str, key_path: Option<&str>, auto_increment: bool) -> &mut IdbObjectStore {
        self.object_stores.insert(name.to_string(), IdbObjectStore {
            name: name.to_string(),
            key_path: key_path.map(|s| s.to_string()),
            auto_increment,
            records: HashMap::new(),
            indexes: HashMap::new(),
        });
        self.object_stores.get_mut(name).unwrap()
    }

    pub fn delete_object_store(&mut self, name: &str) -> bool {
        self.object_stores.remove(name).is_some()
    }

    pub fn object_store_names(&self) -> Vec<String> {
        self.object_stores.keys().cloned().collect()
    }
}

pub struct IdbManager {
    databases: HashMap<String, IdbDatabase>,
}

impl IdbManager {
    pub fn new() -> Self { Self { databases: HashMap::new() } }

    fn make_key(origin: &str, name: &str) -> String {
        format!("{}::{}", origin, name)
    }

    pub fn open(&mut self, origin: &str, name: &str, version: u32) -> &mut IdbDatabase {
        let key = Self::make_key(origin, name);
        self.databases.entry(key).or_insert_with(|| IdbDatabase {
            name: name.to_string(),
            version,
            object_stores: HashMap::new(),
        })
    }

    pub fn delete_database(&mut self, origin: &str, name: &str) -> bool {
        let key = Self::make_key(origin, name);
        self.databases.remove(&key).is_some()
    }

    pub fn list_databases(&self, origin: &str) -> Vec<String> {
        let prefix = format!("{}::", origin);
        self.databases.keys()
            .filter(|k| k.starts_with(&prefix))
            .map(|k| k[prefix.len()..].to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_open_and_list() {
        let mut mgr = IdbManager::new();
        mgr.open("https://example.com", "mydb", 1);
        mgr.open("https://example.com", "other", 1);
        mgr.open("https://other.com", "mydb", 1);
        let dbs = mgr.list_databases("https://example.com");
        assert_eq!(dbs.len(), 2);
        assert!(dbs.contains(&"mydb".to_string()));
        assert!(dbs.contains(&"other".to_string()));
    }

    #[test]
    fn test_delete_database() {
        let mut mgr = IdbManager::new();
        mgr.open("https://example.com", "mydb", 1);
        assert!(mgr.delete_database("https://example.com", "mydb"));
        assert!(!mgr.delete_database("https://example.com", "mydb"));
        assert_eq!(mgr.list_databases("https://example.com").len(), 0);
    }

    #[test]
    fn test_object_store_crud() {
        let mut mgr = IdbManager::new();
        let db = mgr.open("https://example.com", "testdb", 1);
        db.create_object_store("users", Some("id"), false);
        db.create_object_store("logs", None, true);
        let names = db.object_store_names();
        assert_eq!(names.len(), 2);
        assert!(db.delete_object_store("logs"));
        assert!(!db.delete_object_store("logs"));
        assert_eq!(db.object_store_names().len(), 1);
    }

    #[test]
    fn test_put_get_delete_clear() {
        let mut store = IdbObjectStore {
            name: "test".to_string(),
            key_path: None,
            auto_increment: false,
            records: HashMap::new(),
            indexes: HashMap::new(),
        };
        store.put("k1", json!({"name": "Alice"}));
        store.put("k2", json!(42));
        assert_eq!(store.count(), 2);
        assert_eq!(store.get("k1"), Some(&json!({"name": "Alice"})));
        assert_eq!(store.get("missing"), None);
        assert!(store.delete("k1"));
        assert!(!store.delete("k1"));
        assert_eq!(store.count(), 1);
        store.clear();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_get_all_and_indexes() {
        let mut store = IdbObjectStore {
            name: "test".to_string(),
            key_path: Some("id".to_string()),
            auto_increment: false,
            records: HashMap::new(),
            indexes: HashMap::new(),
        };
        store.put("a", json!(1));
        store.put("b", json!(2));
        store.put("c", json!(3));
        let all = store.get_all();
        assert_eq!(all.len(), 3);
        store.create_index("by_name", "name", true);
        store.create_index("by_age", "age", false);
        assert_eq!(store.indexes.len(), 2);
        let idx = store.indexes.get("by_name").unwrap();
        assert_eq!(idx.key_path, "name");
        assert!(idx.unique);
    }

    #[test]
    fn test_put_overwrites() {
        let mut store = IdbObjectStore {
            name: "test".to_string(),
            key_path: None,
            auto_increment: false,
            records: HashMap::new(),
            indexes: HashMap::new(),
        };
        store.put("k1", json!("old"));
        store.put("k1", json!("new"));
        assert_eq!(store.count(), 1);
        assert_eq!(store.get("k1"), Some(&json!("new")));
    }
}
