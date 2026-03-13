// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Sculpt mode overlay: symmetry lines, vertex mask, and multires indicator.
//!
//! All operations work on plain data structures without GPU or windowing
//! dependencies.

#![allow(dead_code)]

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for the sculpt overlay display.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SculptOverlayConfig {
    /// Whether the overlay is visible at all. Default: `true`.
    pub visible: bool,
    /// Whether to show the symmetry plane indicator. Default: `true`.
    pub show_symmetry: bool,
    /// Whether to show the mask overlay. Default: `true`.
    pub show_mask: bool,
    /// Current multires level (0 = base). Default: `0`.
    pub multires_level: u32,
    /// RGBA color for the symmetry line `[r, g, b, a]`. Default: cyan.
    pub symmetry_color: [f32; 4],
    /// RGBA color for the mask overlay. Default: red at 50% alpha.
    pub mask_color: [f32; 4],
}

impl Default for SculptOverlayConfig {
    fn default() -> Self {
        Self {
            visible: true,
            show_symmetry: true,
            show_mask: true,
            multires_level: 0,
            symmetry_color: [0.0, 1.0, 1.0, 1.0],
            mask_color: [1.0, 0.0, 0.0, 0.5],
        }
    }
}

/// Per-vertex sculpt mask (0 = fully unmasked, 1 = fully masked).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SculptMask {
    /// Per-vertex mask values in `[0, 1]`.
    pub values: Vec<f32>,
}

impl SculptMask {
    /// Create a new mask with `n` vertices all set to `0.0` (unmasked).
    #[allow(dead_code)]
    pub fn new(n: usize) -> Self {
        Self { values: vec![0.0; n] }
    }
}

/// The sculpt overlay state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SculptOverlay {
    /// Display configuration.
    pub config: SculptOverlayConfig,
    /// Active symmetry plane: `0` = X, `1` = Y, `2` = Z, `None` = disabled.
    pub symmetry_plane: Option<u8>,
    /// Optional vertex mask.
    pub mask: Option<SculptMask>,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// A list of pairs `(vertex_index_a, vertex_index_b)` representing mask boundary edges.
pub type MaskBoundaryEdges = Vec<(usize, usize)>;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`SculptOverlayConfig`].
#[allow(dead_code)]
pub fn default_sculpt_overlay_config() -> SculptOverlayConfig {
    SculptOverlayConfig::default()
}

/// Create a new [`SculptOverlay`] with the given configuration.
#[allow(dead_code)]
pub fn new_sculpt_overlay(config: SculptOverlayConfig) -> SculptOverlay {
    SculptOverlay {
        config,
        symmetry_plane: None,
        mask: None,
    }
}

/// Set the active symmetry plane axis (`0` = X, `1` = Y, `2` = Z).
#[allow(dead_code)]
pub fn set_symmetry_plane(overlay: &mut SculptOverlay, axis: u8) {
    overlay.symmetry_plane = Some(axis.min(2));
}

/// Mirror `point` across the active symmetry plane.
///
/// Returns the mirrored point, or the original if no symmetry plane is set.
#[allow(dead_code)]
pub fn symmetry_mirror_point(overlay: &SculptOverlay, point: [f32; 3]) -> [f32; 3] {
    match overlay.symmetry_plane {
        Some(0) => [-point[0], point[1], point[2]],
        Some(1) => [point[0], -point[1], point[2]],
        Some(2) => [point[0], point[1], -point[2]],
        _ => point,
    }
}

/// Attach a [`SculptMask`] to `overlay`.
#[allow(dead_code)]
pub fn set_mask(overlay: &mut SculptOverlay, mask: SculptMask) {
    overlay.mask = Some(mask);
}

/// Apply the mask to a `strength` value at vertex `idx`.
///
/// Returns `strength * (1 - mask_value)`.  If no mask is set or `idx` is
/// out of range, returns `strength` unmodified.
#[allow(dead_code)]
pub fn apply_mask_to_strength(overlay: &SculptOverlay, idx: usize, strength: f32) -> f32 {
    match &overlay.mask {
        Some(mask) if idx < mask.values.len() => strength * (1.0 - mask.values[idx]),
        _ => strength,
    }
}

