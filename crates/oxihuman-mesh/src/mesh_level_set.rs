// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Signed distance field / level set representation for mesh operations.

// ── Structs ──────────────────────────────────────────────────────────────────

/// Configuration for level set construction.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LevelSetConfig {
    pub resolution: u32,
    pub padding: f32,
    pub narrow_band_width: f32,
}

/// A 3-D grid of signed distance values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LevelSetGrid {
    pub data: Vec<f32>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub voxel_size: f32,
}

/// Result of building a level set from mesh geometry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LevelSetResult {
    pub grid: LevelSetGrid,
    pub zero_crossing_count: usize,
    pub min_dist: f32,
    pub max_dist: f32,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Build a default `LevelSetConfig`.
#[allow(dead_code)]
pub fn default_level_set_config() -> LevelSetConfig {
    LevelSetConfig {
        resolution: 32,
        padding: 0.1,
        narrow_band_width: 3.0,
    }
}

/// Allocate a new `LevelSetGrid` filled with `f32::INFINITY`.
#[allow(dead_code)]
pub fn new_level_set_grid(w: u32, h: u32, d: u32, voxel: f32) -> LevelSetGrid {
    let size = (w as usize) * (h as usize) * (d as usize);
    LevelSetGrid {
        data: vec![f32::INFINITY; size],
        width: w,
        height: h,
        depth: d,
        voxel_size: voxel,
    }
}

/// Compute the flat index into `grid.data` for voxel `(x, y, z)`.
#[allow(dead_code)]
pub fn grid_index(grid: &LevelSetGrid, x: u32, y: u32, z: u32) -> usize {
    (x as usize) * (grid.height as usize) * (grid.depth as usize)
        + (y as usize) * (grid.depth as usize)
        + (z as usize)
}

/// Read the value at voxel `(x, y, z)`.
#[allow(dead_code)]
pub fn grid_value_at(grid: &LevelSetGrid, x: u32, y: u32, z: u32) -> f32 {
    let idx = grid_index(grid, x, y, z);
    grid.data[idx]
}

/// Write a value to voxel `(x, y, z)`.
#[allow(dead_code)]
pub fn grid_set_value(grid: &mut LevelSetGrid, x: u32, y: u32, z: u32, val: f32) {
    let idx = grid_index(grid, x, y, z);
    grid.data[idx] = val;
}

/// Compute unsigned distance from point `p` to triangle `(a, b, c)`.
fn point_triangle_dist(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    // Edge vectors
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];

    let dot_ab_ab = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let dot_ab_ac = ab[0] * ac[0] + ab[1] * ac[1] + ab[2] * ac[2];
    let dot_ac_ac = ac[0] * ac[0] + ac[1] * ac[1] + ac[2] * ac[2];
    let dot_ap_ab = ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2];
    let dot_ap_ac = ap[0] * ac[0] + ap[1] * ac[1] + ap[2] * ac[2];

    let denom = (dot_ab_ab * dot_ac_ac - dot_ab_ac * dot_ab_ac).max(1e-20);
    let s = (dot_ac_ac * dot_ap_ab - dot_ab_ac * dot_ap_ac) / denom;
    let t = (dot_ab_ab * dot_ap_ac - dot_ab_ac * dot_ap_ab) / denom;

    let (s, t) = if s < 0.0 {
        (0.0_f32, t.clamp(0.0, 1.0))
    } else if t < 0.0 {
        (s.clamp(0.0, 1.0), 0.0_f32)
    } else if s + t > 1.0 {
        let inv = 1.0 / (s + t);
        (s * inv, t * inv)
    } else {
        (s, t)
    };

    let closest = [
        a[0] + s * ab[0] + t * ac[0],
        a[1] + s * ab[1] + t * ac[1],
        a[2] + s * ab[2] + t * ac[2],
    ];
    let diff = [
        p[0] - closest[0],
        p[1] - closest[1],
        p[2] - closest[2],
    ];
    (diff[0] * diff[0] + diff[1] * diff[1] + diff[2] * diff[2]).sqrt()
}

