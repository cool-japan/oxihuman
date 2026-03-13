// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;
use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// ProxyShape
// ---------------------------------------------------------------------------

/// Simplified physics proxy shape fitted to a mesh region.
#[derive(Debug, Clone)]
pub enum ProxyShape {
    /// Sphere: center + radius
    Sphere { center: [f32; 3], radius: f32 },
    /// Capsule: two endpoint centers + radius
    Capsule {
        p0: [f32; 3],
        p1: [f32; 3],
        radius: f32,
    },
    /// Oriented bounding box: center, half-extents, axes
    OrientedBox {
        center: [f32; 3],
        half_extents: [f32; 3],
        axes: [[f32; 3]; 3],
    },
    /// AABB: min and max corners
    Aabb { min: [f32; 3], max: [f32; 3] },
}

impl ProxyShape {
    /// Volume of the proxy shape.
    pub fn volume(&self) -> f32 {
        match self {
            ProxyShape::Sphere { radius, .. } => (4.0 / 3.0) * PI * radius * radius * radius,
            ProxyShape::Capsule { p0, p1, radius } => {
                let h = vec3_len(vec3_sub(*p1, *p0));
                PI * radius * radius * h + (4.0 / 3.0) * PI * radius * radius * radius
            }
            ProxyShape::OrientedBox { half_extents, .. } => {
                8.0 * half_extents[0] * half_extents[1] * half_extents[2]
            }
            ProxyShape::Aabb { min, max } => {
                let dx = (max[0] - min[0]).max(0.0);
                let dy = (max[1] - min[1]).max(0.0);
                let dz = (max[2] - min[2]).max(0.0);
                dx * dy * dz
            }
        }
    }

    /// Surface area of the proxy shape.
    pub fn surface_area(&self) -> f32 {
        match self {
            ProxyShape::Sphere { radius, .. } => 4.0 * PI * radius * radius,
            ProxyShape::Capsule { p0, p1, radius } => {
                let h = vec3_len(vec3_sub(*p1, *p0));
                2.0 * PI * radius * h + 4.0 * PI * radius * radius
            }
            ProxyShape::OrientedBox { half_extents, .. } => {
                let [a, b, c] = *half_extents;
                8.0 * (a * b + b * c + c * a)
            }
            ProxyShape::Aabb { min, max } => {
                let dx = (max[0] - min[0]).max(0.0);
                let dy = (max[1] - min[1]).max(0.0);
                let dz = (max[2] - min[2]).max(0.0);
                2.0 * (dx * dy + dy * dz + dz * dx)
            }
        }
    }

    /// Center of mass.
    pub fn center(&self) -> [f32; 3] {
        match self {
            ProxyShape::Sphere { center, .. } => *center,
            ProxyShape::Capsule { p0, p1, .. } => vec3_scale(vec3_add(*p0, *p1), 0.5),
            ProxyShape::OrientedBox { center, .. } => *center,
            ProxyShape::Aabb { min, max } => vec3_scale(vec3_add(*min, *max), 0.5),
        }
    }

    /// Bounding box (min, max).
    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        match self {
            ProxyShape::Sphere { center, radius } => {
                let r = *radius;
                let [cx, cy, cz] = *center;
                ([cx - r, cy - r, cz - r], [cx + r, cy + r, cz + r])
            }
            ProxyShape::Capsule { p0, p1, radius } => {
                let r = *radius;
                let min_x = p0[0].min(p1[0]) - r;
                let min_y = p0[1].min(p1[1]) - r;
                let min_z = p0[2].min(p1[2]) - r;
                let max_x = p0[0].max(p1[0]) + r;
                let max_y = p0[1].max(p1[1]) + r;
                let max_z = p0[2].max(p1[2]) + r;
                ([min_x, min_y, min_z], [max_x, max_y, max_z])
            }
            ProxyShape::OrientedBox {
                center,
                half_extents,
                axes,
            } => {
                // Project all 8 corners
                let mut mn = [f32::MAX; 3];
                let mut mx = [f32::MIN; 3];
                for sx in [-1.0f32, 1.0] {
                    for sy in [-1.0f32, 1.0] {
                        for sz in [-1.0f32, 1.0] {
                            let corner = vec3_add(
                                *center,
                                vec3_add(
                                    vec3_add(
                                        vec3_scale(axes[0], sx * half_extents[0]),
                                        vec3_scale(axes[1], sy * half_extents[1]),
                                    ),
                                    vec3_scale(axes[2], sz * half_extents[2]),
                                ),
                            );
                            for k in 0..3 {
                                mn[k] = mn[k].min(corner[k]);
                                mx[k] = mx[k].max(corner[k]);
                            }
                        }
                    }
                }
                (mn, mx)
            }
            ProxyShape::Aabb { min, max } => (*min, *max),
        }
    }
}

