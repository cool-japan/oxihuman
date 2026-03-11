// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bezier/B-spline patch representation for smooth surface modelling.
//!
//! Provides a bicubic Bezier patch (4×4 control-point grid) with de Casteljau
//! evaluation, normal computation, tessellation, subdivision, and bounding-box
//! queries.  All tessellation and subdivision functions operate entirely on
//! plain Rust types; no external dependencies are required.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

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
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        scale3(v, 1.0 / l)
    }
}

/// Lerp between two 3-D points.
#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    add3(scale3(a, 1.0 - t), scale3(b, t))
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// A bicubic Bezier patch defined by a 4×4 grid of control points.
///
/// Control points are stored in row-major order: `ctrl[row][col]` where
/// `row` indexes the *v* direction and `col` indexes the *u* direction.
#[allow(dead_code)]
pub struct BezierPatch {
    /// 4×4 grid of control points in world space.
    pub ctrl: [[f32; 3]; 16],
}

/// Configuration for patch tessellation and evaluation.
#[allow(dead_code)]
pub struct PatchConfig {
    /// Number of subdivisions per parameter axis (resolution × resolution quads).
    pub resolution: u32,
    /// Finite-difference step for normal/tangent estimation.
    pub normal_eps: f32,
    /// Whether to flip the computed normal.
    pub flip_normal: bool,
}

/// A single evaluated sample on a patch surface.
#[allow(dead_code)]
pub struct PatchSample {
    /// World-space position on the patch.
    pub position: [f32; 3],
    /// Surface normal (unit length).
    pub normal: [f32; 3],
    /// Tangent in the *u* direction.
    pub tangent_u: [f32; 3],
    /// Tangent in the *v* direction.
    pub tangent_v: [f32; 3],
    /// Parameter values that produced this sample.
    pub uv: [f32; 2],
}

/// Tessellated patch output (triangles only).
#[allow(dead_code)]
pub struct PatchTessellation {
    /// Flat list of positions (one per vertex).
    pub positions: Vec<[f32; 3]>,
    /// Flat list of normals (one per vertex, matching `positions`).
    pub normals: Vec<[f32; 3]>,
    /// Flat list of UV parameter coords (one pair per vertex).
    pub uvs: Vec<[f32; 2]>,
    /// Triangle indices into `positions`/`normals`/`uvs`.
    pub indices: Vec<u32>,
}

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Four sub-patches produced by `subdivide_patch`.
pub type SubPatches = [BezierPatch; 4];

// ---------------------------------------------------------------------------
// Constructor helpers
// ---------------------------------------------------------------------------

/// Return a default `PatchConfig` (8-division resolution).
#[allow(dead_code)]
pub fn default_patch_config() -> PatchConfig {
    PatchConfig {
        resolution: 8,
        normal_eps: 1e-4,
        flip_normal: false,
    }
}

/// Create a new `BezierPatch` from a flat array of 16 control points.
///
/// `ctrl` is row-major: index `r*4 + c` gives control point (row=r, col=c).
#[allow(dead_code)]
pub fn new_bezier_patch(ctrl: [[f32; 3]; 16]) -> BezierPatch {
    BezierPatch { ctrl }
}

// ---------------------------------------------------------------------------
// Core evaluation  (de Casteljau)
// ---------------------------------------------------------------------------

/// Evaluate a cubic Bezier curve at parameter `t` given 4 control points.
fn casteljau_cubic(p: &[[f32; 3]; 4], t: f32) -> [f32; 3] {
    let q0 = lerp3(p[0], p[1], t);
    let q1 = lerp3(p[1], p[2], t);
    let q2 = lerp3(p[2], p[3], t);
    let r0 = lerp3(q0, q1, t);
    let r1 = lerp3(q1, q2, t);
    lerp3(r0, r1, t)
}

// Row4 is a 4-element array of 3-D points — used as a "row" or "column"
// of the 4×4 control grid.
type Row4 = [[f32; 3]; 4];

