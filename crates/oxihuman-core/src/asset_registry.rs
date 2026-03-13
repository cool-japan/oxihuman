// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Asset catalog with type tagging and metadata.

use std::collections::HashMap;

/// Broad category of a registered asset.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetKind {
    Mesh,
    Texture,
    Material,
    Animation,
    Audio,
    Other,
}

/// A single entry in the asset registry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssetCatalogEntry {
    pub id: u64,
    pub name: String,
    pub kind: AssetKind,
    pub metadata: HashMap<String, String>,
}

/// The central registry that holds all registered assets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssetCatalog {
    entries: HashMap<u64, AssetCatalogEntry>,
    next_id: u64,
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Create a new, empty [`AssetCatalog`].
#[allow(dead_code)]
pub fn new_asset_registry() -> AssetCatalog {
    AssetCatalog {
        entries: HashMap::new(),
        next_id: 1,
    }
}

/// Register an asset with the given name and kind. Returns the assigned ID.
#[allow(dead_code)]
pub fn register_asset(registry: &mut AssetCatalog, name: &str, kind: AssetKind) -> u64 {
    let id = registry.next_id;
    registry.next_id += 1;
    registry.entries.insert(
        id,
        AssetCatalogEntry {
            id,
            name: name.to_string(),
            kind,
            metadata: HashMap::new(),
        },
    );
    id
}

/// Remove the asset with the given ID. Returns `true` if it existed.
#[allow(dead_code)]
pub fn unregister_asset(registry: &mut AssetCatalog, id: u64) -> bool {
    registry.entries.remove(&id).is_some()
}

/// Find the first asset entry whose name matches exactly.
#[allow(dead_code)]
pub fn find_by_name<'a>(registry: &'a AssetCatalog, name: &str) -> Option<&'a AssetCatalogEntry> {
    registry.entries.values().find(|e| e.name == name)
}

/// Collect all asset entries of the given kind.
#[allow(dead_code)]
pub fn find_by_type<'a>(
    registry: &'a AssetCatalog,
    kind: &AssetKind,
) -> Vec<&'a AssetCatalogEntry> {
    registry
        .entries
        .values()
        .filter(|e| &e.kind == kind)
        .collect()
}

/// Return the total number of registered assets.
#[allow(dead_code)]
pub fn asset_count(registry: &AssetCatalog) -> usize {
    registry.entries.len()
}

/// Return the number of assets of a specific kind.
#[allow(dead_code)]
pub fn asset_count_by_type(registry: &AssetCatalog, kind: &AssetKind) -> usize {
    registry
        .entries
        .values()
        .filter(|e| &e.kind == kind)
        .count()
}

/// Serialise the registry to a simple JSON string.
#[allow(dead_code)]
pub fn registry_to_json(registry: &AssetCatalog) -> String {
    let mut items: Vec<String> = registry
        .entries
        .values()
        .map(|e| {
            format!(
                "  {{\"id\": {}, \"name\": \"{}\", \"kind\": \"{:?}\"}}",
                e.id, e.name, e.kind
            )
        })
        .collect();
    items.sort();
    format!("[\n{}\n]", items.join(",\n"))
}

/// List all asset names in the registry (unsorted).
#[allow(dead_code)]
pub fn list_asset_names(registry: &AssetCatalog) -> Vec<&str> {
    registry.entries.values().map(|e| e.name.as_str()).collect()
}

/// Get all metadata for the given asset ID.
#[allow(dead_code)]
pub fn get_asset_metadata(registry: &AssetCatalog, id: u64) -> Option<&HashMap<String, String>> {
    registry.entries.get(&id).map(|e| &e.metadata)
}

/// Set a single metadata key-value pair for the given asset ID.
/// Returns `false` if the asset was not found.
#[allow(dead_code)]
pub fn set_asset_metadata(registry: &mut AssetCatalog, id: u64, key: &str, value: &str) -> bool {
    match registry.entries.get_mut(&id) {
        Some(entry) => {
            entry.metadata.insert(key.to_string(), value.to_string());
            true
        }
        None => false,
    }
}

/// Return `true` if an asset with the given ID is present.
#[allow(dead_code)]
pub fn contains_asset(registry: &AssetCatalog, id: u64) -> bool {
    registry.entries.contains_key(&id)
}

