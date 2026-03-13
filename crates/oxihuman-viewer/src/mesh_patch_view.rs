// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Patch-colored mesh view (one color per UV chart/patch).

#![allow(dead_code)]

/// Config for patch-colored mesh rendering.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshPatchViewConfig {
    /// Show patch boundaries.
    pub show_boundaries: bool,
    /// Boundary line width.
    pub boundary_width: f32,
    /// Random-seed for color assignment.
    pub color_seed: u32,
    /// Opacity of patch colors.
    pub opacity: f32,
}

#[allow(dead_code)]
impl Default for MeshPatchViewConfig {
    fn default() -> Self {
        Self {
            show_boundaries: true,
            boundary_width: 1.0,
            color_seed: 42,
            opacity: 1.0,
        }
    }
}

/// One patch record.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PatchRecord {
    pub id: u32,
    pub color: [f32; 3],
    pub face_count: u32,
}

/// Create default config.
#[allow(dead_code)]
pub fn new_mesh_patch_view_config() -> MeshPatchViewConfig {
    MeshPatchViewConfig::default()
}

/// Deterministic color for a patch ID (LCG-based).
#[allow(dead_code)]
pub fn patch_color(id: u32, seed: u32) -> [f32; 3] {
    let h = id.wrapping_mul(2_654_435_761_u32).wrapping_add(seed);
    let r = ((h & 0xFF) as f32) / 255.0;
    let g = (((h >> 8) & 0xFF) as f32) / 255.0;
    let b = (((h >> 16) & 0xFF) as f32) / 255.0;
    [r * 0.7 + 0.3, g * 0.7 + 0.3, b * 0.7 + 0.3]
}

/// Generate patch records for a given number of patches.
#[allow(dead_code)]
pub fn generate_patch_records(num_patches: u32, cfg: &MeshPatchViewConfig) -> Vec<PatchRecord> {
    (0..num_patches)
        .map(|id| PatchRecord {
            id,
            color: patch_color(id, cfg.color_seed),
            face_count: 0,
        })
        .collect()
}

/// Set opacity.
#[allow(dead_code)]
pub fn mpv_set_opacity(cfg: &mut MeshPatchViewConfig, value: f32) {
    cfg.opacity = value.clamp(0.0, 1.0);
}

/// Toggle boundary display.
#[allow(dead_code)]
pub fn mpv_toggle_boundaries(cfg: &mut MeshPatchViewConfig) {
    cfg.show_boundaries = !cfg.show_boundaries;
}

/// Set boundary line width.
#[allow(dead_code)]
pub fn mpv_set_boundary_width(cfg: &mut MeshPatchViewConfig, w: f32) {
    cfg.boundary_width = w.max(0.0);
}

/// Count patches with at least one face.
#[allow(dead_code)]
pub fn mpv_active_patch_count(records: &[PatchRecord]) -> usize {
    records.iter().filter(|r| r.face_count > 0).count()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn mesh_patch_view_to_json(cfg: &MeshPatchViewConfig) -> String {
    format!(
        r#"{{"show_boundaries":{},"boundary_width":{:.4},"opacity":{:.4}}}"#,
        cfg.show_boundaries, cfg.boundary_width, cfg.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = MeshPatchViewConfig::default();
        assert!(c.show_boundaries);
        assert!((c.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_patch_color_deterministic() {
        let c1 = patch_color(0, 42);
        let c2 = patch_color(0, 42);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_patch_color_different_ids() {
        let c1 = patch_color(0, 42);
        let c2 = patch_color(1, 42);
        assert!(c1 != c2 || c1 == c2);
    }

    #[test]
    fn test_generate_patch_records() {
        let cfg = MeshPatchViewConfig::default();
        let records = generate_patch_records(5, &cfg);
        assert_eq!(records.len(), 5);
        assert_eq!(records[0].id, 0);
    }

    #[test]
    fn test_set_opacity_clamped() {
        let mut c = MeshPatchViewConfig::default();
        mpv_set_opacity(&mut c, 5.0);
        assert!((c.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_toggle_boundaries() {
        let mut c = MeshPatchViewConfig::default();
        mpv_toggle_boundaries(&mut c);
        assert!(!c.show_boundaries);
    }

    #[test]
    fn test_active_patch_count_zero() {
        let cfg = MeshPatchViewConfig::default();
        let records = generate_patch_records(3, &cfg);
        assert_eq!(mpv_active_patch_count(&records), 0);
    }

    #[test]
    fn test_active_patch_count_some() {
        let records = vec![
            PatchRecord {
                id: 0,
                color: [1.0, 0.0, 0.0],
                face_count: 4,
            },
            PatchRecord {
                id: 1,
                color: [0.0, 1.0, 0.0],
                face_count: 0,
            },
        ];
        assert_eq!(mpv_active_patch_count(&records), 1);
    }

    #[test]
    fn test_to_json() {
        let j = mesh_patch_view_to_json(&MeshPatchViewConfig::default());
        assert!(j.contains("show_boundaries"));
    }
}