fn row_ctrl_v(patch: &BezierPatch, row: usize) -> Row4 {
    [
        patch.ctrl[row * 4],
        patch.ctrl[row * 4 + 1],
        patch.ctrl[row * 4 + 2],
        patch.ctrl[row * 4 + 3],
    ]
}

fn col_ctrl_v(patch: &BezierPatch, col: usize) -> Row4 {
    [
        patch.ctrl[col],
        patch.ctrl[4 + col],
        patch.ctrl[8 + col],
        patch.ctrl[12 + col],
    ]
}

/// Evaluate the bicubic Bezier patch at parameter `(u, v)` using de Casteljau.
///
/// First reduce each of the 4 rows along `u`, then reduce the resulting column
/// along `v`.
#[allow(dead_code)]
pub fn evaluate_patch(patch: &BezierPatch, u: f32, v: f32) -> [f32; 3] {
    let row_pts: Row4 = [
        casteljau_cubic(&row_ctrl_v(patch, 0), u),
        casteljau_cubic(&row_ctrl_v(patch, 1), u),
        casteljau_cubic(&row_ctrl_v(patch, 2), u),
        casteljau_cubic(&row_ctrl_v(patch, 3), u),
    ];
    casteljau_cubic(&row_pts, v)
}

// ---------------------------------------------------------------------------
// Tangents
// ---------------------------------------------------------------------------

/// Compute the tangent vector in the *u* direction at `(u, v)`.
///
/// Uses a central finite difference with step size from `PatchConfig::normal_eps`.
#[allow(dead_code)]
pub fn patch_tangent_u(patch: &BezierPatch, u: f32, v: f32, cfg: &PatchConfig) -> [f32; 3] {
    let eps = cfg.normal_eps;
    let u0 = (u - eps).clamp(0.0, 1.0);
    let u1 = (u + eps).clamp(0.0, 1.0);
    let p0 = evaluate_patch(patch, u0, v);
    let p1 = evaluate_patch(patch, u1, v);
    normalize3(sub3(p1, p0))
}

/// Compute the tangent vector in the *v* direction at `(u, v)`.
#[allow(dead_code)]
pub fn patch_tangent_v(patch: &BezierPatch, u: f32, v: f32, cfg: &PatchConfig) -> [f32; 3] {
    let eps = cfg.normal_eps;
    let v0 = (v - eps).clamp(0.0, 1.0);
    let v1 = (v + eps).clamp(0.0, 1.0);
    let p0 = evaluate_patch(patch, u, v0);
    let p1 = evaluate_patch(patch, u, v1);
    normalize3(sub3(p1, p0))
}

/// Compute the surface normal at `(u, v)` as the cross product of the two
/// partial derivative directions.
#[allow(dead_code)]
pub fn patch_normal(patch: &BezierPatch, u: f32, v: f32, cfg: &PatchConfig) -> [f32; 3] {
    let tu = patch_tangent_u(patch, u, v, cfg);
    let tv = patch_tangent_v(patch, u, v, cfg);
    let n = normalize3(cross3(tu, tv));
    if cfg.flip_normal {
        scale3(n, -1.0)
    } else {
        n
    }
}

// ---------------------------------------------------------------------------
// Bounding box / midpoint
// ---------------------------------------------------------------------------

/// Compute the axis-aligned bounding box of all 16 control points.
///
/// Returns `(min, max)`.
#[allow(dead_code)]
pub fn patch_bounding_box(patch: &BezierPatch) -> ([f32; 3], [f32; 3]) {
    let mut mn = patch.ctrl[0];
    let mut mx = patch.ctrl[0];
    for &p in &patch.ctrl[1..] {
        mn[0] = mn[0].min(p[0]);
        mn[1] = mn[1].min(p[1]);
        mn[2] = mn[2].min(p[2]);
        mx[0] = mx[0].max(p[0]);
        mx[1] = mx[1].max(p[1]);
        mx[2] = mx[2].max(p[2]);
    }
    (mn, mx)
}

/// Return the centroid of the 16 control points (approximate patch centre).
#[allow(dead_code)]
pub fn patch_midpoint(patch: &BezierPatch) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    for p in &patch.ctrl {
        sum = add3(sum, *p);
    }
    scale3(sum, 1.0 / 16.0)
}

