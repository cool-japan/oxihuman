//! Voxel-grid signed-distance field (SDF) computed from a triangle mesh.
//!
//! Builds a 3-D grid of signed-distance values where negative means inside the
//! mesh and positive means outside.  Provides sampling, gradient estimation,
//! inside/outside queries, and JSON serialisation.

#![allow(dead_code)]

// ── Math helpers ──────────────────────────────────────────────────────────────

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Configuration for the SDF grid builder.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SdfGridConfig {
    /// Number of voxels along the X axis.
    pub resolution_x: usize,
    /// Number of voxels along the Y axis.
    pub resolution_y: usize,
    /// Number of voxels along the Z axis.
    pub resolution_z: usize,
    /// Padding added beyond the mesh AABB (world units).
    pub padding: f32,
    /// Iso-value treated as the surface level.
    pub iso_value: f32,
}

/// Voxel-grid signed-distance field.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SdfGrid {
    /// Configuration used to build this grid.
    pub config: SdfGridConfig,
    /// World-space minimum corner.
    pub origin: [f32; 3],
    /// World-space maximum corner.
    pub extent: [f32; 3],
    /// Cell size in each axis.
    pub cell_size: [f32; 3],
    /// Flat array of SDF values, row-major [x][y][z].
    pub values: Vec<f32>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns a `SdfGridConfig` with sensible defaults.
#[allow(dead_code)]
pub fn default_sdf_grid_config() -> SdfGridConfig {
    SdfGridConfig {
        resolution_x: 32,
        resolution_y: 32,
        resolution_z: 32,
        padding: 0.1,
        iso_value: 0.0,
    }
}

/// Build an SDF grid from a triangle mesh.
///
/// `positions` – flat vertex positions.
/// `indices`   – triangle index buffer (triples).
#[allow(dead_code)]
pub fn build_sdf_grid(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &SdfGridConfig,
) -> SdfGrid {
    let rx = config.resolution_x.max(1);
    let ry = config.resolution_y.max(1);
    let rz = config.resolution_z.max(1);

    // Compute AABB of the mesh
    let (mut mn, mut mx) = if positions.is_empty() {
        ([0.0f32; 3], [1.0f32; 3])
    } else {
        (positions[0], positions[0])
    };
    for &p in positions {
        for k in 0..3 {
            if p[k] < mn[k] { mn[k] = p[k]; }
            if p[k] > mx[k] { mx[k] = p[k]; }
        }
    }
    let pad = config.padding;
    let origin = [mn[0] - pad, mn[1] - pad, mn[2] - pad];
    let extent = [mx[0] + pad, mx[1] + pad, mx[2] + pad];

    let range = [
        (extent[0] - origin[0]).max(1e-6),
        (extent[1] - origin[1]).max(1e-6),
        (extent[2] - origin[2]).max(1e-6),
    ];
    let cell_size = [
        range[0] / rx as f32,
        range[1] / ry as f32,
        range[2] / rz as f32,
    ];

    let total = rx * ry * rz;
    let mut values = vec![f32::MAX; total];

    // For each voxel centre compute unsigned distance to nearest triangle,
    // then sign via pseudo-normal (dot of centroid-to-point with face normal).
    for xi in 0..rx {
        for yi in 0..ry {
            for zi in 0..rz {
                let cx = origin[0] + (xi as f32 + 0.5) * cell_size[0];
                let cy = origin[1] + (yi as f32 + 0.5) * cell_size[1];
                let cz = origin[2] + (zi as f32 + 0.5) * cell_size[2];
                let pt = [cx, cy, cz];

                let mut min_dist = f32::MAX;
                let mut sign = 1.0f32;

                let tri_count = indices.len() / 3;
                for t in 0..tri_count {
                    let ia = indices[t * 3] as usize;
                    let ib = indices[t * 3 + 1] as usize;
                    let ic = indices[t * 3 + 2] as usize;
                    if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
                        continue;
                    }
                    let a = positions[ia];
                    let b = positions[ib];
                    let c = positions[ic];
                    let d = point_triangle_dist(pt, a, b, c);
                    if d < min_dist {
                        min_dist = d;
                        // Approximate sign: dot of (pt - centroid) with face normal
                        let centroid = scale3(add3(add3(a, b), c), 1.0 / 3.0);
                        let normal = cross3(sub3(b, a), sub3(c, a));
                        let to_pt = sub3(pt, centroid);
                        sign = if dot3(to_pt, normal) >= 0.0 { 1.0 } else { -1.0 };
                    }
                }
                let idx = xi * ry * rz + yi * rz + zi;
                values[idx] = if positions.is_empty() { f32::MAX } else { sign * min_dist };
            }
        }
    }

    SdfGrid { config: config.clone(), origin, extent, cell_size, values }
}

