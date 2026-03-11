// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mesh-to-SDF (Signed Distance Field) generation.
//!
//! Generates a 3D SDF grid from a triangle mesh using spatial hashing for
//! accelerated closest-triangle queries and a pseudo-normal sign test for
//! inside/outside classification.

use std::collections::HashMap;

use anyhow::{bail, Result};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for SDF generation.
#[derive(Debug, Clone)]
pub struct SdfConfig {
    /// Number of grid cells along each axis.
    pub resolution: [usize; 3],
    /// Padding around the mesh bounding box (world units).
    pub padding: f64,
}

impl Default for SdfConfig {
    fn default() -> Self {
        Self {
            resolution: [64, 64, 64],
            padding: 0.1,
        }
    }
}

// ---------------------------------------------------------------------------
// SdfGrid
// ---------------------------------------------------------------------------

/// A 3-D signed distance field stored on a regular grid.
///
/// Negative values are inside the mesh, positive values are outside.
#[derive(Debug, Clone)]
pub struct SdfGrid {
    /// Flat storage in x-major order: index = x + y*rx + z*rx*ry.
    data: Vec<f64>,
    /// Grid resolution along each axis.
    resolution: [usize; 3],
    /// World-space position of the grid origin (corner with smallest coords).
    origin: [f64; 3],
    /// Uniform cell size (same in all 3 axes).
    cell_size: f64,
}

impl SdfGrid {
    // -- accessors ----------------------------------------------------------

    /// Grid resolution along each axis.
    #[inline]
    pub fn resolution(&self) -> [usize; 3] {
        self.resolution
    }

    /// World-space origin of the grid.
    #[inline]
    pub fn origin(&self) -> [f64; 3] {
        self.origin
    }

    /// Uniform cell size.
    #[inline]
    pub fn cell_size(&self) -> f64 {
        self.cell_size
    }

    /// Raw SDF data slice.
    #[inline]
    pub fn data(&self) -> &[f64] {
        &self.data
    }

    // -- construction -------------------------------------------------------

    /// Build an SDF grid from a triangle mesh.
    ///
    /// * `vertices`  – vertex positions
    /// * `triangles` – index triples (indices into `vertices`)
    /// * `normals`   – per-vertex normals (same length as `vertices`)
    /// * `config`    – grid resolution and padding
    ///
    /// # Errors
    ///
    /// Returns an error when inputs are inconsistent (e.g. empty mesh,
    /// mismatched array lengths, zero resolution).
    pub fn from_mesh(
        vertices: &[[f64; 3]],
        triangles: &[[usize; 3]],
        normals: &[[f64; 3]],
        config: &SdfConfig,
    ) -> Result<Self> {
        // -- validate inputs ------------------------------------------------
        if vertices.is_empty() {
            bail!("vertex list is empty");
        }
        if triangles.is_empty() {
            bail!("triangle list is empty");
        }
        if normals.len() != vertices.len() {
            bail!(
                "normals length ({}) does not match vertices length ({})",
                normals.len(),
                vertices.len()
            );
        }
        for dim in &config.resolution {
            if *dim == 0 {
                bail!("grid resolution must be non-zero on all axes");
            }
        }
        for tri in triangles {
            for &idx in tri {
                if idx >= vertices.len() {
                    bail!(
                        "triangle index {} out of range (vertex count = {})",
                        idx,
                        vertices.len()
                    );
                }
            }
        }

        // -- compute AABB ---------------------------------------------------
        let (aabb_min, aabb_max) = mesh_aabb(vertices);
        let origin = [
            aabb_min[0] - config.padding,
            aabb_min[1] - config.padding,
            aabb_min[2] - config.padding,
        ];
        let extent = [
            (aabb_max[0] - aabb_min[0]) + 2.0 * config.padding,
            (aabb_max[1] - aabb_min[1]) + 2.0 * config.padding,
            (aabb_max[2] - aabb_min[2]) + 2.0 * config.padding,
        ];
        // Uniform cell size = max extent / max resolution so the field covers
        // the whole bounding box on all axes.
        let cell_size = extent[0].max(extent[1]).max(extent[2])
            / (config.resolution[0]
                .max(config.resolution[1])
                .max(config.resolution[2]) as f64);
        if cell_size <= 0.0 || !cell_size.is_finite() {
            bail!("degenerate mesh bounding box produces invalid cell size");
        }

        // -- build spatial hash of triangles --------------------------------
        let avg_edge = average_edge_length(vertices, triangles);
        let hash_cell = if avg_edge > 0.0 {
            2.0 * avg_edge
        } else {
            cell_size
        };
        let spatial = SpatialHash::build(vertices, triangles, hash_cell);

        // -- evaluate SDF at each grid point --------------------------------
        let [rx, ry, rz] = config.resolution;
        let total = rx * ry * rz;
        let mut data = vec![0.0_f64; total];

        for iz in 0..rz {
            let pz = origin[2] + (iz as f64 + 0.5) * cell_size;
            for iy in 0..ry {
                let py = origin[1] + (iy as f64 + 0.5) * cell_size;
                for ix in 0..rx {
                    let px = origin[0] + (ix as f64 + 0.5) * cell_size;
                    let p = [px, py, pz];

                    let (dist, sign) =
                        closest_distance_and_sign(&p, vertices, triangles, normals, &spatial);

                    let idx = ix + iy * rx + iz * rx * ry;
                    data[idx] = dist * sign;
                }
            }
        }

        Ok(Self {
            data,
            resolution: config.resolution,
            origin,
            cell_size,
        })
    }

