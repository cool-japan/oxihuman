// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! COLLADA (.dae) stub export for DCC tool interchange.
//!
//! Provides lightweight COLLADA asset construction and XML serialization
//! without any external XML dependency.

#![allow(dead_code)]

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for COLLADA export.
#[derive(Debug, Clone)]
pub struct ColladaExportConfig {
    /// Author metadata. Default: `"OxiHuman"`.
    pub author: String,
    /// Up-axis string. Either `"Y_UP"` or `"Z_UP"`. Default: `"Y_UP"`.
    pub up_axis: String,
    /// Unit name. Default: `"meter"`.
    pub unit_name: String,
    /// Meters-per-unit scale. Default: `1.0`.
    pub unit_meter: f32,
    /// Whether to include normals in geometry output.
    pub include_normals: bool,
    /// Whether to include UV coordinates in geometry output.
    pub include_uvs: bool,
}

impl Default for ColladaExportConfig {
    fn default() -> Self {
        Self {
            author: "OxiHuman".to_string(),
            up_axis: "Y_UP".to_string(),
            unit_name: "meter".to_string(),
            unit_meter: 1.0,
            include_normals: true,
            include_uvs: true,
        }
    }
}

/// A COLLADA asset container.
#[derive(Debug, Clone)]
pub struct ColladaAsset {
    /// Unique asset identifier.
    pub id: String,
    /// Human-readable asset name.
    pub name: String,
    /// Author string embedded in asset metadata.
    pub author: String,
    /// Up-axis string (`"Y_UP"` or `"Z_UP"`).
    pub up_axis: String,
    /// Geometry objects belonging to this asset.
    pub geometries: Vec<ColladaGeometry>,
}

/// A single geometry object inside a COLLADA asset.
#[derive(Debug, Clone)]
pub struct ColladaGeometry {
    /// Unique geometry identifier.
    pub id: String,
    /// Vertex positions flattened as `[x, y, z, ...]`.
    pub positions: Vec<f32>,
    /// Face index list (triangles assumed).
    pub indices: Vec<u32>,
    /// Optional normals flattened as `[nx, ny, nz, ...]`.
    pub normals: Option<Vec<f32>>,
    /// Optional UV coordinates flattened as `[u, v, ...]`.
    pub uvs: Option<Vec<f32>>,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Result type for collada XML generation.
pub type ColladaXmlResult = Result<String, String>;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`ColladaExportConfig`].
#[allow(dead_code)]
pub fn default_collada_config() -> ColladaExportConfig {
    ColladaExportConfig::default()
}

/// Create a new [`ColladaAsset`] with the given `id` and `name`.
#[allow(dead_code)]
pub fn new_collada_asset(id: &str, name: &str) -> ColladaAsset {
    ColladaAsset {
        id: id.to_string(),
        name: name.to_string(),
        author: "OxiHuman".to_string(),
        up_axis: "Y_UP".to_string(),
        geometries: Vec::new(),
    }
}

/// Append a [`ColladaGeometry`] to `asset` and return the updated asset.
#[allow(dead_code)]
pub fn add_geometry(mut asset: ColladaAsset, geom: ColladaGeometry) -> ColladaAsset {
    asset.geometries.push(geom);
    asset
}

/// Return the number of geometries in `asset`.
#[allow(dead_code)]
pub fn geometry_count(asset: &ColladaAsset) -> usize {
    asset.geometries.len()
}

