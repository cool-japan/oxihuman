// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Multi-LOD GLB export: decimate a mesh at several ratios and write one GLB per level.

use std::path::{Path, PathBuf};

use anyhow::Result;
use oxihuman_mesh::{decimate_ratio, MeshBuffers};

use crate::glb::export_glb;

/// Configuration for a single LOD level.
#[derive(Debug, Clone)]
pub struct LodLevel {
    /// Target ratio of faces to keep (0.0..=1.0). 1.0 = full resolution.
    pub ratio: f32,
    /// Suffix for the output filename, e.g. `"_lod0"`, `"_lod1"`, `"_lod2"`.
    pub suffix: String,
}

impl LodLevel {
    /// Create a new [`LodLevel`].
    pub fn new(ratio: f32, suffix: impl Into<String>) -> Self {
        Self {
            ratio,
            suffix: suffix.into(),
        }
    }
}

/// Default LOD levels: LOD0 (1.0), LOD1 (0.5), LOD2 (0.25).
pub fn default_lod_levels() -> Vec<LodLevel> {
    vec![
        LodLevel::new(1.0, "_lod0"),
        LodLevel::new(0.5, "_lod1"),
        LodLevel::new(0.25, "_lod2"),
    ]
}

/// Export a mesh at multiple LOD levels, writing GLB files to `output_dir`.
///
/// For each level, decimates the mesh to the target ratio (using
/// [`oxihuman_mesh::decimate_ratio`]) and exports as a GLB file named
/// `base_name + level.suffix + ".glb"`.
///
/// Returns the list of output file paths.
pub fn export_lod_pack(
    mesh: &MeshBuffers,
    base_name: &str,
    output_dir: &Path,
    levels: &[LodLevel],
) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(output_dir)?;
    let mut paths = Vec::with_capacity(levels.len());

    for level in levels {
        let decimated = decimate_ratio(mesh, level.ratio);
        let filename = format!("{}{}.glb", base_name, level.suffix);
        let out_path = output_dir.join(&filename);
        export_glb(&decimated, &out_path)?;
        paths.push(out_path);
    }

    Ok(paths)
}

/// Convenience wrapper: export with the default 3 LOD levels.
pub fn export_default_lod_pack(
    mesh: &MeshBuffers,
    base_name: &str,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    export_lod_pack(mesh, base_name, output_dir, &default_lod_levels())
}

// ── Stats ─────────────────────────────────────────────────────────────────────

/// Per-level statistics collected after export.
#[derive(Debug, Clone)]
pub struct LodLevelStats {
    pub suffix: String,
    pub ratio: f32,
    pub vertex_count: usize,
    pub face_count: usize,
    pub file_size_bytes: u64,
}

/// Statistics about a complete LOD pack export.
#[derive(Debug, Clone)]
pub struct LodPackStats {
    pub levels: Vec<LodLevelStats>,
}

impl LodPackStats {
    /// Sum of file sizes across all LOD levels.
    pub fn total_size_bytes(&self) -> u64 {
        self.levels.iter().map(|l| l.file_size_bytes).sum()
    }

    /// Ratio of the LOD0 (full-res) file size to the last (lowest-res) LOD file size.
    /// Returns 1.0 if fewer than two levels are present.
    pub fn compression_ratio(&self) -> f32 {
        if self.levels.len() < 2 {
            return 1.0;
        }
        let lod0_size = self.levels[0].file_size_bytes;
        let last_size = self.levels.last().map_or(0, |l| l.file_size_bytes);
        if last_size == 0 {
            return f32::INFINITY;
        }
        lod0_size as f32 / last_size as f32
    }
}