/// Build a signed distance field from mesh positions and triangles.
///
/// Uses a simplified approach: assign the unsigned distance to the nearest
/// triangle surface for each voxel within `cfg.narrow_band_width` voxels.
/// Values outside the narrow band are set to `narrow_band_width * voxel_size`.
#[allow(dead_code)]
pub fn build_level_set(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    cfg: &LevelSetConfig,
) -> LevelSetResult {
    let res = cfg.resolution;
    if res == 0 || positions.is_empty() {
        let grid = new_level_set_grid(1, 1, 1, 1.0);
        return LevelSetResult {
            grid,
            zero_crossing_count: 0,
            min_dist: 0.0,
            max_dist: 0.0,
        };
    }

    // Compute mesh AABB
    let (mut xmin, mut ymin, mut zmin) = (f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let (mut xmax, mut ymax, mut zmax) = (f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for p in positions {
        xmin = xmin.min(p[0]);
        ymin = ymin.min(p[1]);
        zmin = zmin.min(p[2]);
        xmax = xmax.max(p[0]);
        ymax = ymax.max(p[1]);
        zmax = zmax.max(p[2]);
    }
    let pad = cfg.padding;
    xmin -= pad;
    ymin -= pad;
    zmin -= pad;
    xmax += pad;
    ymax += pad;
    zmax += pad;

    let ext_x = (xmax - xmin).max(1e-6);
    let ext_y = (ymax - ymin).max(1e-6);
    let ext_z = (zmax - zmin).max(1e-6);
    let voxel_size = ext_x.max(ext_y).max(ext_z) / res as f32;

    let w = ((ext_x / voxel_size).ceil() as u32).max(1);
    let h = ((ext_y / voxel_size).ceil() as u32).max(1);
    let d = ((ext_z / voxel_size).ceil() as u32).max(1);

    let mut grid = new_level_set_grid(w, h, d, voxel_size);
    let narrow = cfg.narrow_band_width * voxel_size;

    for xi in 0..w {
        for yi in 0..h {
            for zi in 0..d {
                let px = xmin + (xi as f32 + 0.5) * voxel_size;
                let py = ymin + (yi as f32 + 0.5) * voxel_size;
                let pz = zmin + (zi as f32 + 0.5) * voxel_size;
                let p = [px, py, pz];

                let mut min_d = narrow;
                for tri in triangles {
                    let a = positions[tri[0] as usize];
                    let b = positions[tri[1] as usize];
                    let c = positions[tri[2] as usize];
                    let dist = point_triangle_dist(p, a, b, c);
                    if dist < min_d {
                        min_d = dist;
                    }
                }
                grid_set_value(&mut grid, xi, yi, zi, min_d);
            }
        }
    }

    // Count zero crossings (voxels with distance < voxel_size)
    let zero_crossing_count = grid
        .data
        .iter()
        .filter(|&&v| v < voxel_size)
        .count();
    let min_dist = grid.data.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_dist = grid.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    LevelSetResult {
        grid,
        zero_crossing_count,
        min_dist,
        max_dist,
    }
}

/// Compute the gradient of the level set at voxel `(x, y, z)` using central differences.
#[allow(dead_code)]
pub fn level_set_gradient(grid: &LevelSetGrid, x: u32, y: u32, z: u32) -> [f32; 3] {
    let w = grid.width;
    let h = grid.height;
    let d = grid.depth;
    let inv2 = 1.0 / (2.0 * grid.voxel_size.max(1e-12));

    let sample = |xi: u32, yi: u32, zi: u32| -> f32 {
        if xi >= w || yi >= h || zi >= d {
            return 0.0;
        }
        grid_value_at(grid, xi, yi, zi)
    };

    let xp = if x + 1 < w { sample(x + 1, y, z) } else { sample(x, y, z) };
    let xm = if x > 0 { sample(x - 1, y, z) } else { sample(x, y, z) };
    let yp = if y + 1 < h { sample(x, y + 1, z) } else { sample(x, y, z) };
    let ym = if y > 0 { sample(x, y - 1, z) } else { sample(x, y, z) };
    let zp = if z + 1 < d { sample(x, y, z + 1) } else { sample(x, y, z) };
    let zm = if z > 0 { sample(x, y, z - 1) } else { sample(x, y, z) };

    [(xp - xm) * inv2, (yp - ym) * inv2, (zp - zm) * inv2]
}

/// Count the number of voxels that cross the iso-surface at value `iso`.
#[allow(dead_code)]
pub fn march_cubes_count(grid: &LevelSetGrid, iso: f32) -> usize {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let d = grid.depth as usize;
    let mut count = 0usize;
    for xi in 0..w.saturating_sub(1) {
        for yi in 0..h.saturating_sub(1) {
            for zi in 0..d.saturating_sub(1) {
                // Check cube corners for sign change
                let corners = [
                    grid.data[xi * h * d + yi * d + zi],
                    grid.data[(xi + 1) * h * d + yi * d + zi],
                    grid.data[xi * h * d + (yi + 1) * d + zi],
                    grid.data[(xi + 1) * h * d + (yi + 1) * d + zi],
                    grid.data[xi * h * d + yi * d + (zi + 1)],
                    grid.data[(xi + 1) * h * d + yi * d + (zi + 1)],
                    grid.data[xi * h * d + (yi + 1) * d + (zi + 1)],
                    grid.data[(xi + 1) * h * d + (yi + 1) * d + (zi + 1)],
                ];
                let any_below = corners.iter().any(|&v| v < iso);
                let any_above = corners.iter().any(|&v| v >= iso);
                if any_below && any_above {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Serialize a `LevelSetResult` summary to a JSON string.
#[allow(dead_code)]
pub fn level_set_to_json(r: &LevelSetResult) -> String {
    format!(
        "{{\"voxel_count\":{},\"zero_crossing_count\":{},\"min_dist\":{:.4},\"max_dist\":{:.4}}}",
        grid_voxel_count(&r.grid),
        r.zero_crossing_count,
        r.min_dist,
        r.max_dist,
    )
}

/// Return the total number of voxels in a grid.
#[allow(dead_code)]
pub fn grid_voxel_count(grid: &LevelSetGrid) -> usize {
    (grid.width as usize) * (grid.height as usize) * (grid.depth as usize)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_resolution() {
        let cfg = default_level_set_config();
        assert_eq!(cfg.resolution, 32);
        assert!(cfg.padding > 0.0);
    }

    #[test]
    fn new_grid_has_correct_size() {
        let grid = new_level_set_grid(4, 5, 6, 0.1);
        assert_eq!(grid.data.len(), 4 * 5 * 6);
        assert_eq!(grid.width, 4);
        assert_eq!(grid.height, 5);
        assert_eq!(grid.depth, 6);
    }

    #[test]
    fn grid_index_and_set_value() {
        let mut grid = new_level_set_grid(3, 3, 3, 0.1);
        grid_set_value(&mut grid, 1, 2, 0, 42.0);
        assert!((grid_value_at(&grid, 1, 2, 0) - 42.0).abs() < 1e-6);
    }

    #[test]
    fn grid_voxel_count_correct() {
        let grid = new_level_set_grid(2, 3, 4, 0.1);
        assert_eq!(grid_voxel_count(&grid), 24);
    }

    #[test]
    fn build_level_set_single_triangle() {
        let positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let triangles = vec![[0u32, 1, 2]];
        let cfg = LevelSetConfig {
            resolution: 4,
            padding: 0.1,
            narrow_band_width: 3.0,
        };
        let result = build_level_set(&positions, &triangles, &cfg);
        assert!(!result.grid.data.is_empty());
        assert!(result.min_dist >= 0.0);
    }

    #[test]
    fn level_set_gradient_finite() {
        let mut grid = new_level_set_grid(4, 4, 4, 0.1);
        // Set a linear gradient
        for x in 0..4u32 {
            for y in 0..4u32 {
                for z in 0..4u32 {
                    grid_set_value(&mut grid, x, y, z, x as f32 * 0.1);
                }
            }
        }
        let g = level_set_gradient(&grid, 2, 2, 2);
        assert!(g[0].is_finite() && g[1].is_finite() && g[2].is_finite());
    }

    #[test]
    fn march_cubes_count_empty_grid() {
        let grid = new_level_set_grid(4, 4, 4, 0.1);
        // All infinity, no crossings below iso 1.0
        let count = march_cubes_count(&grid, 1.0);
        assert_eq!(count, 0);
    }

    #[test]
    fn level_set_to_json_has_fields() {
        let grid = new_level_set_grid(2, 2, 2, 0.1);
        let result = LevelSetResult {
            grid,
            zero_crossing_count: 5,
            min_dist: 0.01,
            max_dist: 1.0,
        };
        let json = level_set_to_json(&result);
        assert!(json.contains("voxel_count"));
        assert!(json.contains("zero_crossing_count"));
        assert!(json.contains("min_dist"));
    }

    #[test]
    fn build_level_set_empty_positions() {
        let cfg = default_level_set_config();
        let result = build_level_set(&[], &[], &cfg);
        assert_eq!(result.zero_crossing_count, 0);
    }
}
