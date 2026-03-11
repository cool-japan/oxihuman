// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::marching_cubes::{marching_cubes, ScalarField};
use crate::mesh::MeshBuffers;

// ── Public types ──────────────────────────────────────────────────────────────

/// A 3-D grid of signed (or unsigned) distance values.
///
/// Data is stored row-major: `data[iz * ny * nx + iy * nx + ix]`.
pub struct SdfGrid {
    /// Distance values, row-major `[z][y][x]`.
    pub data: Vec<f32>,
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    /// World-space origin of the first grid cell.
    pub origin: [f32; 3],
    /// Uniform cell size (same on all axes).
    pub cell_size: f32,
}

/// Parameters for `compute_sdf`.
pub struct SdfParams {
    /// Number of grid cells per axis (default 32).
    pub resolution: usize,
    /// Extra padding beyond mesh bounds in world units (default 0.1).
    pub padding: f32,
    /// If `true` compute signed distance (inside = negative), else unsigned.
    pub use_sign: bool,
}

impl Default for SdfParams {
    fn default() -> Self {
        SdfParams {
            resolution: 32,
            padding: 0.1,
            use_sign: true,
        }
    }
}

/// Summary statistics for an `SdfGrid`.
pub struct SdfStats {
    pub min_dist: f32,
    pub max_dist: f32,
    pub mean_dist: f32,
    /// Fraction of grid cells with a negative distance value (inside).
    pub negative_fraction: f32,
}

// ── Internal math helpers ─────────────────────────────────────────────────────

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
    dot3(v, v).sqrt()
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Return the closest point on triangle (a, b, c) to point p.
pub fn closest_point_on_triangle(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ap = sub3(p, a);

    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = sub3(p, b);
    let d3 = dot3(ab, bp);
    let d4 = dot3(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return add3(a, scale3(ab, v));
    }

    let cp = sub3(p, c);
    let d5 = dot3(ab, cp);
    let d6 = dot3(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return add3(a, scale3(ac, w));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return add3(b, scale3(sub3(c, b), w));
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    add3(add3(a, scale3(ab, v)), scale3(ac, w))
}

/// Check whether barycentric coordinates (u, v, w) lie inside the triangle.
pub fn point_in_triangle_bary(u: f32, v: f32, w: f32) -> bool {
    u >= 0.0 && v >= 0.0 && w >= 0.0
}

/// Smooth minimum (Inigo Quilez formulation).  `k` controls blend radius.
pub fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 {
        return a.min(b);
    }
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    let lerp = b * (1.0 - h) + a * h;
    lerp - k * h * (1.0 - h)
}

// ── SdfGrid helpers ───────────────────────────────────────────────────────────

impl SdfGrid {
    /// Create a new grid filled with `fill_value`.
    pub fn new(nx: usize, ny: usize, nz: usize, origin: [f32; 3], cell_size: f32) -> Self {
        SdfGrid {
            data: vec![f32::MAX; nx * ny * nz],
            nx,
            ny,
            nz,
            origin,
            cell_size,
        }
    }

    /// Linear index into `data` for grid cell (ix, iy, iz).
    #[inline]
    pub fn index(&self, ix: usize, iy: usize, iz: usize) -> usize {
        iz * self.ny * self.nx + iy * self.nx + ix
    }

    /// World-space centre of grid cell (ix, iy, iz).
    #[inline]
    pub fn cell_centre(&self, ix: usize, iy: usize, iz: usize) -> [f32; 3] {
        [
            self.origin[0] + (ix as f32 + 0.5) * self.cell_size,
            self.origin[1] + (iy as f32 + 0.5) * self.cell_size,
            self.origin[2] + (iz as f32 + 0.5) * self.cell_size,
        ]
    }