/// Export a LOD pack and collect statistics for each level.
pub fn export_lod_pack_with_stats(
    mesh: &MeshBuffers,
    base_name: &str,
    output_dir: &Path,
    levels: &[LodLevel],
) -> Result<LodPackStats> {
    std::fs::create_dir_all(output_dir)?;
    let mut level_stats = Vec::with_capacity(levels.len());

    for level in levels {
        let decimated = decimate_ratio(mesh, level.ratio);
        let filename = format!("{}{}.glb", base_name, level.suffix);
        let out_path = output_dir.join(&filename);
        export_glb(&decimated, &out_path)?;

        let file_size_bytes = std::fs::metadata(&out_path)?.len();
        level_stats.push(LodLevelStats {
            suffix: level.suffix.clone(),
            ratio: level.ratio,
            vertex_count: decimated.positions.len(),
            face_count: decimated.indices.len() / 3,
            file_size_bytes,
        });
    }

    Ok(LodPackStats {
        levels: level_stats,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;

    /// Build a simple suited grid mesh for testing.
    fn suited_grid_mesh() -> MeshBuffers {
        let n = 6usize; // 5x5 grid → 50 faces
        let mut positions = Vec::new();
        for i in 0..n {
            for j in 0..n {
                positions.push([i as f32, 0.0, j as f32]);
            }
        }
        let nv = positions.len();
        let mut normals = Vec::with_capacity(nv);
        let mut uvs = Vec::with_capacity(nv);
        for _ in 0..nv {
            normals.push([0.0f32, 1.0, 0.0]);
            uvs.push([0.0f32, 0.0]);
        }
        let mut indices = Vec::new();
        for i in 0..(n - 1) {
            for j in 0..(n - 1) {
                let b = (i * n + j) as u32;
                indices.extend_from_slice(&[b, b + 1, b + n as u32]);
                indices.extend_from_slice(&[b + 1, b + n as u32 + 1, b + n as u32]);
            }
        }
        MeshBuffers {
            positions,
            normals,
            uvs,
            tangents: vec![],
            colors: None,
            indices,
            has_suit: true,
        }
    }

    fn tmp_dir(name: &str) -> PathBuf {
        let p = PathBuf::from(format!("/tmp/test_lod_export_{}", name));
        std::fs::create_dir_all(&p).expect("should succeed");
        p
    }

    // ── 1 ──────────────────────────────────────────────────────────────────
    #[test]
    fn default_lod_levels_has_three_entries() {
        let levels = default_lod_levels();
        assert_eq!(levels.len(), 3, "expected 3 default LOD levels");
    }

    // ── 2 ──────────────────────────────────────────────────────────────────
    #[test]
    fn lod_level_new_fields() {
        let l = LodLevel::new(0.75, "_lod_test");
        assert!((l.ratio - 0.75).abs() < f32::EPSILON);
        assert_eq!(l.suffix, "_lod_test");
    }

    // ── 3 ──────────────────────────────────────────────────────────────────
    #[test]
    fn export_lod_pack_creates_files() {
        let mesh = suited_grid_mesh();
        let dir = tmp_dir("creates_files");
        let paths = export_lod_pack(&mesh, "human", &dir, &default_lod_levels()).expect("should succeed");
        for p in &paths {
            assert!(p.exists(), "expected file to exist: {}", p.display());
        }
    }

    // ── 4 ──────────────────────────────────────────────────────────────────
    #[test]
    fn export_lod_pack_returns_correct_count() {
        let mesh = suited_grid_mesh();
        let dir = tmp_dir("correct_count");
        let levels = default_lod_levels();
        let paths = export_lod_pack(&mesh, "human", &dir, &levels).expect("should succeed");
        assert_eq!(paths.len(), levels.len());
    }

    // ── 5 ──────────────────────────────────────────────────────────────────
    #[test]
    fn export_lod_pack_files_are_valid_glb() {
        let mesh = suited_grid_mesh();
        let dir = tmp_dir("valid_glb");
        let paths = export_lod_pack(&mesh, "human", &dir, &default_lod_levels()).expect("should succeed");
        // GLB magic: 0x46546C67 in little-endian bytes "glTF"
        let magic: [u8; 4] = [0x67, 0x6C, 0x54, 0x46];
        for p in &paths {
            let data = std::fs::read(p).expect("should succeed");
            assert!(data.len() >= 4, "file too short: {}", p.display());
            assert_eq!(&data[0..4], &magic, "bad GLB magic in {}", p.display());
        }
    }

    // ── 6 ──────────────────────────────────────────────────────────────────
    #[test]
    fn export_default_lod_pack_creates_three_files() {
        let mesh = suited_grid_mesh();
        let dir = tmp_dir("default_three");
        let paths = export_default_lod_pack(&mesh, "human", &dir).expect("should succeed");
        assert_eq!(paths.len(), 3);
        for p in &paths {
            assert!(p.exists());
        }
    }

    // ── 7 ──────────────────────────────────────────────────────────────────
    #[test]
    fn lod_level_1_0_full_resolution() {
        let mesh = suited_grid_mesh();
        let full_faces = mesh.indices.len() / 3;
        let decimated = decimate_ratio(&mesh, 1.0);
        assert_eq!(
            decimated.indices.len() / 3,
            full_faces,
            "ratio=1.0 should keep all faces"
        );
    }

    // ── 8 ──────────────────────────────────────────────────────────────────
    #[test]
    fn export_lod_pack_with_stats_level_count_matches() {
        let mesh = suited_grid_mesh();
        let dir = tmp_dir("stats_count");
        let levels = default_lod_levels();
        let stats = export_lod_pack_with_stats(&mesh, "human", &dir, &levels).expect("should succeed");
        assert_eq!(stats.levels.len(), levels.len());
    }

    // ── 9 ──────────────────────────────────────────────────────────────────
    #[test]
    fn stats_total_size_positive() {
        let mesh = suited_grid_mesh();
        let dir = tmp_dir("total_size");
        let stats =
            export_lod_pack_with_stats(&mesh, "human", &dir, &default_lod_levels()).expect("should succeed");
        assert!(stats.total_size_bytes() > 0, "total size must be > 0");
    }

    // ── 10 ─────────────────────────────────────────────────────────────────
    #[test]
    fn lower_lod_file_size_not_larger_than_full_res() {
        let mesh = suited_grid_mesh();
        let dir = tmp_dir("size_ordering");
        let stats =
            export_lod_pack_with_stats(&mesh, "human", &dir, &default_lod_levels()).expect("should succeed");
        // LOD0 should be >= LOD2 in file size
        let lod0_size = stats.levels[0].file_size_bytes;
        let lod2_size = stats.levels[2].file_size_bytes;
        assert!(
            lod0_size >= lod2_size,
            "LOD0 size ({}) should be >= LOD2 size ({})",
            lod0_size,
            lod2_size
        );
    }
}
