// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Rectangular cloth patch with structural + shear constraints.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothPatchConfig {
    pub rows: usize,
    pub cols: usize,
    pub spacing: f32,
    pub compliance: f32,
    pub inv_mass: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothPatch {
    pub positions: Vec<[f32; 3]>,
    pub prev_positions: Vec<[f32; 3]>,
    pub config: ClothPatchConfig,
    pinned: Vec<bool>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothPatchStats {
    pub vertex_count: usize,
    pub constraint_count: usize,
    pub rows: usize,
    pub cols: usize,
}

#[allow(dead_code)]
pub fn default_cloth_patch_config() -> ClothPatchConfig {
    ClothPatchConfig { rows: 4, cols: 4, spacing: 0.1, compliance: 0.0, inv_mass: 1.0 }
}

#[allow(dead_code)]
pub fn new_cloth_patch(config: ClothPatchConfig) -> ClothPatch {
    let rows = config.rows;
    let cols = config.cols;
    let spacing = config.spacing;
    let count = rows * cols;
    let mut positions = Vec::with_capacity(count);
    for r in 0..rows {
        for c in 0..cols {
            positions.push([c as f32 * spacing, 0.0, r as f32 * spacing]);
        }
    }
    let prev_positions = positions.clone();
    let pinned = vec![false; count];
    ClothPatch { positions, prev_positions, config, pinned }
}

#[allow(dead_code)]
pub fn patch_vertex_count(patch: &ClothPatch) -> usize {
    patch.positions.len()
}

#[allow(dead_code)]
pub fn patch_stats(patch: &ClothPatch) -> ClothPatchStats {
    let rows = patch.config.rows;
    let cols = patch.config.cols;
    // structural horizontal + vertical + shear
    let h = rows * (cols.saturating_sub(1));
    let v = cols * (rows.saturating_sub(1));
    let shear = if rows > 1 && cols > 1 { 2 * (rows - 1) * (cols - 1) } else { 0 };
    ClothPatchStats {
        vertex_count: rows * cols,
        constraint_count: h + v + shear,
        rows,
        cols,
    }
}

#[allow(dead_code)]
pub fn patch_step(patch: &mut ClothPatch, dt: f32, gravity: [f32; 3]) {
    let n = patch.positions.len();
    for i in 0..n {
        if patch.pinned[i] || patch.config.inv_mass <= 0.0 {
            continue;
        }
        let [gx, gy, gz] = gravity;
        let g = [gx, gy, gz];
        #[allow(clippy::needless_range_loop)]
        for axis in 0..3 {
            let prev = patch.prev_positions[i][axis];
            let cur = patch.positions[i][axis];
            let new_pos = cur * 2.0 - prev + g[axis] * dt * dt;
            patch.prev_positions[i][axis] = cur;
            patch.positions[i][axis] = new_pos;
        }
    }
}

#[allow(dead_code)]
pub fn patch_pin_vertex(patch: &mut ClothPatch, idx: usize) {
    if idx < patch.pinned.len() {
        patch.pinned[idx] = true;
    }
}

#[allow(dead_code)]
pub fn patch_vertex_pos(patch: &ClothPatch, idx: usize) -> [f32; 3] {
    if idx < patch.positions.len() {
        patch.positions[idx]
    } else {
        [0.0; 3]
    }
}

#[allow(dead_code)]
pub fn patch_to_json(patch: &ClothPatch) -> String {
    format!(
        "{{\"rows\":{},\"cols\":{},\"vertex_count\":{}}}",
        patch.config.rows,
        patch.config.cols,
        patch.positions.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_cloth_patch_config();
        assert_eq!(cfg.rows, 4);
        assert_eq!(cfg.cols, 4);
    }

    #[test]
    fn test_new_patch_vertex_count() {
        let cfg = ClothPatchConfig { rows: 3, cols: 4, spacing: 0.1, compliance: 0.0, inv_mass: 1.0 };
        let patch = new_cloth_patch(cfg);
        assert_eq!(patch_vertex_count(&patch), 12);
    }

    #[test]
    fn test_patch_stats() {
        let cfg = ClothPatchConfig { rows: 2, cols: 3, spacing: 0.1, compliance: 0.0, inv_mass: 1.0 };
        let patch = new_cloth_patch(cfg);
        let stats = patch_stats(&patch);
        assert_eq!(stats.rows, 2);
        assert_eq!(stats.cols, 3);
        assert_eq!(stats.vertex_count, 6);
    }

    #[test]
    fn test_patch_step_moves_vertices() {
        let cfg = default_cloth_patch_config();
        let mut patch = new_cloth_patch(cfg);
        let before_y = patch.positions[5][1];
        patch_step(&mut patch, 0.01, [0.0, -9.81, 0.0]);
        assert!(patch.positions[5][1] < before_y);
    }

    #[test]
    fn test_pin_vertex() {
        let cfg = default_cloth_patch_config();
        let mut patch = new_cloth_patch(cfg);
        patch_pin_vertex(&mut patch, 0);
        let before = patch_vertex_pos(&patch, 0);
        patch_step(&mut patch, 0.1, [0.0, -9.81, 0.0]);
        assert_eq!(patch_vertex_pos(&patch, 0), before);
    }

    #[test]
    fn test_vertex_pos() {
        let cfg = ClothPatchConfig { rows: 2, cols: 2, spacing: 1.0, compliance: 0.0, inv_mass: 1.0 };
        let patch = new_cloth_patch(cfg);
        let pos = patch_vertex_pos(&patch, 1);
        assert!((pos[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_cloth_patch_config();
        let patch = new_cloth_patch(cfg);
        let json = patch_to_json(&patch);
        assert!(json.contains("\"rows\":4"));
    }

    #[test]
    fn test_out_of_bounds_vertex_pos() {
        let cfg = ClothPatchConfig { rows: 2, cols: 2, spacing: 1.0, compliance: 0.0, inv_mass: 1.0 };
        let patch = new_cloth_patch(cfg);
        let pos = patch_vertex_pos(&patch, 999);
        assert_eq!(pos, [0.0; 3]);
    }

    #[test]
    fn test_pin_out_of_bounds() {
        let cfg = ClothPatchConfig { rows: 2, cols: 2, spacing: 1.0, compliance: 0.0, inv_mass: 1.0 };
        let mut patch = new_cloth_patch(cfg);
        patch_pin_vertex(&mut patch, 999); // Should not panic
    }
}
