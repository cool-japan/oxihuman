// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Asset bundle manifest export.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssetEntry {
    pub id: u32,
    pub name: String,
    pub asset_type: String,
    pub size_bytes: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssetBundleExport {
    pub name: String,
    pub version: u32,
    pub assets: Vec<AssetEntry>,
}

#[allow(dead_code)]
pub fn new_asset_bundle_export(name: &str, version: u32) -> AssetBundleExport {
    AssetBundleExport {
        name: name.to_string(),
        version,
        assets: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn bundle_add_asset(bundle: &mut AssetBundleExport, entry: AssetEntry) {
    bundle.assets.push(entry);
}

#[allow(dead_code)]
pub fn bundle_remove_asset(bundle: &mut AssetBundleExport, id: u32) {
    bundle.assets.retain(|a| a.id != id);
}

#[allow(dead_code)]
pub fn bundle_asset_count(bundle: &AssetBundleExport) -> usize {
    bundle.assets.len()
}

#[allow(dead_code)]
pub fn bundle_get_asset(bundle: &AssetBundleExport, id: u32) -> Option<&AssetEntry> {
    bundle.assets.iter().find(|a| a.id == id)
}

#[allow(dead_code)]
pub fn bundle_total_size(bundle: &AssetBundleExport) -> usize {
    bundle.assets.iter().map(|a| a.size_bytes).sum()
}

#[allow(dead_code)]
pub fn bundle_find_by_name<'a>(bundle: &'a AssetBundleExport, name: &str) -> Option<&'a AssetEntry> {
    bundle.assets.iter().find(|a| a.name == name)
}

#[allow(dead_code)]
pub fn bundle_validate(bundle: &AssetBundleExport) -> bool {
    !bundle.name.is_empty()
}

#[allow(dead_code)]
pub fn bundle_to_json(bundle: &AssetBundleExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"version\":{},\"asset_count\":{},\"total_bytes\":{}}}",
        bundle.name,
        bundle.version,
        bundle.assets.len(),
        bundle_total_size(bundle)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: u32, name: &str) -> AssetEntry {
        AssetEntry { id, name: name.to_string(), asset_type: "mesh".to_string(), size_bytes: 1024 }
    }

    #[test]
    fn test_new_bundle() {
        let b = new_asset_bundle_export("pack", 1);
        assert_eq!(b.name, "pack");
        assert_eq!(bundle_asset_count(&b), 0);
    }

    #[test]
    fn test_add_asset() {
        let mut b = new_asset_bundle_export("pack", 1);
        bundle_add_asset(&mut b, make_entry(1, "mesh1"));
        assert_eq!(bundle_asset_count(&b), 1);
    }

    #[test]
    fn test_remove_asset() {
        let mut b = new_asset_bundle_export("pack", 1);
        bundle_add_asset(&mut b, make_entry(1, "a"));
        bundle_add_asset(&mut b, make_entry(2, "b"));
        bundle_remove_asset(&mut b, 1);
        assert_eq!(bundle_asset_count(&b), 1);
    }

    #[test]
    fn test_get_asset() {
        let mut b = new_asset_bundle_export("pack", 1);
        bundle_add_asset(&mut b, make_entry(42, "tex"));
        assert!(bundle_get_asset(&b, 42).is_some());
        assert!(bundle_get_asset(&b, 99).is_none());
    }

    #[test]
    fn test_total_size() {
        let mut b = new_asset_bundle_export("pack", 1);
        bundle_add_asset(&mut b, make_entry(1, "a")); // 1024
        bundle_add_asset(&mut b, make_entry(2, "b")); // 1024
        assert_eq!(bundle_total_size(&b), 2048);
    }

    #[test]
    fn test_find_by_name() {
        let mut b = new_asset_bundle_export("pack", 1);
        bundle_add_asset(&mut b, make_entry(1, "skeleton"));
        assert!(bundle_find_by_name(&b, "skeleton").is_some());
        assert!(bundle_find_by_name(&b, "other").is_none());
    }

    #[test]
    fn test_validate() {
        let b = new_asset_bundle_export("pack", 1);
        assert!(bundle_validate(&b));
        let bad = AssetBundleExport { name: "".to_string(), version: 1, assets: Vec::new() };
        assert!(!bundle_validate(&bad));
    }

    #[test]
    fn test_to_json() {
        let b = new_asset_bundle_export("mypack", 2);
        let j = bundle_to_json(&b);
        assert!(j.contains("mypack"));
    }
}