// ---------------------------------------------------------------------------
// Tessellation
// ---------------------------------------------------------------------------

/// Number of vertices produced by `tessellate_patch` at `resolution`.
#[allow(dead_code)]
pub fn patch_vertex_count(resolution: u32) -> usize {
    let n = resolution as usize + 1;
    n * n
}

/// Number of triangles produced by `tessellate_patch` at `resolution`.
#[allow(dead_code)]
pub fn patch_triangle_count(resolution: u32) -> usize {
    let n = resolution as usize;
    n * n * 2
}

/// Tessellate the patch into a triangle mesh with `cfg.resolution` divisions.
///
/// Returns a `PatchTessellation` containing positions, normals, UVs and indices.
#[allow(dead_code)]
pub fn tessellate_patch(patch: &BezierPatch, cfg: &PatchConfig) -> PatchTessellation {
    let res = cfg.resolution.max(1) as usize;
    let n = res + 1;

    let mut positions = Vec::with_capacity(n * n);
    let mut normals = Vec::with_capacity(n * n);
    let mut uvs = Vec::with_capacity(n * n);

    for row in 0..n {
        let v = row as f32 / res as f32;
        for col in 0..n {
            let u = col as f32 / res as f32;
            positions.push(evaluate_patch(patch, u, v));
            normals.push(patch_normal(patch, u, v, cfg));
            uvs.push([u, v]);
        }
    }

    let mut indices = Vec::with_capacity(res * res * 6);
    for row in 0..res {
        for col in 0..res {
            let i0 = (row * n + col) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + n as u32;
            let i3 = i2 + 1;
            // Triangle 1
            indices.push(i0);
            indices.push(i1);
            indices.push(i3);
            // Triangle 2
            indices.push(i0);
            indices.push(i3);
            indices.push(i2);
        }
    }

    PatchTessellation {
        positions,
        normals,
        uvs,
        indices,
    }
}

// ---------------------------------------------------------------------------
// Subdivision (split into 4 sub-patches)
// ---------------------------------------------------------------------------

/// Split a single cubic Bezier curve (4 control points) at `t = 0.5`.
///
/// Returns `(left_ctrl, right_ctrl)` where each is a 4-element array.
fn split_cubic_at_half(p: &Row4) -> (Row4, Row4) {
    let q0 = lerp3(p[0], p[1], 0.5);
    let q1 = lerp3(p[1], p[2], 0.5);
    let q2 = lerp3(p[2], p[3], 0.5);
    let r0 = lerp3(q0, q1, 0.5);
    let r1 = lerp3(q1, q2, 0.5);
    let s = lerp3(r0, r1, 0.5);
    ([p[0], q0, r0, s], [s, r1, q2, p[3]])
}