/// Remove all assets from the registry.
#[allow(dead_code)]
pub fn clear_registry(registry: &mut AssetCatalog) {
    registry.entries.clear();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry() -> AssetCatalog {
        new_asset_registry()
    }

    #[test]
    fn test_new_registry_empty() {
        let r = make_registry();
        assert_eq!(asset_count(&r), 0);
    }

    #[test]
    fn test_register_asset_increases_count() {
        let mut r = make_registry();
        register_asset(&mut r, "hero_mesh", AssetKind::Mesh);
        assert_eq!(asset_count(&r), 1);
    }

    #[test]
    fn test_register_returns_unique_ids() {
        let mut r = make_registry();
        let id1 = register_asset(&mut r, "a", AssetKind::Mesh);
        let id2 = register_asset(&mut r, "b", AssetKind::Mesh);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_unregister_asset() {
        let mut r = make_registry();
        let id = register_asset(&mut r, "tex", AssetKind::Texture);
        assert!(unregister_asset(&mut r, id));
        assert_eq!(asset_count(&r), 0);
    }

    #[test]
    fn test_unregister_missing_returns_false() {
        let mut r = make_registry();
        assert!(!unregister_asset(&mut r, 9999));
    }

    #[test]
    fn test_find_by_name_found() {
        let mut r = make_registry();
        register_asset(&mut r, "body", AssetKind::Mesh);
        assert!(find_by_name(&r, "body").is_some());
    }

    #[test]
    fn test_find_by_name_not_found() {
        let r = make_registry();
        assert!(find_by_name(&r, "ghost").is_none());
    }

    #[test]
    fn test_find_by_type() {
        let mut r = make_registry();
        register_asset(&mut r, "m1", AssetKind::Mesh);
        register_asset(&mut r, "t1", AssetKind::Texture);
        register_asset(&mut r, "m2", AssetKind::Mesh);
        let meshes = find_by_type(&r, &AssetKind::Mesh);
        assert_eq!(meshes.len(), 2);
    }

    #[test]
    fn test_asset_count_by_type() {
        let mut r = make_registry();
        register_asset(&mut r, "a1", AssetKind::Audio);
        register_asset(&mut r, "a2", AssetKind::Audio);
        register_asset(&mut r, "m1", AssetKind::Mesh);
        assert_eq!(asset_count_by_type(&r, &AssetKind::Audio), 2);
        assert_eq!(asset_count_by_type(&r, &AssetKind::Mesh), 1);
    }

    #[test]
    fn test_registry_to_json_non_empty() {
        let mut r = make_registry();
        register_asset(&mut r, "anim_walk", AssetKind::Animation);
        let json = registry_to_json(&r);
        assert!(json.contains("anim_walk"));
    }

    #[test]
    fn test_list_asset_names() {
        let mut r = make_registry();
        register_asset(&mut r, "alpha", AssetKind::Other);
        let names = list_asset_names(&r);
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"alpha"));
    }

    #[test]
    fn test_set_and_get_metadata() {
        let mut r = make_registry();
        let id = register_asset(&mut r, "mat1", AssetKind::Material);
        assert!(set_asset_metadata(&mut r, id, "author", "alice"));
        let meta = get_asset_metadata(&r, id).expect("should succeed");
        assert_eq!(meta.get("author").map(String::as_str), Some("alice"));
    }

    #[test]
    fn test_set_metadata_missing_id() {
        let mut r = make_registry();
        assert!(!set_asset_metadata(&mut r, 999, "key", "val"));
    }

    #[test]
    fn test_contains_asset_true() {
        let mut r = make_registry();
        let id = register_asset(&mut r, "x", AssetKind::Other);
        assert!(contains_asset(&r, id));
    }

    #[test]
    fn test_contains_asset_false() {
        let r = make_registry();
        assert!(!contains_asset(&r, 42));
    }

    #[test]
    fn test_clear_registry() {
        let mut r = make_registry();
        register_asset(&mut r, "a", AssetKind::Mesh);
        register_asset(&mut r, "b", AssetKind::Texture);
        clear_registry(&mut r);
        assert_eq!(asset_count(&r), 0);
    }

    #[test]
    fn test_asset_kind_equality() {
        assert_eq!(AssetKind::Mesh, AssetKind::Mesh);
        assert_ne!(AssetKind::Mesh, AssetKind::Texture);
    }
}