/// Produce a minimal COLLADA 1.4.1 XML string for `asset`.
///
/// Returns `Err` if the asset ID is empty.
#[allow(dead_code)]
pub fn collada_to_xml_stub(asset: &ColladaAsset, cfg: &ColladaExportConfig) -> ColladaXmlResult {
    if asset.id.is_empty() {
        return Err("asset id must not be empty".to_string());
    }
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<COLLADA xmlns=\"http://www.collada.org/2005/11/COLLADASchema\" version=\"1.4.1\">\n");

    // Asset metadata
    xml.push_str("  <asset>\n");
    xml.push_str(&format!(
        "    <contributor><author>{}</author></contributor>\n",
        cfg.author
    ));
    xml.push_str(&format!(
        "    <unit name=\"{}\" meter=\"{}\"/>\n",
        cfg.unit_name, cfg.unit_meter
    ));
    xml.push_str(&format!("    <up_axis>{}</up_axis>\n", asset.up_axis));
    xml.push_str("  </asset>\n");

    // Geometry library
    xml.push_str("  <library_geometries>\n");
    for g in &asset.geometries {
        xml.push_str(&format!("    <geometry id=\"{}\">\n", g.id));
        xml.push_str("      <mesh>\n");
        // Position source
        let pos_count = g.positions.len() / 3;
        xml.push_str(&format!(
            "        <source id=\"{}-positions\">\n",
            g.id
        ));
        xml.push_str(&format!(
            "          <float_array count=\"{}\">{}</float_array>\n",
            g.positions.len(),
            g.positions
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        ));
        xml.push_str(&format!(
            "          <technique_common><accessor count=\"{}\" stride=\"3\"/></technique_common>\n",
            pos_count
        ));
        xml.push_str("        </source>\n");

        // Vertices
        xml.push_str(&format!(
            "        <vertices id=\"{}-vertices\">\n",
            g.id
        ));
        xml.push_str(&format!(
            "          <input semantic=\"POSITION\" source=\"#{}-positions\"/>\n",
            g.id
        ));
        xml.push_str("        </vertices>\n");

        // Triangles
        let tri_count = g.indices.len() / 3;
        xml.push_str(&format!(
            "        <triangles count=\"{}\">\n",
            tri_count
        ));
        xml.push_str(&format!(
            "          <input semantic=\"VERTEX\" source=\"#{}-vertices\" offset=\"0\"/>\n",
            g.id
        ));
        let idx_str = g
            .indices
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        xml.push_str(&format!("          <p>{}</p>\n", idx_str));
        xml.push_str("        </triangles>\n");
        xml.push_str("      </mesh>\n");
        xml.push_str("    </geometry>\n");
    }
    xml.push_str("  </library_geometries>\n");
    xml.push_str("</COLLADA>\n");
    Ok(xml)
}

/// Validate a [`ColladaAsset`]: returns `Ok(())` or `Err` with a description.
#[allow(dead_code)]
pub fn validate_collada_asset(asset: &ColladaAsset) -> Result<(), String> {
    if asset.id.is_empty() {
        return Err("asset id is empty".to_string());
    }
    if asset.name.is_empty() {
        return Err("asset name is empty".to_string());
    }
    for g in &asset.geometries {
        if g.positions.is_empty() {
            return Err(format!("geometry '{}' has no positions", g.id));
        }
        if g.indices.len() % 3 != 0 {
            return Err(format!(
                "geometry '{}' index count not divisible by 3",
                g.id
            ));
        }
    }
    Ok(())
}

/// Return the asset's ID string.
#[allow(dead_code)]
pub fn collada_asset_id(asset: &ColladaAsset) -> &str {
    &asset.id
}

/// Set the author on an asset, returning the updated asset.
#[allow(dead_code)]
pub fn set_author(mut asset: ColladaAsset, author: &str) -> ColladaAsset {
    asset.author = author.to_string();
    asset
}

/// Set the up-axis on an asset, returning the updated asset.
///
/// Accepted values: `"Y_UP"`, `"Z_UP"`.
#[allow(dead_code)]
pub fn set_up_axis(mut asset: ColladaAsset, up_axis: &str) -> ColladaAsset {
    asset.up_axis = up_axis.to_string();
    asset
}

/// Estimate the file size in bytes for the XML output of `asset`.
///
/// This is a rough heuristic based on vertex and face counts.
#[allow(dead_code)]
pub fn collada_file_size_estimate(asset: &ColladaAsset) -> usize {
    let base = 512usize; // XML header + asset block
    let per_vertex = 24usize; // ~24 bytes per float3 as text
    let per_index = 8usize; // ~8 bytes per index
    let vc = collada_vertex_count(asset);
    let fc = collada_face_count(asset);
    base + vc * per_vertex + fc * 3 * per_index
}

/// Return the total number of vertices across all geometries in `asset`.
#[allow(dead_code)]
pub fn collada_vertex_count(asset: &ColladaAsset) -> usize {
    asset.geometries.iter().map(|g| g.positions.len() / 3).sum()
}

