//! Asset metadata export (distinct from metadata_export).
#![allow(dead_code)]

use std::collections::HashMap;

/// Asset metadata store.
#[allow(dead_code)]
pub struct AssetMetadata2 {
    pub fields: HashMap<String, String>,
    pub created_at: String,
}

/// Create a new asset metadata object.
#[allow(dead_code)]
pub fn new_asset_metadata2(created_at: &str) -> AssetMetadata2 {
    AssetMetadata2 { fields: HashMap::new(), created_at: created_at.to_string() }
}

/// Set a metadata field.
#[allow(dead_code)]
pub fn set_meta2_field(meta: &mut AssetMetadata2, key: &str, value: &str) {
    meta.fields.insert(key.to_string(), value.to_string());
}

/// Get a metadata field.
#[allow(dead_code)]
pub fn get_meta2_field<'a>(meta: &'a AssetMetadata2, key: &str) -> Option<&'a str> {
    meta.fields.get(key).map(|s| s.as_str())
}

/// Export metadata to JSON.
#[allow(dead_code)]
pub fn export_metadata2_json(meta: &AssetMetadata2) -> String {
    let fields: Vec<String> = meta.fields.iter()
        .map(|(k, v)| format!(r#""{}":"{}""#, k, v))
        .collect();
    format!(r#"{{"created_at":"{}","fields":{{{}}}}}"#, meta.created_at, fields.join(","))
}

/// Get field count.
#[allow(dead_code)]
pub fn meta2_field_count(meta: &AssetMetadata2) -> usize { meta.fields.len() }

/// Check if a field exists.
#[allow(dead_code)]
pub fn meta2_has_field(meta: &AssetMetadata2, key: &str) -> bool { meta.fields.contains_key(key) }

/// Convert to TOML-like string.
#[allow(dead_code)]
pub fn meta2_to_toml_string(meta: &AssetMetadata2) -> String {
    let mut s = format!("created_at = \"{}\"\n", meta.created_at);
    for (k, v) in &meta.fields {
        s.push_str(&format!("{} = \"{}\"\n", k, v));
    }
    s
}

/// Get created_at timestamp.
#[allow(dead_code)]
pub fn meta2_created_at(meta: &AssetMetadata2) -> &str { &meta.created_at }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metadata_empty() {
        let m = new_asset_metadata2("2026-01-01");
        assert_eq!(meta2_field_count(&m), 0);
    }

    #[test]
    fn test_set_get_field() {
        let mut m = new_asset_metadata2("2026-01-01");
        set_meta2_field(&mut m, "author", "Alice");
        assert_eq!(get_meta2_field(&m, "author"), Some("Alice"));
    }

    #[test]
    fn test_get_missing_field() {
        let m = new_asset_metadata2("2026-01-01");
        assert!(get_meta2_field(&m, "missing").is_none());
    }

    #[test]
    fn test_export_json() {
        let mut m = new_asset_metadata2("2026-03-06");
        set_meta2_field(&mut m, "version", "1.0");
        let j = export_metadata2_json(&m);
        assert!(j.contains("created_at"));
    }

    #[test]
    fn test_meta_field_count() {
        let mut m = new_asset_metadata2("2026-01-01");
        set_meta2_field(&mut m, "a", "1");
        set_meta2_field(&mut m, "b", "2");
        assert_eq!(meta2_field_count(&m), 2);
    }

    #[test]
    fn test_meta_has_field() {
        let mut m = new_asset_metadata2("2026-01-01");
        set_meta2_field(&mut m, "format", "glb");
        assert!(meta2_has_field(&m, "format"));
        assert!(!meta2_has_field(&m, "missing"));
    }

    #[test]
    fn test_meta_to_toml_string() {
        let mut m = new_asset_metadata2("2026-01-01");
        set_meta2_field(&mut m, "desc", "test asset");
        let t = meta2_to_toml_string(&m);
        assert!(t.contains("created_at"));
    }

    #[test]
    fn test_meta_created_at() {
        let m = new_asset_metadata2("2026-03-06");
        assert_eq!(meta2_created_at(&m), "2026-03-06");
    }

    #[test]
    fn test_meta_overwrite_field() {
        let mut m = new_asset_metadata2("2026-01-01");
        set_meta2_field(&mut m, "key", "v1");
        set_meta2_field(&mut m, "key", "v2");
        assert_eq!(get_meta2_field(&m, "key"), Some("v2"));
        assert_eq!(meta2_field_count(&m), 1);
    }
}