/// Return the number of vertices whose mask value is > 0.5.
#[allow(dead_code)]
pub fn masked_vertex_count(overlay: &SculptOverlay) -> usize {
    match &overlay.mask {
        Some(mask) => mask.values.iter().filter(|&&v| v > 0.5).count(),
        None => 0,
    }
}

/// Serialize the overlay configuration to a compact JSON string.
#[allow(dead_code)]
pub fn sculpt_overlay_to_json(overlay: &SculptOverlay) -> String {
    let sym = match overlay.symmetry_plane {
        Some(a) => format!("{}", a),
        None => "null".to_string(),
    };
    let mask_len = match &overlay.mask {
        Some(m) => m.values.len(),
        None => 0,
    };
    format!(
        r#"{{"visible":{},"show_symmetry":{},"show_mask":{},"multires_level":{},"symmetry_plane":{},"mask_vertices":{}}}"#,
        overlay.config.visible,
        overlay.config.show_symmetry,
        overlay.config.show_mask,
        overlay.config.multires_level,
        sym,
        mask_len,
    )
}

/// Toggle the `show_symmetry` flag.
#[allow(dead_code)]
pub fn toggle_symmetry(overlay: &mut SculptOverlay) {
    overlay.config.show_symmetry = !overlay.config.show_symmetry;
}

/// Toggle the `show_mask` flag.
#[allow(dead_code)]
pub fn toggle_mask_display(overlay: &mut SculptOverlay) {
    overlay.config.show_mask = !overlay.config.show_mask;
}

/// Clear the mask (set to `None`).
#[allow(dead_code)]
pub fn clear_mask(overlay: &mut SculptOverlay) {
    overlay.mask = None;
}

/// Return whether the overlay is visible.
#[allow(dead_code)]
pub fn overlay_visible(overlay: &SculptOverlay) -> bool {
    overlay.config.visible
}

/// Set the multires subdivision level.
#[allow(dead_code)]
pub fn set_multires_level(overlay: &mut SculptOverlay, level: u32) {
    overlay.config.multires_level = level;
}