    /// Check that two grids have the same layout (origin, size, resolution).
    fn same_layout(&self, other: &SdfGrid) -> bool {
        self.nx == other.nx
            && self.ny == other.ny
            && self.nz == other.nz
            && (self.origin[0] - other.origin[0]).abs() < 1e-5
            && (self.origin[1] - other.origin[1]).abs() < 1e-5
            && (self.origin[2] - other.origin[2]).abs() < 1e-5
            && (self.cell_size - other.cell_size).abs() < 1e-7
    }
}

// ── Core: compute_sdf ─────────────────────────────────────────────────────────

/// Build a signed (or unsigned) distance field from a triangle mesh.
///
/// Uses brute-force O(n × m) point-to-triangle distance.  For sign, a +X ray
/// cast counts triangle intersections; an odd count means the point is inside.
pub fn compute_sdf(mesh: &MeshBuffers, params: &SdfParams) -> SdfGrid {
    // 1. Compute AABB with padding.
    let (mut mn, mut mx) = aabb(mesh);
    for i in 0..3 {
        mn[i] -= params.padding;
        mx[i] += params.padding;
    }

    let size = [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]];
    let cell_size = size[0].max(size[1]).max(size[2]).max(1e-6) / params.resolution as f32;

    let nx = ((size[0] / cell_size).ceil() as usize).max(1);
    let ny = ((size[1] / cell_size).ceil() as usize).max(1);
    let nz = ((size[2] / cell_size).ceil() as usize).max(1);

    let mut grid = SdfGrid::new(nx, ny, nz, mn, cell_size);

    // Gather triangles.
    let tris: Vec<([f32; 3], [f32; 3], [f32; 3])> = mesh
        .indices
        .chunks_exact(3)
        .map(|tri| {
            let a = mesh.positions[tri[0] as usize];
            let b = mesh.positions[tri[1] as usize];
            let c = mesh.positions[tri[2] as usize];
            (a, b, c)
        })
        .collect();

    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let p = grid.cell_centre(ix, iy, iz);
                let mut min_dist_sq = f32::MAX;

                for &(a, b, c) in &tris {
                    let closest = closest_point_on_triangle(p, a, b, c);
                    let d = sub3(p, closest);
                    let dsq = dot3(d, d);
                    if dsq < min_dist_sq {
                        min_dist_sq = dsq;
                    }
                }

                let mut dist = min_dist_sq.sqrt();

                if params.use_sign && !tris.is_empty() && is_inside_ray_cast(p, &tris) {
                    dist = -dist;
                }

                let idx = grid.index(ix, iy, iz);
                grid.data[idx] = dist;
            }
        }
    }

    grid
}

/// Ray-cast along +X from `p`; return true if inside (odd crossing count).
fn is_inside_ray_cast(p: [f32; 3], tris: &[([f32; 3], [f32; 3], [f32; 3])]) -> bool {
    let mut crossings = 0u32;
    for &(a, b, c) in tris {
        if ray_x_triangle_intersect(p, a, b, c) {
            crossings += 1;
        }
    }
    crossings % 2 == 1
}