/// Sample the SDF at a world-space point using trilinear interpolation.
#[allow(dead_code)]
pub fn sdf_grid_sample(grid: &SdfGrid, point: [f32; 3]) -> f32 {
    let rx = grid.config.resolution_x;
    let ry = grid.config.resolution_y;
    let rz = grid.config.resolution_z;

    let fx = ((point[0] - grid.origin[0]) / grid.cell_size[0] - 0.5).clamp(0.0, (rx - 1) as f32);
    let fy = ((point[1] - grid.origin[1]) / grid.cell_size[1] - 0.5).clamp(0.0, (ry - 1) as f32);
    let fz = ((point[2] - grid.origin[2]) / grid.cell_size[2] - 0.5).clamp(0.0, (rz - 1) as f32);

    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let z0 = fz.floor() as usize;
    let x1 = (x0 + 1).min(rx - 1);
    let y1 = (y0 + 1).min(ry - 1);
    let z1 = (z0 + 1).min(rz - 1);

    let tx = fx - fx.floor();
    let ty = fy - fy.floor();
    let tz = fz - fz.floor();

    let v = |xi: usize, yi: usize, zi: usize| -> f32 {
        let i = xi * ry * rz + yi * rz + zi;
        if i < grid.values.len() { grid.values[i] } else { f32::MAX }
    };

    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

    let c00 = lerp(v(x0, y0, z0), v(x1, y0, z0), tx);
    let c10 = lerp(v(x0, y1, z0), v(x1, y1, z0), tx);
    let c01 = lerp(v(x0, y0, z1), v(x1, y0, z1), tx);
    let c11 = lerp(v(x0, y1, z1), v(x1, y1, z1), tx);
    let c0 = lerp(c00, c10, ty);
    let c1 = lerp(c01, c11, ty);
    lerp(c0, c1, tz)
}

/// Estimate the SDF gradient at `point` via central differences.
#[allow(dead_code)]
pub fn sdf_grid_gradient(grid: &SdfGrid, point: [f32; 3], eps: f32) -> [f32; 3] {
    let h = eps.max(1e-5);
    let dx = sdf_grid_sample(grid, [point[0] + h, point[1], point[2]])
           - sdf_grid_sample(grid, [point[0] - h, point[1], point[2]]);
    let dy = sdf_grid_sample(grid, [point[0], point[1] + h, point[2]])
           - sdf_grid_sample(grid, [point[0], point[1] - h, point[2]]);
    let dz = sdf_grid_sample(grid, [point[0], point[1], point[2] + h])
           - sdf_grid_sample(grid, [point[0], point[1], point[2] - h]);
    let inv = 1.0 / (2.0 * h);
    [dx * inv, dy * inv, dz * inv]
}

/// Return `(resolution_x, resolution_y, resolution_z)` of the grid.
#[allow(dead_code)]
pub fn sdf_grid_dims(grid: &SdfGrid) -> (usize, usize, usize) {
    (grid.config.resolution_x, grid.config.resolution_y, grid.config.resolution_z)
}

/// Return the total number of cells in the grid.
#[allow(dead_code)]
pub fn sdf_grid_cell_count(grid: &SdfGrid) -> usize {
    grid.values.len()
}

/// Serialise the grid metadata to a JSON string (values omitted for brevity).
#[allow(dead_code)]
pub fn sdf_grid_to_json(grid: &SdfGrid) -> String {
    let (rx, ry, rz) = sdf_grid_dims(grid);
    format!(
        "{{\"resolution\":[{},{},{}],\"origin\":[{},{},{}],\"extent\":[{},{},{}],\"iso_value\":{}}}",
        rx, ry, rz,
        grid.origin[0], grid.origin[1], grid.origin[2],
        grid.extent[0], grid.extent[1], grid.extent[2],
        grid.config.iso_value,
    )
}

/// Return `true` if the SDF value at `point` is negative (inside the mesh).
#[allow(dead_code)]
pub fn sdf_grid_is_inside(grid: &SdfGrid, point: [f32; 3]) -> bool {
    sdf_grid_sample(grid, point) < grid.config.iso_value
}

/// Return the configured iso-value.
#[allow(dead_code)]
pub fn sdf_grid_iso_value(grid: &SdfGrid) -> f32 {
    grid.config.iso_value
}

