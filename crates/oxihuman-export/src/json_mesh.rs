// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Minimal JSON mesh exporter for debugging and tooling.

use anyhow::Result;
use oxihuman_mesh::mesh::MeshBuffers;
use serde_json::{json, Value};

/// Export mesh as a JSON value (positions + normals + uvs + indices).
/// Useful for debugging and lightweight tooling.
pub fn export_json_mesh(mesh: &MeshBuffers) -> Value {
    json!({
        "vertex_count": mesh.positions.len(),
        "face_count": mesh.indices.len() / 3,
        "has_suit": mesh.has_suit,
        "positions": mesh.positions,
        "normals":   mesh.normals,
        "uvs":       mesh.uvs,
        "indices":   mesh.indices,
    })
}

/// Write the JSON mesh to a file (pretty-printed).
pub fn export_json_mesh_to_file(mesh: &MeshBuffers, path: &std::path::Path) -> Result<()> {
    let val = export_json_mesh(mesh);
    let json_str = serde_json::to_string_pretty(&val)?;
    std::fs::write(path, json_str)?;
    Ok(())
}

/// Import positions and indices from a JSON mesh value (round-trip helper).
pub fn import_json_mesh_positions(val: &Value) -> Option<Vec<[f32; 3]>> {
    let arr = val["positions"].as_array()?;
    arr.iter()
        .map(|v| {
            let t = v.as_array()?;
            if t.len() < 3 {
                return None;
            }
            Some([
                t[0].as_f64()? as f32,
                t[1].as_f64()? as f32,
                t[2].as_f64()? as f32,
            ])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn triangle_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            normals: vec![[0.0, 1.0, 0.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: true,
        })
    }

    #[test]
    fn json_mesh_has_expected_fields() {
        let m = triangle_mesh();
        let v = export_json_mesh(&m);
        assert_eq!(v["vertex_count"], 3);
        assert_eq!(v["face_count"], 1);
        assert_eq!(v["has_suit"], true);
    }

    #[test]
    fn json_positions_round_trip() {
        let m = triangle_mesh();
        let v = export_json_mesh(&m);
        let positions = import_json_mesh_positions(&v).unwrap();
        assert_eq!(positions.len(), 3);
        assert!((positions[0][0] - 1.0).abs() < 1e-5);
        assert!((positions[2][2] - 9.0).abs() < 1e-5);
    }

    #[test]
    fn export_json_creates_file() {
        let m = triangle_mesh();
        let path = std::path::PathBuf::from("/tmp/test_oxihuman_mesh.json");
        export_json_mesh_to_file(&m, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("vertex_count"));
        std::fs::remove_file(&path).ok();
    }
}