    // -- queries ------------------------------------------------------------

    /// Trilinear-interpolated SDF value at an arbitrary world-space point.
    ///
    /// Points outside the grid are clamped to the boundary.
    pub fn sample(&self, point: &[f64; 3]) -> f64 {
        let [rx, ry, rz] = self.resolution;

        // Continuous grid coordinates (cell-centred)
        let gx = (point[0] - self.origin[0]) / self.cell_size - 0.5;
        let gy = (point[1] - self.origin[1]) / self.cell_size - 0.5;
        let gz = (point[2] - self.origin[2]) / self.cell_size - 0.5;

        // Clamp so we stay in-bounds
        let gx = gx.clamp(0.0, (rx as f64) - 1.0001);
        let gy = gy.clamp(0.0, (ry as f64) - 1.0001);
        let gz = gz.clamp(0.0, (rz as f64) - 1.0001);

        let ix0 = gx.floor() as usize;
        let iy0 = gy.floor() as usize;
        let iz0 = gz.floor() as usize;
        let ix1 = (ix0 + 1).min(rx - 1);
        let iy1 = (iy0 + 1).min(ry - 1);
        let iz1 = (iz0 + 1).min(rz - 1);

        let fx = gx - ix0 as f64;
        let fy = gy - iy0 as f64;
        let fz = gz - iz0 as f64;

        let idx = |x: usize, y: usize, z: usize| x + y * rx + z * rx * ry;

        let c000 = self.data[idx(ix0, iy0, iz0)];
        let c100 = self.data[idx(ix1, iy0, iz0)];
        let c010 = self.data[idx(ix0, iy1, iz0)];
        let c110 = self.data[idx(ix1, iy1, iz0)];
        let c001 = self.data[idx(ix0, iy0, iz1)];
        let c101 = self.data[idx(ix1, iy0, iz1)];
        let c011 = self.data[idx(ix0, iy1, iz1)];
        let c111 = self.data[idx(ix1, iy1, iz1)];

        trilinear(fx, fy, fz, c000, c100, c010, c110, c001, c101, c011, c111)
    }

    /// Numerical gradient of the SDF at a world-space point.
    ///
    /// Uses central finite differences with a step of one cell size.
    pub fn gradient(&self, point: &[f64; 3]) -> [f64; 3] {
        let h = self.cell_size;
        let inv_2h = 1.0 / (2.0 * h);

        let dx = self.sample(&[point[0] + h, point[1], point[2]])
            - self.sample(&[point[0] - h, point[1], point[2]]);
        let dy = self.sample(&[point[0], point[1] + h, point[2]])
            - self.sample(&[point[0], point[1] - h, point[2]]);
        let dz = self.sample(&[point[0], point[1], point[2] + h])
            - self.sample(&[point[0], point[1], point[2] - h]);

        [dx * inv_2h, dy * inv_2h, dz * inv_2h]
    }

