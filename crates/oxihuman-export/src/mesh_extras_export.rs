#![allow(dead_code)]
//! Mesh extras export.

/// Mesh extras export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MeshExtrasExport {
    pub entries: Vec<(String, String)>,
}

/// Export mesh extras.
#[allow(dead_code)]
pub fn export_mesh_extras(entries: Vec<(String, String)>) -> MeshExtrasExport {
    MeshExtrasExport { entries }
}

/// Get key count.
#[allow(dead_code)]
pub fn extras_key_count(e: &MeshExtrasExport) -> usize {
    e.entries.len()
}

/// Get value at index.
#[allow(dead_code)]
pub fn extras_value_at(e: &MeshExtrasExport, index: usize) -> &str {
    if index < e.entries.len() {
        &e.entries[index].1
    } else {
        ""
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn extras_to_json(e: &MeshExtrasExport) -> String {
    let pairs: Vec<String> = e
        .entries
        .iter()
        .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
        .collect();
    format!("{{{}}}", pairs.join(","))
}

/// Check if key exists.
#[allow(dead_code)]
pub fn extras_has_key(e: &MeshExtrasExport, key: &str) -> bool {
    e.entries.iter().any(|(k, _)| k == key)
}

/// Get all keys.
#[allow(dead_code)]
pub fn extras_keys(e: &MeshExtrasExport) -> Vec<&str> {
    e.entries.iter().map(|(k, _)| k.as_str()).collect()
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn extras_export_size(e: &MeshExtrasExport) -> usize {
    e.entries.iter().map(|(k, v)| k.len() + v.len()).sum()
}

/// Validate extras.
#[allow(dead_code)]
pub fn validate_extras(e: &MeshExtrasExport) -> bool {
    e.entries.iter().all(|(k, _)| !k.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_mesh_extras() {
        let e = export_mesh_extras(vec![("key".to_string(), "val".to_string())]);
        assert_eq!(e.entries.len(), 1);
    }

    #[test]
    fn test_extras_key_count() {
        let e = export_mesh_extras(vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ]);
        assert_eq!(extras_key_count(&e), 2);
    }

    #[test]
    fn test_extras_value_at() {
        let e = export_mesh_extras(vec![("k".to_string(), "v".to_string())]);
        assert_eq!(extras_value_at(&e, 0), "v");
        assert_eq!(extras_value_at(&e, 5), "");
    }

    #[test]
    fn test_extras_to_json() {
        let e = export_mesh_extras(vec![("name".to_string(), "test".to_string())]);
        let j = extras_to_json(&e);
        assert!(j.contains("name"));
    }

    #[test]
    fn test_extras_has_key() {
        let e = export_mesh_extras(vec![("x".to_string(), "1".to_string())]);
        assert!(extras_has_key(&e, "x"));
        assert!(!extras_has_key(&e, "y"));
    }

    #[test]
    fn test_extras_keys() {
        let e = export_mesh_extras(vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ]);
        assert_eq!(extras_keys(&e), vec!["a", "b"]);
    }

    #[test]
    fn test_extras_export_size() {
        let e = export_mesh_extras(vec![("ab".to_string(), "cd".to_string())]);
        assert_eq!(extras_export_size(&e), 4);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_mesh_extras(vec![("k".to_string(), "v".to_string())]);
        assert!(validate_extras(&e));
    }

    #[test]
    fn test_validate_empty_key() {
        let e = export_mesh_extras(vec![("".to_string(), "v".to_string())]);
        assert!(!validate_extras(&e));
    }

    #[test]
    fn test_extras_empty() {
        let e = export_mesh_extras(vec![]);
        assert_eq!(extras_key_count(&e), 0);
        assert!(validate_extras(&e));
    }
}
