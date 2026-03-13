// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Alembic (ABC) export stub for OxiHuman.
//!
//! Provides lightweight data types and helper functions for describing an
//! Alembic scene and estimating export parameters.  Actual binary ABC
//! serialisation is left as a future integration task.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for an Alembic export operation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AlembicConfig {
    /// Frames per second for the exported archive.
    pub frame_rate: f32,
    /// First frame of the export range (inclusive).
    pub start_frame: u32,
    /// Last frame of the export range (inclusive).
    pub end_frame: u32,
    /// Whether to include per-vertex normals in each sample.
    pub include_normals: bool,
}

/// A single mesh object within an Alembic scene.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AlembicMesh {
    /// Object name / path component in the Alembic hierarchy.
    pub name: String,
    /// Number of vertices in this mesh.
    pub vertex_count: usize,
    /// Number of polygons (faces) in this mesh.
    pub poly_count: usize,
}

/// A collection of meshes that form one Alembic archive.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AlembicScene {
    /// Meshes contained in the scene.
    pub meshes: Vec<AlembicMesh>,
    /// Total number of frames in the archive.
    pub frame_count: u32,
}

/// Result produced by [`alembic_export_stub`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlembicExportResult {
    /// The scene that was (stub-)exported.
    pub scene: AlembicScene,
    /// Rough byte estimate for the resulting archive.
    pub byte_estimate: usize,
    /// `true` when the stub ran without error.
    pub success: bool,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a sensible default [`AlembicConfig`].
#[allow(dead_code)]
pub fn default_alembic_config() -> AlembicConfig {
    AlembicConfig {
        frame_rate: 24.0,
        start_frame: 1,
        end_frame: 24,
        include_normals: true,
    }
}

/// Construct a new [`AlembicMesh`] with the given name and topology counts.
#[allow(dead_code)]
pub fn new_alembic_mesh(name: &str, vcount: usize, pcount: usize) -> AlembicMesh {
    AlembicMesh {
        name: name.to_string(),
        vertex_count: vcount,
        poly_count: pcount,
    }
}

/// Construct an empty [`AlembicScene`].
#[allow(dead_code)]
pub fn new_alembic_scene() -> AlembicScene {
    AlembicScene::default()
}

/// Append `mesh` to `scene`.
#[allow(dead_code)]
pub fn alembic_scene_add_mesh(scene: &mut AlembicScene, mesh: AlembicMesh) {
    scene.meshes.push(mesh);
}

/// Stub export — validates parameters and returns an estimated result.
///
/// No actual file I/O is performed.
#[allow(dead_code)]
pub fn alembic_export_stub(scene: &AlembicScene, cfg: &AlembicConfig) -> AlembicExportResult {
    let frames = alembic_frame_count(cfg) as usize;
    // Very rough heuristic: ~48 bytes per vertex per frame for positions + normals
    let bytes_per_vert = if cfg.include_normals { 48 } else { 24 };
    let total_verts: usize = scene.meshes.iter().map(|m| m.vertex_count).sum();
    let byte_estimate = total_verts * bytes_per_vert * frames.max(1);

    AlembicExportResult {
        scene: scene.clone(),
        byte_estimate,
        success: cfg.end_frame >= cfg.start_frame,
    }
}

/// Serialise an [`AlembicMesh`] to a compact JSON string.
#[allow(dead_code)]
pub fn alembic_mesh_to_json(mesh: &AlembicMesh) -> String {
    format!(
        r#"{{"name":"{}","vertex_count":{},"poly_count":{}}}"#,
        mesh.name, mesh.vertex_count, mesh.poly_count
    )
}

/// Serialise an [`AlembicExportResult`] to a compact JSON string.
#[allow(dead_code)]
pub fn alembic_result_to_json(r: &AlembicExportResult) -> String {
    let mesh_jsons: Vec<String> = r.scene.meshes.iter().map(alembic_mesh_to_json).collect();
    format!(
        r#"{{"mesh_count":{},"frame_count":{},"byte_estimate":{},"success":{}}}"#,
        mesh_jsons.len(),
        r.scene.frame_count,
        r.byte_estimate,
        r.success,
    )
}