/// Subdivide the patch at the midpoint of both parameter axes, producing four
/// equal sub-patches in (u<0.5,v<0.5), (u>0.5,v<0.5), (u<0.5,v>0.5), (u>0.5,v>0.5) order.
#[allow(dead_code)]
pub fn subdivide_patch(patch: &BezierPatch) -> SubPatches {
    // Step 1: split each row along u → produces 4 pairs of 4-point curves.
    let mut left_rows: [Row4; 4] = [[patch.ctrl[0]; 4]; 4];
    let mut right_rows: [Row4; 4] = [[patch.ctrl[0]; 4]; 4];
    for r in 0..4usize {
        let row = row_ctrl_v(patch, r);
        let (l, ri) = split_cubic_at_half(&row);
        left_rows[r] = l;
        right_rows[r] = ri;
    }

    // Helper: build a BezierPatch from a 4×4 stored as rows.
    let from_rows = |rows: &[Row4; 4]| -> BezierPatch {
        let mut ctrl = [[0.0f32; 3]; 16];
        for (r, row) in rows.iter().enumerate() {
            for (c, &pt) in row.iter().enumerate() {
                ctrl[r * 4 + c] = pt;
            }
        }
        BezierPatch { ctrl }
    };

    // Step 2: for each of the two halves (left/right), split columns along v.
    let split_cols = |rows: &[Row4; 4]| -> (BezierPatch, BezierPatch) {
        // Treat each column as a cubic curve and split.
        let mut bot_rows: [Row4; 4] = [rows[0]; 4];
        let mut top_rows: [Row4; 4] = [rows[0]; 4];
        for c in 0..4usize {
            let col: Row4 = [rows[0][c], rows[1][c], rows[2][c], rows[3][c]];
            let (b, t) = split_cubic_at_half(&col);
            for r in 0..4 {
                bot_rows[r][c] = b[r];
                top_rows[r][c] = t[r];
            }
        }
        (from_rows(&bot_rows), from_rows(&top_rows))
    };

    let (ll, lu) = split_cols(&left_rows);
    let (rl, ru) = split_cols(&right_rows);
    [ll, rl, lu, ru]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_patch() -> BezierPatch {
        // Flat unit patch in XZ plane (y=0)
        let mut ctrl = [[0.0f32; 3]; 16];
        for r in 0..4usize {
            for c in 0..4usize {
                ctrl[r * 4 + c] = [c as f32 / 3.0, 0.0, r as f32 / 3.0];
            }
        }
        BezierPatch { ctrl }
    }

    #[test]
    fn test_default_patch_config() {
        let cfg = default_patch_config();
        assert_eq!(cfg.resolution, 8);
        assert!(!cfg.flip_normal);
    }

    #[test]
    fn test_new_bezier_patch_corners() {
        let mut ctrl = [[0.0f32; 3]; 16];
        ctrl[0] = [1.0, 2.0, 3.0];
        let p = new_bezier_patch(ctrl);
        assert_eq!(p.ctrl[0], [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_evaluate_patch_corners() {
        let p = flat_patch();
        // u=0,v=0 → ctrl[0,0] = [0,0,0]
        let pt = evaluate_patch(&p, 0.0, 0.0);
        assert!((pt[0]).abs() < 1e-5, "x should be ~0, got {}", pt[0]);
        assert!((pt[1]).abs() < 1e-5);
        assert!((pt[2]).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_patch_far_corner() {
        let p = flat_patch();
        let pt = evaluate_patch(&p, 1.0, 1.0);
        // Should be at [1,0,1]
        assert!((pt[0] - 1.0).abs() < 1e-5, "x={}", pt[0]);
        assert!((pt[2] - 1.0).abs() < 1e-5, "z={}", pt[2]);
    }

    #[test]
    fn test_patch_tangent_u_not_zero() {
        let p = flat_patch();
        let cfg = default_patch_config();
        let tu = patch_tangent_u(&p, 0.5, 0.5, &cfg);
        let mag = len3(tu);
        assert!(mag > 0.5, "tangent_u should be nonzero");
    }

    #[test]
    fn test_patch_tangent_v_not_zero() {
        let p = flat_patch();
        let cfg = default_patch_config();
        let tv = patch_tangent_v(&p, 0.5, 0.5, &cfg);
        let mag = len3(tv);
        assert!(mag > 0.5, "tangent_v should be nonzero");
    }

    #[test]
    fn test_patch_normal_approximately_up() {
        // Flat XZ patch → normal should point in Y direction.
        let p = flat_patch();
        let cfg = default_patch_config();
        let n = patch_normal(&p, 0.5, 0.5, &cfg);
        // Either +Y or -Y depending on winding; magnitude should be ~1.
        assert!((len3(n) - 1.0).abs() < 0.02, "normal mag={}", len3(n));
        assert!(n[1].abs() > 0.9, "y component={}", n[1]);
    }

    #[test]
    fn test_patch_normal_flip() {
        let p = flat_patch();
        let cfg_no = default_patch_config();
        let mut cfg_flip = default_patch_config();
        cfg_flip.flip_normal = true;
        let n_no = patch_normal(&p, 0.5, 0.5, &cfg_no);
        let n_flip = patch_normal(&p, 0.5, 0.5, &cfg_flip);
        assert!((n_no[0] + n_flip[0]).abs() < 1e-5);
        assert!((n_no[1] + n_flip[1]).abs() < 1e-5);
        assert!((n_no[2] + n_flip[2]).abs() < 1e-5);
    }

    #[test]
    fn test_patch_bounding_box() {
        let p = flat_patch();
        let (mn, mx) = patch_bounding_box(&p);
        assert!((mn[0]).abs() < 1e-5);
        assert!((mx[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_patch_midpoint() {
        let p = flat_patch();
        let mid = patch_midpoint(&p);
        // Centre of [0..1]×[0..1] = [0.5, 0, 0.5]
        assert!((mid[0] - 0.5).abs() < 0.01, "mid.x={}", mid[0]);
        assert!((mid[2] - 0.5).abs() < 0.01, "mid.z={}", mid[2]);
    }

    #[test]
    fn test_patch_vertex_count() {
        assert_eq!(patch_vertex_count(8), 81);
        assert_eq!(patch_vertex_count(1), 4);
    }

    #[test]
    fn test_patch_triangle_count() {
        assert_eq!(patch_triangle_count(8), 128);
        assert_eq!(patch_triangle_count(1), 2);
    }

    #[test]
    fn test_tessellate_patch_counts() {
        let p = flat_patch();
        let cfg = default_patch_config();
        let tess = tessellate_patch(&p, &cfg);
        let res = cfg.resolution as usize;
        let n = res + 1;
        assert_eq!(tess.positions.len(), n * n);
        assert_eq!(tess.normals.len(), n * n);
        assert_eq!(tess.uvs.len(), n * n);
        assert_eq!(tess.indices.len(), res * res * 6);
    }

    #[test]
    fn test_tessellate_patch_resolution_1() {
        let p = flat_patch();
        let mut cfg = default_patch_config();
        cfg.resolution = 1;
        let tess = tessellate_patch(&p, &cfg);
        assert_eq!(tess.positions.len(), 4);
        assert_eq!(tess.indices.len(), 6);
    }

    #[test]
    fn test_subdivide_patch_produces_four() {
        let p = flat_patch();
        let subs = subdivide_patch(&p);
        assert_eq!(subs.len(), 4);
    }

    #[test]
    fn test_subdivide_preserves_corners() {
        // The union of the four sub-patches should cover the same bounding box.
        let p = flat_patch();
        let subs = subdivide_patch(&p);
        let (orig_mn, orig_mx) = patch_bounding_box(&p);
        let mut all_mn = subs[0].ctrl[0];
        let mut all_mx = subs[0].ctrl[0];
        for sub in &subs {
            for &pt in &sub.ctrl {
                all_mn[0] = all_mn[0].min(pt[0]);
                all_mn[1] = all_mn[1].min(pt[1]);
                all_mn[2] = all_mn[2].min(pt[2]);
                all_mx[0] = all_mx[0].max(pt[0]);
                all_mx[1] = all_mx[1].max(pt[1]);
                all_mx[2] = all_mx[2].max(pt[2]);
            }
        }
        assert!((all_mn[0] - orig_mn[0]).abs() < 1e-5);
        assert!((all_mx[0] - orig_mx[0]).abs() < 1e-5);
    }

    #[test]
    fn test_subdivide_midpoint_continuity() {
        // The midpoint evaluated on the original should match midpoint of sub-patches.
        let p = flat_patch();
        let mid_orig = evaluate_patch(&p, 0.5, 0.5);
        let subs = subdivide_patch(&p);
        // The shared corner of all four sub-patches is their "corner at split".
        // Sub-patch 0 (lower-left): its (u=1,v=1) corner should be the midpoint.
        let mid_sub = evaluate_patch(&subs[0], 1.0, 1.0);
        assert!((mid_orig[0] - mid_sub[0]).abs() < 1e-4, "x diff");
        assert!((mid_orig[1] - mid_sub[1]).abs() < 1e-4, "y diff");
        assert!((mid_orig[2] - mid_sub[2]).abs() < 1e-4, "z diff");
    }
}