    /// Approximate closest surface point by stepping from `point` along the
    /// negative gradient by the SDF value.
    ///
    /// This performs a single projection step which is exact for a perfect
    /// SDF and a good approximation for discretised grids.
    pub fn closest_surface_point(&self, point: &[f64; 3]) -> [f64; 3] {
        let d = self.sample(point);
        let g = self.gradient(point);
        let glen = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        if glen < 1e-12 {
            return *point;
        }
        let inv_g = 1.0 / glen;
        [
            point[0] - d * g[0] * inv_g,
            point[1] - d * g[1] * inv_g,
            point[2] - d * g[2] * inv_g,
        ]
    }
}

// ===========================================================================
// Internal helpers
// ===========================================================================

/// Axis-aligned bounding box of a point cloud.
fn mesh_aabb(vertices: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    let mut lo = [f64::MAX; 3];
    let mut hi = [f64::MIN; 3];
    for v in vertices {
        for i in 0..3 {
            if v[i] < lo[i] {
                lo[i] = v[i];
            }
            if v[i] > hi[i] {
                hi[i] = v[i];
            }
        }
    }
    (lo, hi)
}

/// Average edge length of the triangle mesh.
fn average_edge_length(vertices: &[[f64; 3]], triangles: &[[usize; 3]]) -> f64 {
    if triangles.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0_f64;
    let mut count = 0u64;
    for tri in triangles {
        let a = vertices[tri[0]];
        let b = vertices[tri[1]];
        let c = vertices[tri[2]];
        sum += dist3(&a, &b);
        sum += dist3(&b, &c);
        sum += dist3(&c, &a);
        count += 3;
    }
    if count == 0 {
        0.0
    } else {
        sum / (count as f64)
    }
}

// ---------------------------------------------------------------------------
// Spatial hash
// ---------------------------------------------------------------------------

/// Simple uniform-grid spatial hash that maps 3-D integer cells to lists of
/// triangle indices.
struct SpatialHash {
    #[allow(dead_code)]
    cell_size: f64,
    inv_cell: f64,
    map: HashMap<[i64; 3], Vec<usize>>,
}

impl SpatialHash {
    /// Build the spatial hash by rasterising each triangle's AABB into cells.
    fn build(vertices: &[[f64; 3]], triangles: &[[usize; 3]], cell_size: f64) -> Self {
        let inv_cell = 1.0 / cell_size;
        let mut map: HashMap<[i64; 3], Vec<usize>> = HashMap::new();

        for (ti, tri) in triangles.iter().enumerate() {
            let a = vertices[tri[0]];
            let b = vertices[tri[1]];
            let c = vertices[tri[2]];

            // AABB of the triangle
            let lo = [
                a[0].min(b[0]).min(c[0]),
                a[1].min(b[1]).min(c[1]),
                a[2].min(b[2]).min(c[2]),
            ];
            let hi = [
                a[0].max(b[0]).max(c[0]),
                a[1].max(b[1]).max(c[1]),
                a[2].max(b[2]).max(c[2]),
            ];

            let clo = [
                (lo[0] * inv_cell).floor() as i64,
                (lo[1] * inv_cell).floor() as i64,
                (lo[2] * inv_cell).floor() as i64,
            ];
            let chi = [
                (hi[0] * inv_cell).floor() as i64,
                (hi[1] * inv_cell).floor() as i64,
                (hi[2] * inv_cell).floor() as i64,
            ];

            for cz in clo[2]..=chi[2] {
                for cy in clo[1]..=chi[1] {
                    for cx in clo[0]..=chi[0] {
                        map.entry([cx, cy, cz]).or_default().push(ti);
                    }
                }
            }
        }

        Self {
            cell_size,
            inv_cell,
            map,
        }
    }