/// Compute mask boundary edges from `edges`.
///
/// `edges` — slice of `(vertex_a, vertex_b)` pairs.
///
/// Returns pairs where exactly one of the two vertices has mask > 0.5.
#[allow(dead_code)]
pub fn mask_boundary_edges(overlay: &SculptOverlay, edges: &[(usize, usize)]) -> MaskBoundaryEdges {
    let mask_val = |idx: usize| -> f32 {
        match &overlay.mask {
            Some(m) if idx < m.values.len() => m.values[idx],
            _ => 0.0,
        }
    };
    edges
        .iter()
        .filter(|&&(a, b)| {
            let ma = mask_val(a) > 0.5;
            let mb = mask_val(b) > 0.5;
            ma != mb
        })
        .copied()
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_overlay() -> SculptOverlay {
        new_sculpt_overlay(default_sculpt_overlay_config())
    }

    #[test]
    fn test_default_sculpt_overlay_config() {
        let cfg = default_sculpt_overlay_config();
        assert!(cfg.visible);
        assert!(cfg.show_symmetry);
        assert!(cfg.show_mask);
        assert_eq!(cfg.multires_level, 0);
    }

    #[test]
    fn test_new_sculpt_overlay() {
        let overlay = make_overlay();
        assert!(overlay.symmetry_plane.is_none());
        assert!(overlay.mask.is_none());
    }

    #[test]
    fn test_set_symmetry_plane() {
        let mut overlay = make_overlay();
        set_symmetry_plane(&mut overlay, 0);
        assert_eq!(overlay.symmetry_plane, Some(0));
    }

    #[test]
    fn test_set_symmetry_plane_clamps() {
        let mut overlay = make_overlay();
        set_symmetry_plane(&mut overlay, 99);
        assert_eq!(overlay.symmetry_plane, Some(2));
    }

    #[test]
    fn test_symmetry_mirror_x() {
        let mut overlay = make_overlay();
        set_symmetry_plane(&mut overlay, 0);
        let m = symmetry_mirror_point(&overlay, [1.0, 2.0, 3.0]);
        assert!((m[0] + 1.0).abs() < 1e-6);
        assert!((m[1] - 2.0).abs() < 1e-6);
        assert!((m[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_symmetry_mirror_y() {
        let mut overlay = make_overlay();
        set_symmetry_plane(&mut overlay, 1);
        let m = symmetry_mirror_point(&overlay, [1.0, 2.0, 3.0]);
        assert!((m[1] + 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_symmetry_mirror_z() {
        let mut overlay = make_overlay();
        set_symmetry_plane(&mut overlay, 2);
        let m = symmetry_mirror_point(&overlay, [1.0, 2.0, 3.0]);
        assert!((m[2] + 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_symmetry_mirror_none() {
        let overlay = make_overlay();
        let m = symmetry_mirror_point(&overlay, [1.0, 2.0, 3.0]);
        assert_eq!(m, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_set_mask_and_masked_count() {
        let mut overlay = make_overlay();
        let mut mask = SculptMask::new(4);
        mask.values[0] = 1.0;
        mask.values[2] = 0.8;
        set_mask(&mut overlay, mask);
        assert_eq!(masked_vertex_count(&overlay), 2);
    }

    #[test]
    fn test_masked_vertex_count_no_mask() {
        let overlay = make_overlay();
        assert_eq!(masked_vertex_count(&overlay), 0);
    }

    #[test]
    fn test_apply_mask_to_strength() {
        let mut overlay = make_overlay();
        let mut mask = SculptMask::new(3);
        mask.values[0] = 1.0; // fully masked
        mask.values[1] = 0.0; // unmasked
        set_mask(&mut overlay, mask);
        assert!((apply_mask_to_strength(&overlay, 0, 1.0)).abs() < 1e-6);
        assert!((apply_mask_to_strength(&overlay, 1, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_mask_to_strength_no_mask() {
        let overlay = make_overlay();
        assert!((apply_mask_to_strength(&overlay, 0, 0.7) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_toggle_symmetry() {
        let mut overlay = make_overlay();
        assert!(overlay.config.show_symmetry);
        toggle_symmetry(&mut overlay);
        assert!(!overlay.config.show_symmetry);
        toggle_symmetry(&mut overlay);
        assert!(overlay.config.show_symmetry);
    }

    #[test]
    fn test_toggle_mask_display() {
        let mut overlay = make_overlay();
        assert!(overlay.config.show_mask);
        toggle_mask_display(&mut overlay);
        assert!(!overlay.config.show_mask);
    }

    #[test]
    fn test_clear_mask() {
        let mut overlay = make_overlay();
        set_mask(&mut overlay, SculptMask::new(5));
        clear_mask(&mut overlay);
        assert!(overlay.mask.is_none());
    }

    #[test]
    fn test_overlay_visible() {
        let overlay = make_overlay();
        assert!(overlay_visible(&overlay));
    }

    #[test]
    fn test_set_multires_level() {
        let mut overlay = make_overlay();
        set_multires_level(&mut overlay, 3);
        assert_eq!(overlay.config.multires_level, 3);
    }

    #[test]
    fn test_mask_boundary_edges() {
        let mut overlay = make_overlay();
        let mut mask = SculptMask::new(4);
        mask.values[0] = 1.0;
        mask.values[1] = 0.0;
        mask.values[2] = 1.0;
        mask.values[3] = 1.0;
        set_mask(&mut overlay, mask);
        let edges = vec![(0, 1), (1, 2), (2, 3)];
        let boundary = mask_boundary_edges(&overlay, &edges);
        // (0,1): masked vs unmasked -> boundary
        // (1,2): unmasked vs masked -> boundary
        // (2,3): masked vs masked -> not boundary
        assert_eq!(boundary.len(), 2);
    }

    #[test]
    fn test_sculpt_overlay_to_json() {
        let overlay = make_overlay();
        let json = sculpt_overlay_to_json(&overlay);
        assert!(json.contains("visible"));
        assert!(json.contains("multires_level"));
        assert!(json.contains("symmetry_plane"));
    }
}