/// Test whether the +X ray from `orig` intersects triangle (a, b, c).
fn ray_x_triangle_intersect(orig: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> bool {
    // Möller–Trumbore for direction (1, 0, 0).
    let dir = [1.0f32, 0.0, 0.0];
    let edge1 = sub3(b, a);
    let edge2 = sub3(c, a);
    let h = cross3(dir, edge2);
    let det = dot3(edge1, h);
    if det.abs() < 1e-9 {
        return false;
    }
    let inv_det = 1.0 / det;
    let s = sub3(orig, a);
    let u = dot3(s, h) * inv_det;
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    let q = cross3(s, edge1);
    let v = dot3(dir, q) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = dot3(edge2, q) * inv_det;
    t > 0.0
}

fn aabb(mesh: &MeshBuffers) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for p in &mesh.positions {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    // Guard against empty mesh.
    if mesh.positions.is_empty() {
        mn = [-1.0; 3];
        mx = [1.0; 3];
    }
    (mn, mx)
}

// ── sample_sdf ────────────────────────────────────────────────────────────────

/// Trilinearly interpolate the SDF at an arbitrary world-space `point`.
pub fn sample_sdf(grid: &SdfGrid, point: [f32; 3]) -> f32 {
    // Convert to grid-local coordinates.
    let lx = (point[0] - grid.origin[0]) / grid.cell_size - 0.5;
    let ly = (point[1] - grid.origin[1]) / grid.cell_size - 0.5;
    let lz = (point[2] - grid.origin[2]) / grid.cell_size - 0.5;

    let ix = lx.floor() as i32;
    let iy = ly.floor() as i32;
    let iz = lz.floor() as i32;

    let tx = lx - ix as f32;
    let ty = ly - iy as f32;
    let tz = lz - iz as f32;

    let nx = grid.nx as i32;
    let ny = grid.ny as i32;
    let nz = grid.nz as i32;

    let sample = |dx: i32, dy: i32, dz: i32| -> f32 {
        let xi = (ix + dx).clamp(0, nx - 1) as usize;
        let yi = (iy + dy).clamp(0, ny - 1) as usize;
        let zi = (iz + dz).clamp(0, nz - 1) as usize;
        grid.data[grid.index(xi, yi, zi)]
    };

    let c000 = sample(0, 0, 0);
    let c100 = sample(1, 0, 0);
    let c010 = sample(0, 1, 0);
    let c110 = sample(1, 1, 0);
    let c001 = sample(0, 0, 1);
    let c101 = sample(1, 0, 1);
    let c011 = sample(0, 1, 1);
    let c111 = sample(1, 1, 1);

    let c00 = c000 * (1.0 - tx) + c100 * tx;
    let c10 = c010 * (1.0 - tx) + c110 * tx;
    let c01 = c001 * (1.0 - tx) + c101 * tx;
    let c11 = c011 * (1.0 - tx) + c111 * tx;

    let c0 = c00 * (1.0 - ty) + c10 * ty;
    let c1 = c01 * (1.0 - ty) + c11 * ty;

    c0 * (1.0 - tz) + c1 * tz
}

// ── sdf_to_mesh ───────────────────────────────────────────────────────────────

/// Extract an iso-surface mesh from the SDF using Marching Cubes.
pub fn sdf_to_mesh(grid: &SdfGrid, iso_value: f32) -> MeshBuffers {
    // Convert SdfGrid → ScalarField.
    let field = ScalarField {
        data: grid.data.clone(),
        dims: [grid.nx, grid.ny, grid.nz],
        origin: grid.origin,
        spacing: [grid.cell_size; 3],
    };
    marching_cubes(&field, iso_value)
}

// ── Boolean operations ────────────────────────────────────────────────────────

fn check_compatible(a: &SdfGrid, b: &SdfGrid) -> anyhow::Result<()> {
    if !a.same_layout(b) {
        anyhow::bail!(
            "SDF grids are incompatible: layout mismatch \
             (a: {}×{}×{} cell={}, b: {}×{}×{} cell={})",
            a.nx,
            a.ny,
            a.nz,
            a.cell_size,
            b.nx,
            b.ny,
            b.nz,
            b.cell_size,
        );
    }
    Ok(())
}

/// Element-wise minimum (union of two SDFs).
pub fn sdf_union(a: &SdfGrid, b: &SdfGrid) -> anyhow::Result<SdfGrid> {
    check_compatible(a, b)?;
    let data = a
        .data
        .iter()
        .zip(&b.data)
        .map(|(&x, &y)| x.min(y))
        .collect();
    Ok(SdfGrid {
        data,
        nx: a.nx,
        ny: a.ny,
        nz: a.nz,
        origin: a.origin,
        cell_size: a.cell_size,
    })
}

/// Element-wise maximum (intersection of two SDFs).
pub fn sdf_intersection(a: &SdfGrid, b: &SdfGrid) -> anyhow::Result<SdfGrid> {
    check_compatible(a, b)?;
    let data = a
        .data
        .iter()
        .zip(&b.data)
        .map(|(&x, &y)| x.max(y))
        .collect();
    Ok(SdfGrid {
        data,
        nx: a.nx,
        ny: a.ny,
        nz: a.nz,
        origin: a.origin,
        cell_size: a.cell_size,
    })
}

/// Subtraction: `min(a, -b)`.
pub fn sdf_subtraction(a: &SdfGrid, b: &SdfGrid) -> anyhow::Result<SdfGrid> {
    check_compatible(a, b)?;
    let data = a
        .data
        .iter()
        .zip(&b.data)
        .map(|(&x, &y)| x.max(-y))
        .collect();
    Ok(SdfGrid {
        data,
        nx: a.nx,
        ny: a.ny,
        nz: a.nz,
        origin: a.origin,
        cell_size: a.cell_size,
    })
}

/// Smooth union using the smooth-min operator with blend factor `k`.
pub fn sdf_smooth_union(a: &SdfGrid, b: &SdfGrid, k: f32) -> anyhow::Result<SdfGrid> {
    check_compatible(a, b)?;
    let data = a
        .data
        .iter()
        .zip(&b.data)
        .map(|(&x, &y)| smooth_min(x, y, k))
        .collect();
    Ok(SdfGrid {
        data,
        nx: a.nx,
        ny: a.ny,
        nz: a.nz,
        origin: a.origin,
        cell_size: a.cell_size,
    })
}

// ── Analytical primitives ─────────────────────────────────────────────────────

/// Build a grid containing the analytical SDF of a sphere.
pub fn sphere_sdf(center: [f32; 3], radius: f32, params: &SdfParams) -> SdfGrid {
    let half = radius + params.padding;
    let mn = [center[0] - half, center[1] - half, center[2] - half];
    let mx = [center[0] + half, center[1] + half, center[2] + half];
    let span = (mx[0] - mn[0])
        .max(mx[1] - mn[1])
        .max(mx[2] - mn[2])
        .max(1e-6);
    let cell_size = span / params.resolution as f32;
    let nx = ((mx[0] - mn[0]) / cell_size).ceil() as usize;
    let ny = ((mx[1] - mn[1]) / cell_size).ceil() as usize;
    let nz = ((mx[2] - mn[2]) / cell_size).ceil() as usize;

    let mut grid = SdfGrid::new(nx, ny, nz, mn, cell_size);
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let p = grid.cell_centre(ix, iy, iz);
                let d = sub3(p, center);
                let dist = len3(d) - radius;
                let idx = grid.index(ix, iy, iz);
                grid.data[idx] = if params.use_sign { dist } else { dist.abs() };
            }
        }
    }
    grid
}