    /// Cell key for a world-space point.
    #[inline]
    fn cell_of(&self, p: &[f64; 3]) -> [i64; 3] {
        [
            (p[0] * self.inv_cell).floor() as i64,
            (p[1] * self.inv_cell).floor() as i64,
            (p[2] * self.inv_cell).floor() as i64,
        ]
    }

    /// Iterate over candidate triangle indices near `point`.
    ///
    /// Searches the 3x3x3 neighbourhood around the point, then expands if
    /// nothing is found.
    fn query_near(&self, point: &[f64; 3]) -> Vec<usize> {
        let c = self.cell_of(point);
        let mut results = Vec::new();

        // Start with radius 1 (3x3x3 = 27 cells).  Expand if empty.
        for radius in 1..=16_i64 {
            results.clear();
            for dz in -radius..=radius {
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let key = [c[0] + dx, c[1] + dy, c[2] + dz];
                        if let Some(tris) = self.map.get(&key) {
                            results.extend_from_slice(tris);
                        }
                    }
                }
            }
            if !results.is_empty() {
                break;
            }
        }

        // Remove duplicates (a triangle can appear in multiple cells)
        results.sort_unstable();
        results.dedup();
        results
    }
}

// ---------------------------------------------------------------------------
// Closest point on triangle + sign computation
// ---------------------------------------------------------------------------

/// Find the closest distance from `point` to the mesh and the sign
/// (+1 outside, -1 inside) using the pseudo-normal test.
fn closest_distance_and_sign(
    point: &[f64; 3],
    vertices: &[[f64; 3]],
    triangles: &[[usize; 3]],
    normals: &[[f64; 3]],
    spatial: &SpatialHash,
) -> (f64, f64) {
    let candidates = spatial.query_near(point);

    let mut best_dist_sq = f64::MAX;
    let mut best_closest = *point;
    let mut best_tri_idx = 0usize;
    let mut best_bary = [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];

    for &ti in &candidates {
        let tri = triangles[ti];
        let a = vertices[tri[0]];
        let b = vertices[tri[1]];
        let c = vertices[tri[2]];

        let (cp, bary) = closest_point_on_triangle(point, &a, &b, &c);
        let dsq = dist3_sq(point, &cp);
        if dsq < best_dist_sq {
            best_dist_sq = dsq;
            best_closest = cp;
            best_tri_idx = ti;
            best_bary = bary;
        }
    }

    // If we found nothing (degenerate input), return large positive value
    if best_dist_sq == f64::MAX {
        return (1e30, 1.0);
    }

    let dist = best_dist_sq.sqrt();

    // Pseudo-normal sign test
    let tri = triangles[best_tri_idx];
    let n0 = normals[tri[0]];
    let n1 = normals[tri[1]];
    let n2 = normals[tri[2]];
    let interp_normal = [
        best_bary[0] * n0[0] + best_bary[1] * n1[0] + best_bary[2] * n2[0],
        best_bary[0] * n0[1] + best_bary[1] * n1[1] + best_bary[2] * n2[1],
        best_bary[0] * n0[2] + best_bary[1] * n1[2] + best_bary[2] * n2[2],
    ];

    let to_point = sub3(point, &best_closest);
    let dot = dot3(&to_point, &interp_normal);

    let sign = if dot >= 0.0 { 1.0 } else { -1.0 };

    (dist, sign)
}