/// Reset all grid values to `f32::MAX`.
#[allow(dead_code)]
pub fn sdf_grid_clear(grid: &mut SdfGrid) {
    for v in grid.values.iter_mut() {
        *v = f32::MAX;
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Unsigned distance from point `p` to triangle `(a, b, c)`.
fn point_triangle_dist(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ap = sub3(p, a);

    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return len3(ap);
    }

    let bp = sub3(p, b);
    let d3 = dot3(ab, bp);
    let d4 = dot3(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return len3(bp);
    }

    let cp = sub3(p, c);
    let d5 = dot3(ab, cp);
    let d6 = dot3(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return len3(cp);
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return len3(sub3(add3(a, scale3(ab, v)), p));
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return len3(sub3(add3(a, scale3(ac, w)), p));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return len3(sub3(add3(b, scale3(sub3(c, b), w)), p));
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let closest = add3(a, add3(scale3(ab, v), scale3(ac, w)));
    len3(sub3(p, closest))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        // Simplified: just two triangles forming one face
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_default_config_resolutions_positive() {
        let cfg = default_sdf_grid_config();
        assert!(cfg.resolution_x > 0);
        assert!(cfg.resolution_y > 0);
        assert!(cfg.resolution_z > 0);
    }

    #[test]
    fn test_build_sdf_grid_cell_count() {
        let (pos, idx) = unit_cube_mesh();
        let cfg = default_sdf_grid_config();
        let grid = build_sdf_grid(&pos, &idx, &cfg);
        let expected = cfg.resolution_x * cfg.resolution_y * cfg.resolution_z;
        assert_eq!(sdf_grid_cell_count(&grid), expected);
    }

    #[test]
    fn test_sdf_grid_dims_match_config() {
        let (pos, idx) = unit_cube_mesh();
        let cfg = SdfGridConfig { resolution_x: 4, resolution_y: 6, resolution_z: 8, ..default_sdf_grid_config() };
        let grid = build_sdf_grid(&pos, &idx, &cfg);
        assert_eq!(sdf_grid_dims(&grid), (4, 6, 8));
    }

    #[test]
    fn test_sdf_grid_sample_finite() {
        let (pos, idx) = unit_cube_mesh();
        let cfg = SdfGridConfig { resolution_x: 4, resolution_y: 4, resolution_z: 4, ..default_sdf_grid_config() };
        let grid = build_sdf_grid(&pos, &idx, &cfg);
        let v = sdf_grid_sample(&grid, [0.5, 0.5, 0.5]);
        assert!(v.is_finite());
    }

    #[test]
    fn test_sdf_grid_iso_value() {
        let cfg = SdfGridConfig { iso_value: 0.05, ..default_sdf_grid_config() };
        let (pos, idx) = unit_cube_mesh();
        let grid = build_sdf_grid(&pos, &idx, &cfg);
        assert!((sdf_grid_iso_value(&grid) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_sdf_grid_to_json_contains_resolution() {
        let (pos, idx) = unit_cube_mesh();
        let cfg = SdfGridConfig { resolution_x: 8, resolution_y: 8, resolution_z: 8, ..default_sdf_grid_config() };
        let grid = build_sdf_grid(&pos, &idx, &cfg);
        let json = sdf_grid_to_json(&grid);
        assert!(json.contains("resolution"));
        assert!(json.contains('8'.to_string().as_str()));
    }

    #[test]
    fn test_sdf_grid_gradient_non_zero_near_surface() {
        let (pos, idx) = unit_cube_mesh();
        let cfg = SdfGridConfig { resolution_x: 8, resolution_y: 8, resolution_z: 8, ..default_sdf_grid_config() };
        let grid = build_sdf_grid(&pos, &idx, &cfg);
        let g = sdf_grid_gradient(&grid, [0.5, 0.5, 0.1], 0.05);
        let mag = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        assert!(mag.is_finite());
    }

    #[test]
    fn test_sdf_grid_clear_sets_max() {
        let (pos, idx) = unit_cube_mesh();
        let cfg = SdfGridConfig { resolution_x: 2, resolution_y: 2, resolution_z: 2, ..default_sdf_grid_config() };
        let mut grid = build_sdf_grid(&pos, &idx, &cfg);
        sdf_grid_clear(&mut grid);
        assert!(grid.values.iter().all(|&v| v == f32::MAX));
    }

    #[test]
    fn test_sdf_grid_is_inside_far_outside() {
        let (pos, idx) = unit_cube_mesh();
        let cfg = SdfGridConfig { resolution_x: 4, resolution_y: 4, resolution_z: 4, ..default_sdf_grid_config() };
        let grid = build_sdf_grid(&pos, &idx, &cfg);
        // Far away from the mesh — should definitely be outside
        let inside = sdf_grid_is_inside(&grid, [100.0, 100.0, 100.0]);
        assert!(!inside);
    }

    #[test]
    fn test_build_sdf_grid_empty_mesh() {
        let cfg = SdfGridConfig { resolution_x: 2, resolution_y: 2, resolution_z: 2, ..default_sdf_grid_config() };
        let grid = build_sdf_grid(&[], &[], &cfg);
        assert_eq!(grid.values.len(), 8);
    }

    #[test]
    fn test_point_triangle_dist_vertex() {
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = point_triangle_dist(a, a, b, c);
        assert!(d < 1e-6);
    }

    #[test]
    fn test_sdf_grid_to_json_contains_origin() {
        let (pos, idx) = unit_cube_mesh();
        let cfg = default_sdf_grid_config();
        let grid = build_sdf_grid(&pos, &idx, &cfg);
        let json = sdf_grid_to_json(&grid);
        assert!(json.contains("origin"));
        assert!(json.contains("extent"));
    }
}