// ---------------------------------------------------------------------------
// ProxyFitResult
// ---------------------------------------------------------------------------

/// Result of fitting a proxy shape to a mesh region.
pub struct ProxyFitResult {
    pub shape: ProxyShape,
    /// Fit quality: fraction of mesh vertices covered by the proxy (0..1).
    pub coverage: f32,
    /// Fraction of proxy volume "used" — compactness measure.
    pub compactness: f32,
}

// ---------------------------------------------------------------------------
// ProxyShapeType
// ---------------------------------------------------------------------------

/// Which proxy shape to generate.
pub enum ProxyShapeType {
    Sphere,
    Capsule,
    Aabb,
    Obb,
    /// Auto-select the best fitting shape (sphere vs capsule).
    Auto,
}

// ---------------------------------------------------------------------------
// Vector math helpers (no external deps)
// ---------------------------------------------------------------------------

fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec3_len(a: [f32; 3]) -> f32 {
    vec3_dot(a, a).sqrt()
}

fn vec3_len_sq(a: [f32; 3]) -> f32 {
    vec3_dot(a, a)
}

fn vec3_normalize(a: [f32; 3]) -> [f32; 3] {
    let l = vec3_len(a);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        vec3_scale(a, 1.0 / l)
    }
}

fn vec3_dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    vec3_len_sq(vec3_sub(a, b))
}

fn vec3_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    vec3_dist_sq(a, b).sqrt()
}

// ---------------------------------------------------------------------------
// Centroid
// ---------------------------------------------------------------------------

fn centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    if positions.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for p in positions {
        sum = vec3_add(sum, *p);
    }
    vec3_scale(sum, 1.0 / positions.len() as f32)
}

// ---------------------------------------------------------------------------
// PCA — 3x3 symmetric covariance matrix, power iteration for eigenvectors
// ---------------------------------------------------------------------------

/// Compute the 3×3 covariance matrix of positions (relative to centroid).
fn covariance_matrix(positions: &[[f32; 3]], c: [f32; 3]) -> [[f32; 3]; 3] {
    let mut cov = [[0.0f32; 3]; 3];
    let n = positions.len() as f32;
    if n < 1.0 {
        return cov;
    }
    for p in positions {
        let d = vec3_sub(*p, c);
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }
    for row in &mut cov {
        for v in row.iter_mut() {
            *v /= n;
        }
    }
    cov
}