/// Closest point on triangle ABC to point P, returning the point and
/// barycentric coordinates [u, v, w] such that  closest = u*A + v*B + w*C.
fn closest_point_on_triangle(
    p: &[f64; 3],
    a: &[f64; 3],
    b: &[f64; 3],
    c: &[f64; 3],
) -> ([f64; 3], [f64; 3]) {
    // Voronoi region method (Real-Time Collision Detection, Ericson)
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ap = sub3(p, a);

    let d1 = dot3(&ab, &ap);
    let d2 = dot3(&ac, &ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (*a, [1.0, 0.0, 0.0]);
    }

    let bp = sub3(p, b);
    let d3 = dot3(&ab, &bp);
    let d4 = dot3(&ac, &bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (*b, [0.0, 1.0, 0.0]);
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        let pt = [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
        return (pt, [1.0 - v, v, 0.0]);
    }

    let cp = sub3(p, c);
    let d5 = dot3(&ab, &cp);
    let d6 = dot3(&ac, &cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (*c, [0.0, 0.0, 1.0]);
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        let pt = [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]];
        return (pt, [1.0 - w, 0.0, w]);
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let pt = [
            b[0] + w * (c[0] - b[0]),
            b[1] + w * (c[1] - b[1]),
            b[2] + w * (c[2] - b[2]),
        ];
        return (pt, [0.0, 1.0 - w, w]);
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let u = 1.0 - v - w;
    let pt = [
        a[0] + v * ab[0] + w * ac[0],
        a[1] + v * ab[1] + w * ac[1],
        a[2] + v * ab[2] + w * ac[2],
    ];
    (pt, [u, v, w])
}

// ---------------------------------------------------------------------------
// Trilinear interpolation
// ---------------------------------------------------------------------------

#[inline]
#[allow(clippy::too_many_arguments)]
fn trilinear(
    fx: f64,
    fy: f64,
    fz: f64,
    c000: f64,
    c100: f64,
    c010: f64,
    c110: f64,
    c001: f64,
    c101: f64,
    c011: f64,
    c111: f64,
) -> f64 {
    let c00 = c000 * (1.0 - fx) + c100 * fx;
    let c10 = c010 * (1.0 - fx) + c110 * fx;
    let c01 = c001 * (1.0 - fx) + c101 * fx;
    let c11 = c011 * (1.0 - fx) + c111 * fx;
    let c0 = c00 * (1.0 - fy) + c10 * fy;
    let c1 = c01 * (1.0 - fy) + c11 * fy;
    c0 * (1.0 - fz) + c1 * fz
}

// ---------------------------------------------------------------------------
// Small vec3 helpers (f64)
// ---------------------------------------------------------------------------

#[inline]
fn sub3(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn dot3(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn dist3(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    dist3_sq(a, b).sqrt()
}

#[inline]
fn dist3_sq(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let d = sub3(a, b);
    dot3(&d, &d)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a simple unit cube mesh (8 verts, 12 triangles) centred at origin.
    #[allow(clippy::type_complexity)]
    fn unit_cube() -> (Vec<[f64; 3]>, Vec<[usize; 3]>, Vec<[f64; 3]>) {
        let h = 0.5;
        #[rustfmt::skip]
        let verts: Vec<[f64; 3]> = vec![
            [-h, -h, -h], [ h, -h, -h], [ h,  h, -h], [-h,  h, -h],
            [-h, -h,  h], [ h, -h,  h], [ h,  h,  h], [-h,  h,  h],
        ];

        // 12 triangles (2 per face), winding outward
        #[rustfmt::skip]
        let tris: Vec<[usize; 3]> = vec![
            // -Z
            [0, 2, 1], [0, 3, 2],
            // +Z
            [4, 5, 6], [4, 6, 7],
            // -Y
            [0, 1, 5], [0, 5, 4],
            // +Y
            [2, 3, 7], [2, 7, 6],
            // -X
            [0, 4, 7], [0, 7, 3],
            // +X
            [1, 2, 6], [1, 6, 5],
        ];

        // Compute per-vertex normals by averaging face normals
        let mut norms = vec![[0.0; 3]; verts.len()];
        for tri in &tris {
            let a = verts[tri[0]];
            let b = verts[tri[1]];
            let c = verts[tri[2]];
            let ab = sub3(&b, &a);
            let ac = sub3(&c, &a);
            let n = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            for &vi in tri {
                norms[vi][0] += n[0];
                norms[vi][1] += n[1];
                norms[vi][2] += n[2];
            }
        }
        // normalise
        for n in &mut norms {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-12 {
                n[0] /= len;
                n[1] /= len;
                n[2] /= len;
            }
        }

        (verts, tris, norms)
    }

    #[test]
    fn test_sdf_outside_positive() {
        let (v, t, n) = unit_cube();
        let cfg = SdfConfig {
            resolution: [16, 16, 16],
            padding: 0.5,
        };
        let sdf = SdfGrid::from_mesh(&v, &t, &n, &cfg).unwrap();
        // A point well outside should be positive
        let val = sdf.sample(&[2.0, 0.0, 0.0]);
        assert!(val > 0.0, "expected positive outside, got {val}");
    }

    #[test]
    fn test_sdf_inside_negative() {
        let (v, t, n) = unit_cube();
        let cfg = SdfConfig {
            resolution: [16, 16, 16],
            padding: 0.5,
        };
        let sdf = SdfGrid::from_mesh(&v, &t, &n, &cfg).unwrap();
        // Origin is inside the unit cube
        let val = sdf.sample(&[0.0, 0.0, 0.0]);
        assert!(val < 0.0, "expected negative inside, got {val}");
    }

    #[test]
    fn test_gradient_points_outward() {
        let (v, t, n) = unit_cube();
        let cfg = SdfConfig {
            resolution: [32, 32, 32],
            padding: 0.5,
        };
        let sdf = SdfGrid::from_mesh(&v, &t, &n, &cfg).unwrap();
        let p = [0.8, 0.0, 0.0];
        let g = sdf.gradient(&p);
        // Gradient should point roughly in +X direction
        assert!(g[0] > 0.0, "gradient x should be positive, got {:?}", g);
    }

    #[test]
    fn test_closest_surface_point() {
        let (v, t, n) = unit_cube();
        let cfg = SdfConfig {
            resolution: [32, 32, 32],
            padding: 0.5,
        };
        let sdf = SdfGrid::from_mesh(&v, &t, &n, &cfg).unwrap();
        let p = [1.0, 0.0, 0.0];
        let cp = sdf.closest_surface_point(&p);
        // Should be near x=0.5 face
        assert!(
            (cp[0] - 0.5).abs() < 0.15,
            "closest x should be near 0.5, got {:?}",
            cp
        );
    }

    #[test]
    fn test_from_mesh_empty_vertices() {
        let cfg = SdfConfig::default();
        let res = SdfGrid::from_mesh(&[], &[[0, 1, 2]], &[], &cfg);
        assert!(res.is_err());
    }

    #[test]
    fn test_from_mesh_empty_triangles() {
        let cfg = SdfConfig::default();
        let v = vec![[0.0; 3]];
        let n = vec![[0.0, 1.0, 0.0]];
        let res = SdfGrid::from_mesh(&v, &[], &n, &cfg);
        assert!(res.is_err());
    }

    #[test]
    fn test_from_mesh_mismatched_normals() {
        let cfg = SdfConfig::default();
        let v = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let t = vec![[0, 1, 2]];
        let n = vec![[0.0, 0.0, 1.0]]; // wrong length
        let res = SdfGrid::from_mesh(&v, &t, &n, &cfg);
        assert!(res.is_err());
    }

    #[test]
    fn test_closest_point_on_triangle_vertex_regions() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];

        // Near vertex A
        let (cp, bary) = closest_point_on_triangle(&[-1.0, -1.0, 0.0], &a, &b, &c);
        assert!(dist3(&cp, &a) < 1e-10);
        assert!((bary[0] - 1.0).abs() < 1e-10);

        // Near vertex B
        let (cp, _bary) = closest_point_on_triangle(&[2.0, -1.0, 0.0], &a, &b, &c);
        assert!(dist3(&cp, &b) < 1e-10);

        // Near vertex C
        let (cp, _bary) = closest_point_on_triangle(&[-1.0, 2.0, 0.0], &a, &b, &c);
        assert!(dist3(&cp, &c) < 1e-10);
    }

    #[test]
    fn test_trilinear_corners() {
        // All corners same value => interpolation = that value
        let v = trilinear(0.5, 0.5, 0.5, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0);
        assert!((v - 3.0).abs() < 1e-12);
    }
}