/// Build a grid containing the analytical SDF of an axis-aligned box.
pub fn box_sdf(center: [f32; 3], half_extents: [f32; 3], params: &SdfParams) -> SdfGrid {
    let mn = [
        center[0] - half_extents[0] - params.padding,
        center[1] - half_extents[1] - params.padding,
        center[2] - half_extents[2] - params.padding,
    ];
    let mx = [
        center[0] + half_extents[0] + params.padding,
        center[1] + half_extents[1] + params.padding,
        center[2] + half_extents[2] + params.padding,
    ];
    let span = (mx[0] - mn[0])
        .max(mx[1] - mn[1])
        .max(mx[2] - mn[2])
        .max(1e-6);
    let cell_size = span / params.resolution as f32;
    let nx = ((mx[0] - mn[0]) / cell_size).ceil() as usize;
    let ny = ((mx[1] - mn[1]) / cell_size).ceil() as usize;
    let nz = ((mx[2] - mn[2]) / cell_size).ceil() as usize;

    let mut grid = SdfGrid::new(nx, ny, nz, mn, cell_size);
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let p = grid.cell_centre(ix, iy, iz);
                let q = [
                    (p[0] - center[0]).abs() - half_extents[0],
                    (p[1] - center[1]).abs() - half_extents[1],
                    (p[2] - center[2]).abs() - half_extents[2],
                ];
                let qmax = [q[0].max(0.0), q[1].max(0.0), q[2].max(0.0)];
                let outside = len3(qmax);
                let inside = q[0].max(q[1]).max(q[2]).min(0.0);
                let dist = outside + inside;
                let idx = grid.index(ix, iy, iz);
                grid.data[idx] = if params.use_sign { dist } else { dist.abs() };
            }
        }
    }
    grid
}