/// Matrix-vector multiply for 3×3 matrix.
fn mat3_mul_vec3(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// Power iteration to find the dominant eigenvector of a symmetric 3×3 matrix.
fn power_iter(m: [[f32; 3]; 3], start: [f32; 3]) -> ([f32; 3], f32) {
    let mut v = vec3_normalize(start);
    for _ in 0..64 {
        let mv = mat3_mul_vec3(m, v);
        let l = vec3_len(mv);
        if l < 1e-12 {
            break;
        }
        v = vec3_scale(mv, 1.0 / l);
    }
    let lam = eigenvalue(m, v);
    (v, lam)
}

fn dominant_eigenvector(m: [[f32; 3]; 3]) -> [f32; 3] {
    // Try three axis-aligned starting vectors and pick the one with largest eigenvalue
    let starts: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let mut best_v = [1.0f32, 0.0, 0.0];
    let mut best_lam = f32::NEG_INFINITY;
    for &s in &starts {
        let (v, lam) = power_iter(m, s);
        if lam > best_lam {
            best_lam = lam;
            best_v = v;
        }
    }
    best_v
}

/// Deflate matrix by removing the component along eigenvector `e` (rank-1 deflation).
fn deflate(m: [[f32; 3]; 3], e: [f32; 3], eigenvalue: f32) -> [[f32; 3]; 3] {
    let mut d = m;
    for i in 0..3 {
        for j in 0..3 {
            d[i][j] -= eigenvalue * e[i] * e[j];
        }
    }
    d
}

/// Estimate eigenvalue for eigenvector `e`.
fn eigenvalue(m: [[f32; 3]; 3], e: [f32; 3]) -> f32 {
    let me = mat3_mul_vec3(m, e);
    vec3_dot(me, e)
}

/// Build a vector orthogonal to `a`.
fn orthogonal(a: [f32; 3]) -> [f32; 3] {
    // Pick the axis least parallel to `a`
    let ax = a[0].abs();
    let ay = a[1].abs();
    let az = a[2].abs();
    let perp = if ax <= ay && ax <= az {
        [1.0f32, 0.0, 0.0]
    } else if ay <= az {
        [0.0f32, 1.0, 0.0]
    } else {
        [0.0f32, 0.0, 1.0]
    };
    // Gram-Schmidt
    let dot = vec3_dot(perp, a);
    vec3_normalize(vec3_sub(perp, vec3_scale(a, dot)))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Compute 3 orthonormal eigenvectors via power iteration + deflation.
fn pca_axes(positions: &[[f32; 3]]) -> [[f32; 3]; 3] {
    let c = centroid(positions);
    let cov = covariance_matrix(positions, c);

    let e0 = dominant_eigenvector(cov);
    let lam0 = eigenvalue(cov, e0);
    let cov1 = deflate(cov, e0, lam0);

    let e1_raw = dominant_eigenvector(cov1);
    // Orthogonalize against e0
    let dot01 = vec3_dot(e1_raw, e0);
    let e1 = vec3_normalize(vec3_sub(e1_raw, vec3_scale(e0, dot01)));

    let e2 = vec3_normalize(cross(e0, e1));

    [e0, e1, e2]
}

// ---------------------------------------------------------------------------
// fit_aabb
// ---------------------------------------------------------------------------

/// Fit an AABB to a set of vertices.
pub fn fit_aabb(positions: &[[f32; 3]]) -> ProxyShape {
    if positions.is_empty() {
        return ProxyShape::Aabb {
            min: [0.0; 3],
            max: [0.0; 3],
        };
    }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for p in positions {
        for k in 0..3 {
            mn[k] = mn[k].min(p[k]);
            mx[k] = mx[k].max(p[k]);
        }
    }
    ProxyShape::Aabb { min: mn, max: mx }
}

// ---------------------------------------------------------------------------
// fit_sphere  (Ritter's algorithm)
// ---------------------------------------------------------------------------

/// Fit a sphere to a set of vertices using Ritter's algorithm.
pub fn fit_sphere(positions: &[[f32; 3]]) -> ProxyShape {
    if positions.is_empty() {
        return ProxyShape::Sphere {
            center: [0.0; 3],
            radius: 0.0,
        };
    }
    if positions.len() == 1 {
        return ProxyShape::Sphere {
            center: positions[0],
            radius: 0.0,
        };
    }

    // Step 1: find the point farthest from positions[0]
    let p0 = positions[0];
    let far1 = positions
        .iter()
        .max_by(|a, b| {
            vec3_dist_sq(**a, p0)
                .partial_cmp(&vec3_dist_sq(**b, p0))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
        .unwrap_or(p0);

    // Step 2: find the point farthest from far1
    let far2 = positions
        .iter()
        .max_by(|a, b| {
            vec3_dist_sq(**a, far1)
                .partial_cmp(&vec3_dist_sq(**b, far1))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
        .unwrap_or(far1);

    // Initial sphere: midpoint of far1-far2, half-distance as radius
    let mut center = vec3_scale(vec3_add(far1, far2), 0.5);
    let mut radius = vec3_dist(far1, far2) * 0.5;

    // Expand to cover all vertices
    for &p in positions {
        let d = vec3_dist(p, center);
        if d > radius {
            let new_radius = (radius + d) * 0.5;
            let t = (new_radius - radius) / d;
            center = vec3_add(center, vec3_scale(vec3_sub(p, center), t));
            radius = new_radius;
        }
    }

    ProxyShape::Sphere { center, radius }
}

// ---------------------------------------------------------------------------
// fit_capsule
// ---------------------------------------------------------------------------

/// Fit a capsule to a set of vertices.
///
/// Finds the principal axis via PCA, projects all points, and uses
/// min/max projections as endpoints; radius = max perpendicular distance.
pub fn fit_capsule(positions: &[[f32; 3]]) -> ProxyShape {
    if positions.is_empty() {
        return ProxyShape::Capsule {
            p0: [0.0; 3],
            p1: [0.0, 1.0, 0.0],
            radius: 0.0,
        };
    }
    if positions.len() == 1 {
        return ProxyShape::Capsule {
            p0: positions[0],
            p1: positions[0],
            radius: 0.0,
        };
    }

    let c = centroid(positions);
    let axes = pca_axes(positions);
    let axis = axes[0]; // dominant axis

    // Project onto axis, find min/max t
    let mut t_min = f32::MAX;
    let mut t_max = f32::MIN;
    for p in positions {
        let d = vec3_sub(*p, c);
        let t = vec3_dot(d, axis);
        t_min = t_min.min(t);
        t_max = t_max.max(t);
    }

    let p0 = vec3_add(c, vec3_scale(axis, t_min));
    let p1 = vec3_add(c, vec3_scale(axis, t_max));

    // Radius = max perpendicular distance from axis
    let mut radius = 0.0f32;
    for p in positions {
        let d = vec3_sub(*p, c);
        let t = vec3_dot(d, axis);
        let on_axis = vec3_scale(axis, t);
        let perp = vec3_sub(d, on_axis);
        let perp_dist = vec3_len(perp);
        if perp_dist > radius {
            radius = perp_dist;
        }
    }

    ProxyShape::Capsule { p0, p1, radius }
}

// ---------------------------------------------------------------------------
// fit_obb
// ---------------------------------------------------------------------------

/// Fit an OBB using PCA on vertex positions.
pub fn fit_obb(positions: &[[f32; 3]]) -> ProxyShape {
    if positions.is_empty() {
        return ProxyShape::OrientedBox {
            center: [0.0; 3],
            half_extents: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        };
    }

    let c = centroid(positions);
    let axes = pca_axes(positions);

    let mut t_min = [f32::MAX; 3];
    let mut t_max = [f32::MIN; 3];

    for p in positions {
        let d = vec3_sub(*p, c);
        for k in 0..3 {
            let t = vec3_dot(d, axes[k]);
            t_min[k] = t_min[k].min(t);
            t_max[k] = t_max[k].max(t);
        }
    }

    let half_extents = [
        (t_max[0] - t_min[0]) * 0.5,
        (t_max[1] - t_min[1]) * 0.5,
        (t_max[2] - t_min[2]) * 0.5,
    ];

    // Re-center (OBB center may differ from PCA centroid when data is skewed)
    let obb_center = vec3_add(
        c,
        vec3_add(
            vec3_add(
                vec3_scale(axes[0], (t_min[0] + t_max[0]) * 0.5),
                vec3_scale(axes[1], (t_min[1] + t_max[1]) * 0.5),
            ),
            vec3_scale(axes[2], (t_min[2] + t_max[2]) * 0.5),
        ),
    );

    ProxyShape::OrientedBox {
        center: obb_center,
        half_extents,
        axes,
    }
}

// ---------------------------------------------------------------------------
// coverage helpers
// ---------------------------------------------------------------------------

fn sphere_coverage(positions: &[[f32; 3]], shape: &ProxyShape) -> f32 {
    if positions.is_empty() {
        return 0.0;
    }
    if let ProxyShape::Sphere { center, radius } = shape {
        let inside = positions
            .iter()
            .filter(|p| vec3_dist(**p, *center) <= *radius * (1.0 + 1e-5))
            .count();
        inside as f32 / positions.len() as f32
    } else {
        0.0
    }
}

fn capsule_coverage(positions: &[[f32; 3]], shape: &ProxyShape) -> f32 {
    if positions.is_empty() {
        return 0.0;
    }
    if let ProxyShape::Capsule { p0, p1, radius } = shape {
        let inside = positions
            .iter()
            .filter(|p| point_to_segment_dist(**p, *p0, *p1) <= *radius * (1.0 + 1e-5))
            .count();
        inside as f32 / positions.len() as f32
    } else {
        0.0
    }
}

/// Distance from point `p` to segment [a, b].
fn point_to_segment_dist(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let ab = vec3_sub(b, a);
    let ab_len_sq = vec3_len_sq(ab);
    if ab_len_sq < 1e-20 {
        return vec3_dist(p, a);
    }
    let ap = vec3_sub(p, a);
    let t = (vec3_dot(ap, ab) / ab_len_sq).clamp(0.0, 1.0);
    let closest = vec3_add(a, vec3_scale(ab, t));
    vec3_dist(p, closest)
}

// ---------------------------------------------------------------------------
// mesh_proxy / region_proxy
// ---------------------------------------------------------------------------

/// Generate a proxy for a full mesh.
pub fn mesh_proxy(mesh: &MeshBuffers, shape_type: ProxyShapeType) -> ProxyFitResult {
    region_proxy(&mesh.positions, shape_type)
}

/// Generate a proxy for a vertex subset (e.g. a body region).
pub fn region_proxy(positions: &[[f32; 3]], shape_type: ProxyShapeType) -> ProxyFitResult {
    let shape = match shape_type {
        ProxyShapeType::Sphere => fit_sphere(positions),
        ProxyShapeType::Capsule => fit_capsule(positions),
        ProxyShapeType::Aabb => fit_aabb(positions),
        ProxyShapeType::Obb => fit_obb(positions),
        ProxyShapeType::Auto => {
            let s = fit_sphere(positions);
            let cap = fit_capsule(positions);
            let sc = sphere_coverage(positions, &s);
            let cc = capsule_coverage(positions, &cap);
            if sc >= cc {
                s
            } else {
                cap
            }
        }
    };

    let coverage = match &shape {
        ProxyShape::Sphere { .. } => sphere_coverage(positions, &shape),
        ProxyShape::Capsule { .. } => capsule_coverage(positions, &shape),
        ProxyShape::Aabb { min, max } => {
            if positions.is_empty() {
                0.0
            } else {
                let inside = positions
                    .iter()
                    .filter(|p| {
                        p[0] >= min[0]
                            && p[0] <= max[0]
                            && p[1] >= min[1]
                            && p[1] <= max[1]
                            && p[2] >= min[2]
                            && p[2] <= max[2]
                    })
                    .count();
                inside as f32 / positions.len() as f32
            }
        }
        ProxyShape::OrientedBox { .. } => 1.0, // OBB by construction covers all
    };

    // Compactness: ratio of mesh AABB volume to proxy volume (capped at 1)
    let compactness = if positions.is_empty() {
        0.0
    } else {
        let aabb = fit_aabb(positions);
        let mesh_vol = aabb.volume();
        let proxy_vol = shape.volume();
        if proxy_vol < 1e-12 {
            1.0
        } else {
            (mesh_vol / proxy_vol).min(1.0)
        }
    };

    ProxyFitResult {
        shape,
        coverage,
        compactness,
    }
}

// ---------------------------------------------------------------------------
// Mesh generation helpers
// ---------------------------------------------------------------------------

fn make_mesh(positions: Vec<[f32; 3]>, uvs: Vec<[f32; 2]>, indices: Vec<u32>) -> MeshBuffers {
    let n = positions.len();
    let mut m = MeshBuffers {
        positions,
        normals: vec![[0.0, 1.0, 0.0]; n],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
        uvs,
        indices,
        colors: None,
        has_suit: false,
    };
    compute_normals(&mut m);
    m
}

// ---------------------------------------------------------------------------
// proxy_to_mesh
// ---------------------------------------------------------------------------

/// Convert a proxy shape to a visualization mesh.
pub fn proxy_to_mesh(shape: &ProxyShape, segments: usize) -> MeshBuffers {
    match shape {
        ProxyShape::Sphere { center, radius } => sphere_mesh(*center, *radius, segments),
        ProxyShape::Capsule { p0, p1, radius } => capsule_mesh(*p0, *p1, *radius, segments),
        ProxyShape::Aabb { min, max } => box_mesh(*min, *max),
        ProxyShape::OrientedBox {
            center,
            half_extents,
            axes,
        } => {
            // Construct 8 corners in world space
            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut uvs: Vec<[f32; 2]> = Vec::new();
            let mut indices = Vec::new();

            // 8 corners
            let corners: Vec<[f32; 3]> = {
                let mut v = Vec::new();
                for sx in [-1.0f32, 1.0] {
                    for sy in [-1.0f32, 1.0] {
                        for sz in [-1.0f32, 1.0] {
                            v.push(vec3_add(
                                *center,
                                vec3_add(
                                    vec3_add(
                                        vec3_scale(axes[0], sx * half_extents[0]),
                                        vec3_scale(axes[1], sy * half_extents[1]),
                                    ),
                                    vec3_scale(axes[2], sz * half_extents[2]),
                                ),
                            ));
                        }
                    }
                }
                v
            };

            // 6 faces, each as 2 triangles.
            // Corner indices in (sx, sy, sz) order: (−,−,−)=0 (−,−,+)=1 (−,+,−)=2 (−,+,+)=3
            //                                        (+,−,−)=4 (+,−,+)=5 (+,+,−)=6 (+,+,+)=7
            let faces: &[[usize; 4]] = &[
                [0, 1, 3, 2], // -X face
                [4, 6, 7, 5], // +X face
                [0, 4, 5, 1], // -Y face
                [2, 3, 7, 6], // +Y face
                [0, 2, 6, 4], // -Z face
                [1, 5, 7, 3], // +Z face
            ];
            let face_normals: [[f32; 3]; 6] = [
                vec3_scale(axes[0], -1.0),
                axes[0],
                vec3_scale(axes[1], -1.0),
                axes[1],
                vec3_scale(axes[2], -1.0),
                axes[2],
            ];

            for (fi, face) in faces.iter().enumerate() {
                let base = positions.len() as u32;
                let face_uv = [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
                for (vi, &ci) in face.iter().enumerate() {
                    positions.push(corners[ci]);
                    normals.push(face_normals[fi]);
                    uvs.push(face_uv[vi]);
                }
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }

            let n = positions.len();
            MeshBuffers {
                positions,
                normals,
                tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
                uvs,
                indices,
                colors: None,
                has_suit: false,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// sphere_mesh
// ---------------------------------------------------------------------------

/// Generate a UV sphere mesh at `center` with `radius`.
/// `segments` controls both stacks and slices (minimum 3).
pub fn sphere_mesh(center: [f32; 3], radius: f32, segments: usize) -> MeshBuffers {
    let seg = segments.max(3);
    let stacks = seg;
    let slices = seg;
    let row = slices + 1;

    let mut positions = Vec::new();
    let mut uvs = Vec::new();

    for i in 0..=stacks {
        let phi = PI * i as f32 / stacks as f32;
        let sp = phi.sin();
        let cp = phi.cos();
        for j in 0..=slices {
            let theta = 2.0 * PI * j as f32 / slices as f32;
            let x = center[0] + radius * sp * theta.cos();
            let y = center[1] + radius * cp;
            let z = center[2] + radius * sp * theta.sin();
            positions.push([x, y, z]);
            uvs.push([j as f32 / slices as f32, i as f32 / stacks as f32]);
        }
    }

    let mut indices = Vec::new();
    for i in 0..stacks {
        for j in 0..slices {
            let a = (i * row + j) as u32;
            let b = (i * row + j + 1) as u32;
            let c = ((i + 1) * row + j) as u32;
            let d = ((i + 1) * row + j + 1) as u32;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    make_mesh(positions, uvs, indices)
}

// ---------------------------------------------------------------------------
// box_mesh (AABB)
// ---------------------------------------------------------------------------

/// Generate a box mesh from AABB min/max corners.
pub fn box_mesh(min: [f32; 3], max: [f32; 3]) -> MeshBuffers {
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;

    #[rustfmt::skip]
    #[allow(clippy::type_complexity)]
    let face_data: &[([f32; 3], [[f32; 3]; 4], [[f32; 2]; 4])] = &[
        // +Z front
        ([0.0, 0.0, 1.0], [[x0,y0,z1],[x1,y0,z1],[x1,y1,z1],[x0,y1,z1]],
         [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
        // -Z back
        ([0.0, 0.0,-1.0], [[x1,y0,z0],[x0,y0,z0],[x0,y1,z0],[x1,y1,z0]],
         [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
        // -X left
        ([-1.0,0.0, 0.0], [[x0,y0,z0],[x0,y0,z1],[x0,y1,z1],[x0,y1,z0]],
         [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
        // +X right
        ([ 1.0,0.0, 0.0], [[x1,y0,z1],[x1,y0,z0],[x1,y1,z0],[x1,y1,z1]],
         [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
        // +Y top
        ([0.0, 1.0, 0.0], [[x0,y1,z1],[x1,y1,z1],[x1,y1,z0],[x0,y1,z0]],
         [[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0]]),
        // -Y bottom
        ([0.0,-1.0, 0.0], [[x0,y0,z0],[x1,y0,z0],[x1,y0,z1],[x0,y0,z1]],
         [[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0]]),
    ];

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices = Vec::new();

    for (normal, corners, face_uvs) in face_data {
        let base = positions.len() as u32;
        for (corner, uv) in corners.iter().zip(face_uvs.iter()) {
            positions.push(*corner);
            normals.push(*normal);
            uvs.push(*uv);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    let n = positions.len();
    MeshBuffers {
        positions,
        normals,
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
        uvs,
        indices,
        colors: None,
        has_suit: false,
    }
}

// ---------------------------------------------------------------------------
// capsule_mesh
// ---------------------------------------------------------------------------

/// Generate a capsule mesh between p0 and p1 with given radius.
/// `segments` controls radial resolution (minimum 3); hemisphere rings = segments/2.
pub fn capsule_mesh(p0: [f32; 3], p1: [f32; 3], radius: f32, segments: usize) -> MeshBuffers {
    let seg = segments.max(3);
    let rings = (seg / 2).max(2);
    let row = seg + 1;

    // Axis direction
    let axis = {
        let d = vec3_sub(p1, p0);
        let l = vec3_len(d);
        if l < 1e-10 {
            [0.0f32, 1.0, 0.0]
        } else {
            vec3_scale(d, 1.0 / l)
        }
    };

    // Build local frame (axis, u, v)
    let u = orthogonal(axis);
    let v = cross(axis, u);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // ── top hemisphere (around p1) ──────────────────────────────────────
    for i in 0..=rings {
        let phi = PI * 0.5 * i as f32 / rings as f32; // 0 (pole at p1) → PI/2 (equator)
        let sp = phi.sin();
        let cp = phi.cos();
        for j in 0..=seg {
            let theta = 2.0 * PI * j as f32 / seg as f32;
            // local sphere coords, axis = "up" direction
            let local = vec3_add(
                vec3_add(
                    vec3_scale(u, sp * theta.cos()),
                    vec3_scale(v, sp * theta.sin()),
                ),
                vec3_scale(axis, cp),
            );
            let world = vec3_add(p1, vec3_scale(local, radius));
            positions.push(world);
            uvs.push([j as f32 / seg as f32, i as f32 / (2 * rings + 1) as f32]);
        }
    }
    // top hemisphere indices
    for i in 0..rings {
        for j in 0..seg {
            let a = (i * row + j) as u32;
            let b = (i * row + j + 1) as u32;
            let c = ((i + 1) * row + j) as u32;
            let d = ((i + 1) * row + j + 1) as u32;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    // ── cylinder body ───────────────────────────────────────────────────
    let cyl_top_offset = (rings * row) as u32;
    let cyl_bot_offset = positions.len() as u32;

    for j in 0..=seg {
        let theta = 2.0 * PI * j as f32 / seg as f32;
        let local = vec3_add(vec3_scale(u, theta.cos()), vec3_scale(v, theta.sin()));
        let world = vec3_add(p0, vec3_scale(local, radius));
        positions.push(world);
        uvs.push([
            j as f32 / seg as f32,
            (rings + 1) as f32 / (2 * rings + 1) as f32,
        ]);
    }

    for j in 0..seg {
        let a = cyl_top_offset + j as u32;
        let b = cyl_top_offset + j as u32 + 1;
        let c = cyl_bot_offset + j as u32;
        let d = cyl_bot_offset + j as u32 + 1;
        indices.extend_from_slice(&[a, c, d, a, d, b]);
    }

    // ── bottom hemisphere (around p0) ───────────────────────────────────
    let bot_offset = positions.len() as u32;
    for i in 1..=rings {
        let phi = PI * 0.5 + PI * 0.5 * i as f32 / rings as f32; // PI/2 → PI
        let sp = phi.sin();
        let cp = phi.cos();
        for j in 0..=seg {
            let theta = 2.0 * PI * j as f32 / seg as f32;
            let local = vec3_add(
                vec3_add(
                    vec3_scale(u, sp * theta.cos()),
                    vec3_scale(v, sp * theta.sin()),
                ),
                vec3_scale(axis, cp),
            );
            let world = vec3_add(p0, vec3_scale(local, radius));
            positions.push(world);
            uvs.push([
                j as f32 / seg as f32,
                (rings + 1 + i) as f32 / (2 * rings + 1) as f32,
            ]);
        }
    }

    // Connect cyl bottom ring → bottom hemisphere
    for i in 0..rings {
        let top_base = if i == 0 {
            cyl_bot_offset
        } else {
            bot_offset + ((i - 1) * row) as u32
        };
        let bot_base = if i == 0 {
            bot_offset
        } else {
            bot_offset + (i * row) as u32
        };
        for j in 0..seg {
            let a = top_base + j as u32;
            let b = top_base + j as u32 + 1;
            let c = bot_base + j as u32;
            let d = bot_base + j as u32 + 1;
            indices.extend_from_slice(&[a, c, d, a, d, b]);
        }
    }

    make_mesh(positions, uvs, indices)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn simple_mesh(positions: Vec<[f32; 3]>) -> MeshBuffers {
        let n = positions.len();
        // build a trivial fan triangle for each triple
        let indices: Vec<u32> = if n >= 3 {
            (0..((n / 3) as u32))
                .flat_map(|i| [i * 3, i * 3 + 1, i * 3 + 2])
                .collect()
        } else {
            vec![]
        };
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            has_suit: false,
        })
    }

    #[test]
    fn test_fit_aabb_basic() {
        let pts = vec![[0.0f32, 0.0, 0.0], [1.0, 2.0, 3.0], [-1.0, -2.0, -3.0]];
        let shape = fit_aabb(&pts);
        if let ProxyShape::Aabb { min, max } = shape {
            assert!((min[0] - (-1.0)).abs() < 1e-5);
            assert!((min[1] - (-2.0)).abs() < 1e-5);
            assert!((min[2] - (-3.0)).abs() < 1e-5);
            assert!((max[0] - 1.0).abs() < 1e-5);
            assert!((max[1] - 2.0).abs() < 1e-5);
            assert!((max[2] - 3.0).abs() < 1e-5);
        } else {
            panic!("expected Aabb");
        }
    }

    #[test]
    fn test_fit_sphere_single_point() {
        let pts = vec![[1.0f32, 2.0, 3.0]];
        let shape = fit_sphere(&pts);
        if let ProxyShape::Sphere { center, radius } = shape {
            assert_eq!(center, [1.0, 2.0, 3.0]);
            assert!(radius < 1e-5);
        } else {
            panic!("expected Sphere");
        }
    }

    #[test]
    fn test_fit_sphere_multiple() {
        // Unit sphere vertices — all at distance 1.0 from origin
        let pts: Vec<[f32; 3]> = vec![
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];
        let shape = fit_sphere(&pts);
        if let ProxyShape::Sphere { center, radius } = shape {
            let c_len = vec3_len(center);
            assert!(
                c_len < 0.1,
                "center should be near origin, got {:?}",
                center
            );
            assert!(
                (radius - 1.0).abs() < 0.1,
                "radius should be ~1.0, got {}",
                radius
            );
        } else {
            panic!("expected Sphere");
        }
    }

    #[test]
    fn test_fit_capsule_vertical() {
        // Points along Y axis
        let pts: Vec<[f32; 3]> = (0..10).map(|i| [0.0f32, i as f32 * 0.1, 0.0]).collect();
        let shape = fit_capsule(&pts);
        if let ProxyShape::Capsule { p0, p1, radius } = shape {
            // Axis should be mostly along Y
            let axis = vec3_normalize(vec3_sub(p1, p0));
            let y_component = axis[1].abs();
            assert!(
                y_component > 0.9,
                "axis should be ~Y, got {:?}, y_component={}",
                axis,
                y_component
            );
            // Radius should be very small
            assert!(
                radius < 1e-4,
                "radius for collinear points should be ~0, got {}",
                radius
            );
        } else {
            panic!("expected Capsule");
        }
    }

    #[test]
    fn test_fit_obb_basic() {
        let pts = vec![
            [-1.0f32, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];
        let shape = fit_obb(&pts);
        if let ProxyShape::OrientedBox {
            center,
            half_extents,
            ..
        } = shape
        {
            // Center should be near origin
            let c_len = vec3_len(center);
            assert!(
                c_len < 0.1,
                "center should be near origin, got {:?}",
                center
            );
            // All half-extents should be ~1.0
            for (i, he) in half_extents.iter().enumerate() {
                assert!(
                    (he - 1.0).abs() < 0.1,
                    "half_extent[{}] should be ~1.0, got {}",
                    i,
                    he
                );
            }
        } else {
            panic!("expected OrientedBox");
        }
    }

    #[test]
    fn test_proxy_shape_volume_sphere() {
        let s = ProxyShape::Sphere {
            center: [0.0; 3],
            radius: 1.0,
        };
        let expected = (4.0 / 3.0) * PI;
        assert!(
            (s.volume() - expected).abs() < 1e-4,
            "sphere volume: got {}, expected {}",
            s.volume(),
            expected
        );
    }

    #[test]
    fn test_proxy_shape_volume_aabb() {
        let a = ProxyShape::Aabb {
            min: [0.0; 3],
            max: [2.0, 3.0, 4.0],
        };
        assert!(
            (a.volume() - 24.0).abs() < 1e-4,
            "aabb volume: got {}, expected 24.0",
            a.volume()
        );
    }

    #[test]
    fn test_proxy_shape_center_sphere() {
        let s = ProxyShape::Sphere {
            center: [1.0, 2.0, 3.0],
            radius: 0.5,
        };
        assert_eq!(s.center(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_proxy_shape_center_capsule() {
        let c = ProxyShape::Capsule {
            p0: [0.0, 0.0, 0.0],
            p1: [0.0, 2.0, 0.0],
            radius: 0.5,
        };
        let ctr = c.center();
        assert!(
            (ctr[1] - 1.0).abs() < 1e-5,
            "capsule center y should be 1.0, got {}",
            ctr[1]
        );
    }

    #[test]
    fn test_sphere_mesh_basic() {
        let m = sphere_mesh([0.0, 0.0, 0.0], 1.0, 8);
        assert!(!m.positions.is_empty(), "sphere mesh has no vertices");
        assert!(!m.indices.is_empty(), "sphere mesh has no indices");
        // All indices valid
        let n = m.positions.len() as u32;
        for &idx in &m.indices {
            assert!(idx < n, "out-of-bounds index {}", idx);
        }
        // All positions on unit sphere
        for p in &m.positions {
            let l = vec3_len(*p);
            assert!(
                (l - 1.0).abs() < 1e-3,
                "sphere position {:?} has length {}",
                p,
                l
            );
        }
    }

    #[test]
    fn test_box_mesh_basic() {
        let m = box_mesh([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(m.indices.len() / 3, 12, "box should have 12 triangles");
        let n = m.positions.len() as u32;
        for &idx in &m.indices {
            assert!(idx < n, "out-of-bounds index {}", idx);
        }
    }

    #[test]
    fn test_capsule_mesh_basic() {
        let m = capsule_mesh([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], 0.5, 8);
        assert!(!m.positions.is_empty(), "capsule mesh has no vertices");
        assert!(!m.indices.is_empty(), "capsule mesh has no indices");
        let n = m.positions.len() as u32;
        for &idx in &m.indices {
            assert!(idx < n, "out-of-bounds index {}", idx);
        }
    }

    #[test]
    fn test_mesh_proxy_sphere() {
        // Build a mesh that is roughly a unit sphere
        let pts: Vec<[f32; 3]> = vec![
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];
        let mesh = simple_mesh(pts);
        let result = mesh_proxy(&mesh, ProxyShapeType::Sphere);
        assert!(
            result.coverage > 0.9,
            "coverage should be ~1.0, got {}",
            result.coverage
        );
        if let ProxyShape::Sphere { radius, .. } = result.shape {
            assert!(
                (radius - 1.0).abs() < 0.2,
                "sphere radius should be ~1.0, got {}",
                radius
            );
        } else {
            panic!("expected Sphere");
        }
    }
}