/// Return the number of frames in the export range (`end_frame - start_frame + 1`).
///
/// Returns `0` if `end_frame < start_frame`.
#[allow(dead_code)]
pub fn alembic_frame_count(cfg: &AlembicConfig) -> u32 {
    cfg.end_frame.saturating_sub(cfg.start_frame) + 1
}

/// Return the number of meshes in `scene`.
#[allow(dead_code)]
pub fn alembic_scene_mesh_count(scene: &AlembicScene) -> usize {
    scene.meshes.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_fields() {
        let cfg = default_alembic_config();
        assert!((cfg.frame_rate - 24.0).abs() < f32::EPSILON);
        assert_eq!(cfg.start_frame, 1);
        assert_eq!(cfg.end_frame, 24);
        assert!(cfg.include_normals);
    }

    #[test]
    fn frame_count_normal() {
        let cfg = default_alembic_config(); // 1..=24
        assert_eq!(alembic_frame_count(&cfg), 24);
    }

    #[test]
    fn frame_count_single_frame() {
        let cfg = AlembicConfig {
            frame_rate: 30.0,
            start_frame: 5,
            end_frame: 5,
            include_normals: false,
        };
        assert_eq!(alembic_frame_count(&cfg), 1);
    }

    #[test]
    fn frame_count_inverted_range_saturates() {
        let cfg = AlembicConfig {
            frame_rate: 30.0,
            start_frame: 10,
            end_frame: 5,
            include_normals: false,
        };
        // saturating_sub(10,5) + 1 = 0 + 1 = 1, but end < start so success=false
        let result = alembic_export_stub(
            &new_alembic_scene(),
            &cfg,
        );
        assert!(!result.success);
    }

    #[test]
    fn new_mesh_fields() {
        let m = new_alembic_mesh("body", 10_000, 8_000);
        assert_eq!(m.name, "body");
        assert_eq!(m.vertex_count, 10_000);
        assert_eq!(m.poly_count, 8_000);
    }

    #[test]
    fn scene_add_mesh() {
        let mut scene = new_alembic_scene();
        alembic_scene_add_mesh(&mut scene, new_alembic_mesh("a", 100, 50));
        alembic_scene_add_mesh(&mut scene, new_alembic_mesh("b", 200, 80));
        assert_eq!(alembic_scene_mesh_count(&scene), 2);
    }

    #[test]
    fn export_stub_success() {
        let cfg = default_alembic_config();
        let mut scene = new_alembic_scene();
        alembic_scene_add_mesh(&mut scene, new_alembic_mesh("mesh", 1_000, 500));
        let result = alembic_export_stub(&scene, &cfg);
        assert!(result.success);
        assert!(result.byte_estimate > 0);
    }

    #[test]
    fn export_stub_byte_estimate_no_normals() {
        let cfg = AlembicConfig {
            frame_rate: 24.0,
            start_frame: 1,
            end_frame: 1,
            include_normals: false,
        };
        let mut scene = new_alembic_scene();
        alembic_scene_add_mesh(&mut scene, new_alembic_mesh("m", 100, 50));
        let r = alembic_export_stub(&scene, &cfg);
        // 100 verts * 24 bytes * 1 frame
        assert_eq!(r.byte_estimate, 2_400);
    }

    #[test]
    fn mesh_to_json_contains_name() {
        let m = new_alembic_mesh("torso", 500, 400);
        let json = alembic_mesh_to_json(&m);
        assert!(json.contains("torso"));
        assert!(json.contains("500"));
        assert!(json.contains("400"));
    }

    #[test]
    fn result_to_json_contains_success() {
        let cfg = default_alembic_config();
        let scene = new_alembic_scene();
        let r = alembic_export_stub(&scene, &cfg);
        let json = alembic_result_to_json(&r);
        assert!(json.contains("success"));
        assert!(json.contains("byte_estimate"));
    }

    #[test]
    fn scene_mesh_count_empty() {
        let scene = new_alembic_scene();
        assert_eq!(alembic_scene_mesh_count(&scene), 0);
    }
}
