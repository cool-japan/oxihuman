//! Type-erased key-value storage with type-safe retrieval.

#[allow(dead_code)]
pub enum StoredValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
}

#[allow(dead_code)]
pub struct StorageConfig {
    pub max_entries: usize,
    pub namespace: String,
}

#[allow(dead_code)]
pub struct TypedStorage {
    pub config: StorageConfig,
    pub data: Vec<(String, StoredValue)>,
}

#[allow(dead_code)]
pub fn default_storage_config() -> StorageConfig {
    StorageConfig {
        max_entries: 256,
        namespace: String::from("default"),
    }
}

#[allow(dead_code)]
pub fn new_typed_storage(cfg: StorageConfig) -> TypedStorage {
    TypedStorage { config: cfg, data: Vec::new() }
}

#[allow(dead_code)]
pub fn storage_set(store: &mut TypedStorage, key: &str, val: StoredValue) {
    if let Some(entry) = store.data.iter_mut().find(|(k, _)| k == key) {
        entry.1 = val;
        return;
    }
    if store.data.len() < store.config.max_entries {
        store.data.push((key.to_string(), val));
    }
}

#[allow(dead_code)]
pub fn storage_get<'a>(store: &'a TypedStorage, key: &str) -> Option<&'a StoredValue> {
    store.data.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

#[allow(dead_code)]
pub fn storage_remove(store: &mut TypedStorage, key: &str) -> bool {
    let before = store.data.len();
    store.data.retain(|(k, _)| k != key);
    store.data.len() < before
}

#[allow(dead_code)]
pub fn storage_contains(store: &TypedStorage, key: &str) -> bool {
    store.data.iter().any(|(k, _)| k == key)
}

#[allow(dead_code)]
pub fn storage_get_bool(store: &TypedStorage, key: &str) -> Option<bool> {
    if let Some(StoredValue::Bool(b)) = storage_get(store, key) {
        return Some(*b);
    }
    None
}

#[allow(dead_code)]
pub fn storage_get_int(store: &TypedStorage, key: &str) -> Option<i64> {
    if let Some(StoredValue::Int(i)) = storage_get(store, key) {
        return Some(*i);
    }
    None
}

#[allow(dead_code)]
pub fn storage_get_float(store: &TypedStorage, key: &str) -> Option<f64> {
    if let Some(StoredValue::Float(f)) = storage_get(store, key) {
        return Some(*f);
    }
    None
}


#[allow(dead_code)]
pub fn storage_get_text<'a>(store: &'a TypedStorage, key: &str) -> Option<&'a str> {
    if let Some(StoredValue::Text(s)) = storage_get(store, key) {
        return Some(s.as_str());
    }
    None
}

#[allow(dead_code)]
pub fn storage_entry_count(store: &TypedStorage) -> usize {
    store.data.len()
}

#[allow(dead_code)]
pub fn storage_to_json(store: &TypedStorage) -> String {
    let mut parts = Vec::new();
    for (k, v) in &store.data {
        let val_str = match v {
            StoredValue::Bool(b) => format!("{}", b),
            StoredValue::Int(i) => format!("{}", i),
            StoredValue::Float(f) => format!("{:.6}", f),
            StoredValue::Text(s) => format!("\"{}\"", s),
            StoredValue::Bytes(b) => format!("[{} bytes]", b.len()),
        };
        parts.push(format!("\"{}\":{}", k, val_str));
    }
    format!("{{{}}}", parts.join(","))
}

#[allow(dead_code)]
pub fn storage_value_type_name(val: &StoredValue) -> &'static str {
    match val {
        StoredValue::Bool(_) => "bool",
        StoredValue::Int(_) => "int",
        StoredValue::Float(_) => "float",
        StoredValue::Text(_) => "text",
        StoredValue::Bytes(_) => "bytes",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> TypedStorage {
        new_typed_storage(default_storage_config())
    }

    #[test]
    fn test_default_config() {
        let cfg = default_storage_config();
        assert_eq!(cfg.max_entries, 256);
        assert_eq!(cfg.namespace, "default");
    }

    #[test]
    fn test_set_and_get_bool() {
        let mut store = make_store();
        storage_set(&mut store, "flag", StoredValue::Bool(true));
        assert_eq!(storage_get_bool(&store, "flag"), Some(true));
    }

    #[test]
    fn test_set_and_get_int() {
        let mut store = make_store();
        storage_set(&mut store, "count", StoredValue::Int(42));
        assert_eq!(storage_get_int(&store, "count"), Some(42));
    }

    #[test]
    fn test_set_and_get_float() {
        let mut store = make_store();
        storage_set(&mut store, "pi", StoredValue::Float(std::f64::consts::PI));
        let v = storage_get_float(&store, "pi").unwrap();
        assert!((v - std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn test_set_and_get_text() {
        let mut store = make_store();
        storage_set(&mut store, "name", StoredValue::Text("alice".to_string()));
        assert_eq!(storage_get_text(&store, "name"), Some("alice"));
    }

    #[test]
    fn test_overwrite_existing_key() {
        let mut store = make_store();
        storage_set(&mut store, "x", StoredValue::Int(1));
        storage_set(&mut store, "x", StoredValue::Int(2));
        assert_eq!(storage_entry_count(&store), 1);
        assert_eq!(storage_get_int(&store, "x"), Some(2));
    }

    #[test]
    fn test_contains_and_remove() {
        let mut store = make_store();
        storage_set(&mut store, "key", StoredValue::Bool(false));
        assert!(storage_contains(&store, "key"));
        let removed = storage_remove(&mut store, "key");
        assert!(removed);
        assert!(!storage_contains(&store, "key"));
        assert_eq!(storage_entry_count(&store), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut store = make_store();
        let removed = storage_remove(&mut store, "missing");
        assert!(!removed);
    }

    #[test]
    fn test_get_missing_key_returns_none() {
        let store = make_store();
        assert!(storage_get(&store, "nope").is_none());
        assert!(storage_get_bool(&store, "nope").is_none());
        assert!(storage_get_int(&store, "nope").is_none());
    }

    #[test]
    fn test_type_mismatch_returns_none() {
        let mut store = make_store();
        storage_set(&mut store, "x", StoredValue::Text("hello".to_string()));
        assert!(storage_get_int(&store, "x").is_none());
        assert!(storage_get_bool(&store, "x").is_none());
    }

    #[test]
    fn test_storage_to_json() {
        let mut store = make_store();
        storage_set(&mut store, "a", StoredValue::Int(1));
        storage_set(&mut store, "b", StoredValue::Bool(true));
        let json = storage_to_json(&store);
        assert!(json.contains("\"a\""));
        assert!(json.contains("\"b\""));
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(storage_value_type_name(&StoredValue::Bool(true)), "bool");
        assert_eq!(storage_value_type_name(&StoredValue::Int(0)), "int");
        assert_eq!(storage_value_type_name(&StoredValue::Float(0.0)), "float");
        assert_eq!(
            storage_value_type_name(&StoredValue::Text(String::new())),
            "text"
        );
        assert_eq!(
            storage_value_type_name(&StoredValue::Bytes(vec![])),
            "bytes"
        );
    }

    #[test]
    fn test_bytes_storage() {
        let mut store = make_store();
        storage_set(&mut store, "data", StoredValue::Bytes(vec![1, 2, 3]));
        let v = storage_get(&store, "data").unwrap();
        assert_eq!(storage_value_type_name(v), "bytes");
        let json = storage_to_json(&store);
        assert!(json.contains("3 bytes"));
    }
}