/// Return the total number of triangular faces across all geometries in `asset`.
#[allow(dead_code)]
pub fn collada_face_count(asset: &ColladaAsset) -> usize {
    asset.geometries.iter().map(|g| g.indices.len() / 3).sum()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_geom() -> ColladaGeometry {
        ColladaGeometry {
            id: "mesh0".to_string(),
            positions: vec![
                0.0, 0.0, 0.0, // v0
                1.0, 0.0, 0.0, // v1
                0.0, 1.0, 0.0, // v2
            ],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
        }
    }

    fn sample_asset() -> ColladaAsset {
        let asset = new_collada_asset("asset0", "TestAsset");
        add_geometry(asset, sample_geom())
    }

    #[test]
    fn default_config_fields() {
        let cfg = default_collada_config();
        assert_eq!(cfg.author, "OxiHuman");
        assert_eq!(cfg.up_axis, "Y_UP");
        assert_eq!(cfg.unit_name, "meter");
        assert!((cfg.unit_meter - 1.0).abs() < 1e-6);
        assert!(cfg.include_normals);
        assert!(cfg.include_uvs);
    }

    #[test]
    fn new_asset_has_no_geometries() {
        let a = new_collada_asset("id1", "name1");
        assert_eq!(geometry_count(&a), 0);
        assert_eq!(a.id, "id1");
        assert_eq!(a.name, "name1");
    }

    #[test]
    fn add_geometry_increments_count() {
        let a = new_collada_asset("id1", "name1");
        let a = add_geometry(a, sample_geom());
        assert_eq!(geometry_count(&a), 1);
        let g2 = ColladaGeometry {
            id: "mesh1".to_string(),
            positions: vec![0.0, 0.0, 0.0],
            indices: vec![],
            normals: None,
            uvs: None,
        };
        let a = add_geometry(a, g2);
        assert_eq!(geometry_count(&a), 2);
    }

    #[test]
    fn collada_to_xml_stub_contains_header() {
        let a = sample_asset();
        let cfg = default_collada_config();
        let xml = collada_to_xml_stub(&a, &cfg).expect("should succeed");
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("COLLADA"));
    }

    #[test]
    fn collada_to_xml_stub_contains_asset_id() {
        let a = sample_asset();
        let cfg = default_collada_config();
        let xml = collada_to_xml_stub(&a, &cfg).expect("should succeed");
        assert!(xml.contains("mesh0"));
    }

    #[test]
    fn collada_to_xml_stub_empty_id_errors() {
        let a = ColladaAsset {
            id: String::new(),
            name: "test".to_string(),
            author: "A".to_string(),
            up_axis: "Y_UP".to_string(),
            geometries: vec![],
        };
        let cfg = default_collada_config();
        assert!(collada_to_xml_stub(&a, &cfg).is_err());
    }

    #[test]
    fn validate_ok_for_valid_asset() {
        let a = sample_asset();
        assert!(validate_collada_asset(&a).is_ok());
    }

    #[test]
    fn validate_fails_empty_id() {
        let a = ColladaAsset {
            id: String::new(),
            name: "n".to_string(),
            author: "A".to_string(),
            up_axis: "Y_UP".to_string(),
            geometries: vec![],
        };
        assert!(validate_collada_asset(&a).is_err());
    }

    #[test]
    fn validate_fails_bad_index_count() {
        let mut a = sample_asset();
        a.geometries[0].indices.push(99); // makes 4 indices — not divisible by 3
        assert!(validate_collada_asset(&a).is_err());
    }

    #[test]
    fn collada_asset_id_returns_id() {
        let a = new_collada_asset("myid", "myname");
        assert_eq!(collada_asset_id(&a), "myid");
    }

    #[test]
    fn set_author_changes_author() {
        let a = new_collada_asset("id", "name");
        let a = set_author(a, "NewAuthor");
        assert_eq!(a.author, "NewAuthor");
    }

    #[test]
    fn set_up_axis_changes_up_axis() {
        let a = new_collada_asset("id", "name");
        let a = set_up_axis(a, "Z_UP");
        assert_eq!(a.up_axis, "Z_UP");
    }

    #[test]
    fn collada_vertex_count_correct() {
        let a = sample_asset();
        // 3 positions → 1 vertex each
        assert_eq!(collada_vertex_count(&a), 3);
    }

    #[test]
    fn collada_face_count_correct() {
        let a = sample_asset();
        assert_eq!(collada_face_count(&a), 1);
    }

    #[test]
    fn file_size_estimate_positive() {
        let a = sample_asset();
        assert!(collada_file_size_estimate(&a) > 0);
    }

    #[test]
    fn xml_contains_up_axis() {
        let a = set_up_axis(sample_asset(), "Z_UP");
        let cfg = default_collada_config();
        let xml = collada_to_xml_stub(&a, &cfg).expect("should succeed");
        assert!(xml.contains("Z_UP"));
    }

    #[test]
    fn xml_contains_author_from_config() {
        let a = sample_asset();
        let mut cfg = default_collada_config();
        cfg.author = "TestAuthor".to_string();
        let xml = collada_to_xml_stub(&a, &cfg).expect("should succeed");
        assert!(xml.contains("TestAuthor"));
    }
}