// ── sdf_stats ─────────────────────────────────────────────────────────────────

/// Compute summary statistics over all grid cells.
pub fn sdf_stats(grid: &SdfGrid) -> SdfStats {
    if grid.data.is_empty() {
        return SdfStats {
            min_dist: 0.0,
            max_dist: 0.0,
            mean_dist: 0.0,
            negative_fraction: 0.0,
        };
    }
    let mut min_dist = f32::MAX;
    let mut max_dist = f32::MIN;
    let mut sum = 0.0f64;
    let mut neg_count = 0usize;
    for &v in &grid.data {
        if v < min_dist {
            min_dist = v;
        }
        if v > max_dist {
            max_dist = v;
        }
        sum += v as f64;
        if v < 0.0 {
            neg_count += 1;
        }
    }
    let n = grid.data.len();
    SdfStats {
        min_dist,
        max_dist,
        mean_dist: (sum / n as f64) as f32,
        negative_fraction: neg_count as f32 / n as f32,
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Build a unit sphere mesh (icosphere-style via shapes module).
    fn unit_sphere_mesh() -> MeshBuffers {
        crate::shapes::sphere(1.0, 16, 16)
    }

    fn default_params() -> SdfParams {
        SdfParams {
            resolution: 16,
            padding: 0.2,
            use_sign: true,
        }
    }

    fn default_params_unsigned() -> SdfParams {
        SdfParams {
            resolution: 16,
            padding: 0.2,
            use_sign: false,
        }
    }

    // Build two sphere SDFs with the same layout for boolean tests.
    fn two_matching_sphere_sdfs() -> (SdfGrid, SdfGrid) {
        let p = default_params();
        let a = sphere_sdf([0.0, 0.0, 0.0], 1.0, &p);
        // Use the same grid layout but fill analytically for b.
        let mut b_data = vec![0.0f32; a.data.len()];
        for iz in 0..a.nz {
            for iy in 0..a.ny {
                for ix in 0..a.nx {
                    let pt = a.cell_centre(ix, iy, iz);
                    let d = sub3(pt, [0.5, 0.0, 0.0]);
                    let dist = len3(d) - 1.0;
                    b_data[a.index(ix, iy, iz)] = dist;
                }
            }
        }
        let b = SdfGrid {
            data: b_data,
            nx: a.nx,
            ny: a.ny,
            nz: a.nz,
            origin: a.origin,
            cell_size: a.cell_size,
        };
        (a, b)
    }

    // ── 1. sphere_sdf: centre is negative ────────────────────────────────────

    #[test]
    fn sphere_sdf_centre_is_negative() {
        let p = default_params();
        let grid = sphere_sdf([0.0, 0.0, 0.0], 1.0, &p);
        let v = sample_sdf(&grid, [0.0, 0.0, 0.0]);
        assert!(
            v < 0.0,
            "centre of sphere should be inside (negative), got {v}"
        );
    }

    // ── 2. sphere_sdf: centre distance ≈ -radius ─────────────────────────────

    #[test]
    fn sphere_sdf_centre_value_approx_minus_radius() {
        let p = default_params();
        let radius = 1.0f32;
        let grid = sphere_sdf([0.0, 0.0, 0.0], radius, &p);
        let v = sample_sdf(&grid, [0.0, 0.0, 0.0]);
        assert!((v + radius).abs() < 0.15, "expected ~-1.0, got {v}");
    }

    // ── 3. sphere_sdf: surface ≈ 0 ───────────────────────────────────────────

    #[test]
    fn sphere_sdf_surface_near_zero() {
        let p = default_params();
        let radius = 1.0f32;
        let grid = sphere_sdf([0.0, 0.0, 0.0], radius, &p);
        let v = sample_sdf(&grid, [radius, 0.0, 0.0]);
        assert!(v.abs() < 0.15, "surface should be ~0, got {v}");
    }

    // ── 4. sphere_sdf: outside is positive ───────────────────────────────────

    #[test]
    fn sphere_sdf_outside_is_positive() {
        let p = default_params();
        let grid = sphere_sdf([0.0, 0.0, 0.0], 1.0, &p);
        let v = sample_sdf(&grid, [2.0, 0.0, 0.0]);
        assert!(v > 0.0, "outside sphere should be positive, got {v}");
    }

    // ── 5. box_sdf: centre is negative ───────────────────────────────────────

    #[test]
    fn box_sdf_centre_is_negative() {
        let p = default_params();
        let grid = box_sdf([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], &p);
        let v = sample_sdf(&grid, [0.0, 0.0, 0.0]);
        assert!(v < 0.0, "box centre should be inside (negative), got {v}");
    }

    // ── 6. box_sdf: outside corner is positive ───────────────────────────────

    #[test]
    fn box_sdf_outside_is_positive() {
        let p = default_params();
        let grid = box_sdf([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], &p);
        let v = sample_sdf(&grid, [3.0, 3.0, 3.0]);
        assert!(v > 0.0, "far outside box should be positive, got {v}");
    }

    // ── 7. box_sdf: surface ≈ 0 ──────────────────────────────────────────────

    #[test]
    fn box_sdf_surface_near_zero() {
        let p = default_params();
        let he = [1.0f32, 1.0, 1.0];
        let grid = box_sdf([0.0, 0.0, 0.0], he, &p);
        // Point on face centre.
        let v = sample_sdf(&grid, [1.0, 0.0, 0.0]);
        assert!(v.abs() < 0.15, "box face should be ~0, got {v}");
    }

    // ── 8. sdf_union: result ≤ min of inputs ─────────────────────────────────

    #[test]
    fn sdf_union_yields_min() {
        let (a, b) = two_matching_sphere_sdfs();
        let u = sdf_union(&a, &b).expect("union failed");
        for i in 0..u.data.len() {
            assert!(u.data[i] <= a.data[i] + 1e-5);
            assert!(u.data[i] <= b.data[i] + 1e-5);
        }
    }

    // ── 9. sdf_intersection: result ≥ max of inputs ──────────────────────────

    #[test]
    fn sdf_intersection_yields_max() {
        let (a, b) = two_matching_sphere_sdfs();
        let u = sdf_intersection(&a, &b).expect("intersection failed");
        for i in 0..u.data.len() {
            assert!(u.data[i] >= a.data[i] - 1e-5);
            assert!(u.data[i] >= b.data[i] - 1e-5);
        }
    }

    // ── 10. sdf_subtraction: basic sanity ────────────────────────────────────

    #[test]
    fn sdf_subtraction_sanity() {
        let (a, b) = two_matching_sphere_sdfs();
        let s = sdf_subtraction(&a, &b).expect("subtraction failed");
        // For each cell: result = max(a, -b)
        for i in 0..s.data.len() {
            let expected = a.data[i].max(-b.data[i]);
            assert!((s.data[i] - expected).abs() < 1e-6);
        }
    }

    // ── 11. grid mismatch error ───────────────────────────────────────────────

    #[test]
    fn sdf_union_mismatch_returns_error() {
        let p1 = SdfParams {
            resolution: 8,
            padding: 0.1,
            use_sign: true,
        };
        let p2 = SdfParams {
            resolution: 12,
            padding: 0.1,
            use_sign: true,
        };
        let a = sphere_sdf([0.0, 0.0, 0.0], 1.0, &p1);
        let b = sphere_sdf([0.0, 0.0, 0.0], 1.0, &p2);
        assert!(
            sdf_union(&a, &b).is_err(),
            "should fail on resolution mismatch"
        );
    }

    // ── 12. sample_sdf trilinear in a constant field ──────────────────────────

    #[test]
    fn sample_sdf_constant_field() {
        let mut grid = SdfGrid::new(4, 4, 4, [0.0, 0.0, 0.0], 1.0);
        // Fill with constant 5.0.
        for v in grid.data.iter_mut() {
            *v = 5.0;
        }
        let s = sample_sdf(&grid, [2.0, 2.0, 2.0]);
        assert!(
            (s - 5.0).abs() < 1e-4,
            "constant field should interpolate to 5.0, got {s}"
        );
    }

    // ── 13. sdf_stats negative fraction ─────────────────────────────────────

    #[test]
    fn sdf_stats_negative_fraction() {
        let p = default_params();
        let grid = sphere_sdf([0.0, 0.0, 0.0], 1.0, &p);
        let stats = sdf_stats(&grid);
        assert!(
            stats.negative_fraction > 0.0,
            "sphere interior should yield negative cells"
        );
        assert!(
            stats.negative_fraction < 1.0,
            "sphere exterior should yield positive cells"
        );
        assert!(stats.min_dist < 0.0);
        assert!(stats.max_dist > 0.0);
    }

    // ── 14. sdf_to_mesh returns a non-empty mesh ──────────────────────────────

    #[test]
    fn sdf_to_mesh_returns_valid_mesh() {
        let p = SdfParams {
            resolution: 16,
            padding: 0.2,
            use_sign: true,
        };
        let grid = sphere_sdf([0.0, 0.0, 0.0], 1.0, &p);
        let mesh = sdf_to_mesh(&grid, 0.0);
        assert!(
            mesh.positions.len() >= 3,
            "iso-surface mesh should have vertices"
        );
        assert_eq!(mesh.indices.len() % 3, 0, "indices should form triangles");
    }

    // ── 15. compute_sdf on a simple tetrahedron (unsigned) ───────────────────

    #[test]
    fn compute_sdf_unsigned_tetra() {
        use oxihuman_morph::engine::MeshBuffers as MB;
        // Simple tetrahedron.
        let src = MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 1, 3, 1, 2, 3, 0, 2, 3],
            has_suit: false,
        };
        let mesh = MeshBuffers::from_morph(src);
        let params = SdfParams {
            resolution: 8,
            padding: 0.2,
            use_sign: false,
        };
        let grid = compute_sdf(&mesh, &params);
        let stats = sdf_stats(&grid);
        // Unsigned: all values must be >= 0.
        assert!(
            stats.min_dist >= -1e-5,
            "unsigned SDF should have non-negative values, min={}",
            stats.min_dist
        );
    }

    // ── 16. smooth union is ≤ regular union ──────────────────────────────────

    #[test]
    fn sdf_smooth_union_le_hard_union() {
        let (a, b) = two_matching_sphere_sdfs();
        let hard = sdf_union(&a, &b).unwrap();
        let soft = sdf_smooth_union(&a, &b, 0.3).unwrap();
        // smooth_min ≤ min, so every cell should satisfy soft ≤ hard.
        for i in 0..soft.data.len() {
            assert!(
                soft.data[i] <= hard.data[i] + 1e-5,
                "smooth union cell {i}: {} > {} (hard union)",
                soft.data[i],
                hard.data[i]
            );
        }
    }

    // ── 17. smooth_min helper ────────────────────────────────────────────────

    #[test]
    fn smooth_min_equals_min_when_k_zero() {
        for (a, b) in [(-1.0f32, 2.0), (3.0, 0.5), (0.0, 0.0)] {
            let sm = smooth_min(a, b, 0.0);
            let m = a.min(b);
            assert!(
                (sm - m).abs() < 1e-6,
                "k=0: smooth_min({a},{b})={sm}, min={m}"
            );
        }
    }
}
