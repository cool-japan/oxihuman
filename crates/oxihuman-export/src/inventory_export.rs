// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 3D asset inventory export (name, type, stats).

/// Type of 3D asset.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Animation,
    Skeleton,
    Scene,
    Other(String),
}

impl AssetType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            AssetType::Mesh => "mesh",
            AssetType::Texture => "texture",
            AssetType::Material => "material",
            AssetType::Animation => "animation",
            AssetType::Skeleton => "skeleton",
            AssetType::Scene => "scene",
            AssetType::Other(s) => s.as_str(),
        }
    }
}

/// Stats for a 3D asset.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssetStats {
    pub vertex_count: u32,
    pub face_count: u32,
    pub file_size_bytes: u64,
    pub lod_levels: u8,
}

impl AssetStats {
    #[allow(dead_code)]
    pub fn zero() -> Self {
        Self {
            vertex_count: 0,
            face_count: 0,
            file_size_bytes: 0,
            lod_levels: 0,
        }
    }
}

/// A single asset inventory entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssetEntry {
    pub id: u32,
    pub name: String,
    pub asset_type: AssetType,
    pub path: String,
    pub stats: AssetStats,
}

/// A 3D asset inventory.
#[allow(dead_code)]
pub struct AssetInventory {
    pub project: String,
    pub entries: Vec<AssetEntry>,
    next_id: u32,
}

impl AssetInventory {
    #[allow(dead_code)]
    pub fn new(project: &str) -> Self {
        Self {
            project: project.to_string(),
            entries: Vec::new(),
            next_id: 0,
        }
    }
}

/// Add an asset to the inventory.
#[allow(dead_code)]
pub fn add_asset(
    inventory: &mut AssetInventory,
    name: &str,
    asset_type: AssetType,
    path: &str,
    stats: AssetStats,
) -> u32 {
    let id = inventory.next_id;
    inventory.next_id += 1;
    inventory.entries.push(AssetEntry {
        id,
        name: name.to_string(),
        asset_type,
        path: path.to_string(),
        stats,
    });
    id
}

/// Export inventory to CSV string.
#[allow(dead_code)]
pub fn export_inventory_csv(inventory: &AssetInventory) -> String {
    let mut out = String::from("id,name,type,path,vertices,faces,size_bytes,lod_levels\n");
    for e in &inventory.entries {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            e.id,
            e.name,
            e.asset_type.as_str(),
            e.path,
            e.stats.vertex_count,
            e.stats.face_count,
            e.stats.file_size_bytes,
            e.stats.lod_levels
        ));
    }
    out
}

/// Export inventory to JSON-like string.
#[allow(dead_code)]
pub fn export_inventory_json(inventory: &AssetInventory) -> String {
    let mut out = format!("{{\"project\":\"{}\",\"assets\":[", inventory.project);
    for (i, e) in inventory.entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"id\":{},\"name\":\"{}\",\"type\":\"{}\",\"path\":\"{}\",\
            \"vertices\":{},\"faces\":{},\"size_bytes\":{},\"lod_levels\":{}}}",
            e.id,
            e.name,
            e.asset_type.as_str(),
            e.path,
            e.stats.vertex_count,
            e.stats.face_count,
            e.stats.file_size_bytes,
            e.stats.lod_levels
        ));
    }
    out.push_str("]}");
    out
}

/// Total asset count.
#[allow(dead_code)]
pub fn asset_count(inventory: &AssetInventory) -> usize {
    inventory.entries.len()
}

/// Count assets by type.
#[allow(dead_code)]
pub fn count_by_type(inventory: &AssetInventory, asset_type: &AssetType) -> usize {
    inventory
        .entries
        .iter()
        .filter(|e| &e.asset_type == asset_type)
        .count()
}

/// Total file size across all assets.
#[allow(dead_code)]
pub fn total_file_size(inventory: &AssetInventory) -> u64 {
    inventory
        .entries
        .iter()
        .map(|e| e.stats.file_size_bytes)
        .sum()
}

/// Total vertex count across all mesh assets.
#[allow(dead_code)]
pub fn total_vertex_count(inventory: &AssetInventory) -> u64 {
    inventory
        .entries
        .iter()
        .map(|e| e.stats.vertex_count as u64)
        .sum()
}

/// Find asset by name.
#[allow(dead_code)]
pub fn find_asset_by_name<'a>(inventory: &'a AssetInventory, name: &str) -> Option<&'a AssetEntry> {
    inventory.entries.iter().find(|e| e.name == name)
}

/// Find asset by ID.
#[allow(dead_code)]
pub fn find_asset_by_id(inventory: &AssetInventory, id: u32) -> Option<&AssetEntry> {
    inventory.entries.iter().find(|e| e.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_inventory() -> AssetInventory {
        let mut inv = AssetInventory::new("oxihuman-project");
        add_asset(
            &mut inv,
            "head_mesh",
            AssetType::Mesh,
            "assets/head.glb",
            AssetStats {
                vertex_count: 5000,
                face_count: 9800,
                file_size_bytes: 204800,
                lod_levels: 3,
            },
        );
        add_asset(
            &mut inv,
            "body_mesh",
            AssetType::Mesh,
            "assets/body.glb",
            AssetStats {
                vertex_count: 12000,
                face_count: 23000,
                file_size_bytes: 512000,
                lod_levels: 4,
            },
        );
        add_asset(
            &mut inv,
            "skin_texture",
            AssetType::Texture,
            "assets/skin.png",
            AssetStats {
                vertex_count: 0,
                face_count: 0,
                file_size_bytes: 1048576,
                lod_levels: 0,
            },
        );
        inv
    }

    #[test]
    fn asset_count_correct() {
        let inv = sample_inventory();
        assert_eq!(asset_count(&inv), 3);
    }

    #[test]
    fn count_by_type_mesh() {
        let inv = sample_inventory();
        assert_eq!(count_by_type(&inv, &AssetType::Mesh), 2);
    }

    #[test]
    fn count_by_type_texture() {
        let inv = sample_inventory();
        assert_eq!(count_by_type(&inv, &AssetType::Texture), 1);
    }

    #[test]
    fn total_file_size_correct() {
        let inv = sample_inventory();
        assert_eq!(total_file_size(&inv), 204800 + 512000 + 1048576);
    }

    #[test]
    fn total_vertex_count_correct() {
        let inv = sample_inventory();
        assert_eq!(total_vertex_count(&inv), 17000);
    }

    #[test]
    fn csv_header_present() {
        let inv = sample_inventory();
        let csv = export_inventory_csv(&inv);
        assert!(csv.starts_with("id,name,type"));
    }

    #[test]
    fn json_contains_project() {
        let inv = sample_inventory();
        let json = export_inventory_json(&inv);
        assert!(json.contains("oxihuman-project"));
    }

    #[test]
    fn find_asset_by_name_some() {
        let inv = sample_inventory();
        assert!(find_asset_by_name(&inv, "head_mesh").is_some());
    }

    #[test]
    fn find_asset_by_name_none() {
        let inv = sample_inventory();
        assert!(find_asset_by_name(&inv, "nonexistent").is_none());
    }

    #[test]
    fn find_asset_by_id_correct() {
        let inv = sample_inventory();
        let e = find_asset_by_id(&inv, 1);
        assert!(e.is_some_and(|a| a.name == "body_mesh"));
    }
}
