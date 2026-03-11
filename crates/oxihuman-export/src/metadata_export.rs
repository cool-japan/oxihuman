// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Scene/asset metadata export.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MetadataExportConfig {
    pub schema_version: String,
    pub author: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MetadataExport {
    pub config: MetadataExportConfig,
    pub tags: Vec<String>,
    pub properties: Vec<(String, String)>,
}

#[allow(dead_code)]
pub fn default_metadata_export_config() -> MetadataExportConfig {
    MetadataExportConfig {
        schema_version: "1.0".to_string(),
        author: "oxihuman".to_string(),
    }
}

#[allow(dead_code)]
pub fn new_metadata_export(config: MetadataExportConfig) -> MetadataExport {
    MetadataExport { config, tags: Vec::new(), properties: Vec::new() }
}

#[allow(dead_code)]
pub fn meta_add_tag(meta: &mut MetadataExport, tag: &str) {
    meta.tags.push(tag.to_string());
}

#[allow(dead_code)]
pub fn meta_remove_tag(meta: &mut MetadataExport, tag: &str) {
    meta.tags.retain(|t| t != tag);
}

#[allow(dead_code)]
pub fn meta_has_tag(meta: &MetadataExport, tag: &str) -> bool {
    meta.tags.iter().any(|t| t == tag)
}

#[allow(dead_code)]
pub fn meta_add_property(meta: &mut MetadataExport, key: &str, value: &str) {
    meta.properties.push((key.to_string(), value.to_string()));
}

#[allow(dead_code)]
pub fn meta_get_property<'a>(meta: &'a MetadataExport, key: &str) -> Option<&'a str> {
    meta.properties
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

#[allow(dead_code)]
pub fn meta_tag_count(meta: &MetadataExport) -> usize {
    meta.tags.len()
}

#[allow(dead_code)]
pub fn meta_property_count(meta: &MetadataExport) -> usize {
    meta.properties.len()
}

#[allow(dead_code)]
pub fn meta_to_json(meta: &MetadataExport) -> String {
    format!(
        "{{\"schema_version\":\"{}\",\"author\":\"{}\",\"tag_count\":{},\"property_count\":{}}}",
        meta.config.schema_version,
        meta.config.author,
        meta.tags.len(),
        meta.properties.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_meta() -> MetadataExport {
        new_metadata_export(default_metadata_export_config())
    }

    #[test]
    fn test_default_config() {
        let cfg = default_metadata_export_config();
        assert_eq!(cfg.schema_version, "1.0");
    }

    #[test]
    fn test_add_tag() {
        let mut m = make_meta();
        meta_add_tag(&mut m, "character");
        assert_eq!(meta_tag_count(&m), 1);
    }

    #[test]
    fn test_has_tag() {
        let mut m = make_meta();
        meta_add_tag(&mut m, "rigged");
        assert!(meta_has_tag(&m, "rigged"));
        assert!(!meta_has_tag(&m, "other"));
    }

    #[test]
    fn test_remove_tag() {
        let mut m = make_meta();
        meta_add_tag(&mut m, "a");
        meta_add_tag(&mut m, "b");
        meta_remove_tag(&mut m, "a");
        assert_eq!(meta_tag_count(&m), 1);
        assert!(!meta_has_tag(&m, "a"));
    }

    #[test]
    fn test_add_property() {
        let mut m = make_meta();
        meta_add_property(&mut m, "scene", "main");
        assert_eq!(meta_property_count(&m), 1);
    }

    #[test]
    fn test_get_property() {
        let mut m = make_meta();
        meta_add_property(&mut m, "version", "2");
        assert_eq!(meta_get_property(&m, "version"), Some("2"));
        assert!(meta_get_property(&m, "missing").is_none());
    }

    #[test]
    fn test_to_json() {
        let m = make_meta();
        let j = meta_to_json(&m);
        assert!(j.contains("schema_version"));
    }

    #[test]
    fn test_property_count() {
        let mut m = make_meta();
        meta_add_property(&mut m, "k1", "v1");
        meta_add_property(&mut m, "k2", "v2");
        assert_eq!(meta_property_count(&m), 2);
    }
}
