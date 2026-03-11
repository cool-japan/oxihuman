// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Sculpt-brush deformation tools.
//!
//! Provides grab, smooth, inflate, pinch, flatten, and crease brushes that
//! operate directly on [`MeshBuffers`] vertex positions and normals.

use crate::mesh::MeshBuffers;

// ── Helper math ───────────────────────────────────────────────────────────────

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
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Which sculpt brush algorithm to apply.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BrushType {
    /// Move vertices toward cursor delta.
    Grab,
    /// Average neighbours toward centroid.
    Smooth,
    /// Push vertices along their normals.
    Inflate,
    /// Pull vertices toward brush center.
    Pinch,
    /// Project vertices onto average plane.
    Flatten,
    /// Sharpen by moving toward/away from center.
    Crease,
}

/// Parameters shared by all brush types.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrushParams {
    pub brush_type: BrushType,
    /// World-space radius of the brush.
    pub radius: f32,
    /// Effect scale in 0..1.
    pub strength: f32,
    /// 0 = hard step falloff, 1 = smooth quadratic falloff.
    pub falloff: f32,
    /// Reverse the direction of the brush effect.
    pub invert: bool,
}

impl Default for BrushParams {
    fn default() -> Self {
        Self {
            brush_type: BrushType::Grab,
            radius: 0.1,
            strength: 0.5,
            falloff: 1.0,
            invert: false,
        }
    }
}

/// Summary of one brush stroke application.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrushStrokeResult {
    /// Number of vertices that were displaced.
    pub affected_count: usize,
    /// Maximum displacement magnitude applied in this stroke.
    pub max_displacement: f32,
}

/// Snapshot of a brush stroke — used for undo support.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrushStroke {
    pub params: BrushParams,
    pub center: [f32; 3],
    pub delta: [f32; 3],
    /// Vertex positions captured *before* the stroke was applied.
    pub before_positions: Vec<[f32; 3]>,
}

// ── Falloff ───────────────────────────────────────────────────────────────────

/// Compute the falloff weight for a vertex at distance `d` from the brush
/// centre.
///
/// - `falloff = 0.0`: hard step — weight is 1 inside the radius, 0 outside.
/// - `falloff = 1.0`: smooth quadratic — `(1 - (d/r)²)²`.
/// - Values in between linearly blend the two.
#[allow(dead_code)]
pub fn brush_falloff_weight(d: f32, radius: f32, falloff: f32) -> f32 {
    if radius <= 0.0 || d >= radius {
        return 0.0;
    }
    let t = d / radius; // 0..1
    let hard = 1.0f32;
    let smooth = {
        let u = 1.0 - t * t;
        u * u
    };
    let falloff = falloff.clamp(0.0, 1.0);
    hard * (1.0 - falloff) + smooth * falloff
}

// ── Adjacency ─────────────────────────────────────────────────────────────────

/// Build a per-vertex adjacency list from a triangle index buffer.
///
/// The returned `Vec<Vec<u32>>` has one entry per vertex; each entry is the
/// set of vertices directly connected by an edge.
#[allow(dead_code)]
pub fn build_adjacency(indices: &[u32], vertex_count: usize) -> Vec<Vec<u32>> {
    let mut adj: Vec<std::collections::HashSet<u32>> =
        vec![std::collections::HashSet::new(); vertex_count];

    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        let (au, bu, cu) = (a as usize, b as usize, c as usize);
        if au < vertex_count && bu < vertex_count && cu < vertex_count {
            adj[au].insert(b);
            adj[au].insert(c);
            adj[bu].insert(a);
            adj[bu].insert(c);
            adj[cu].insert(a);
            adj[cu].insert(b);
        }
    }

    adj.into_iter()
        .map(|set| {
            let mut v: Vec<u32> = set.into_iter().collect();
            v.sort_unstable();
            v
        })
        .collect()
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Collect (vertex_index, weight) pairs for all vertices within the brush.
fn gather_affected(
    positions: &[[f32; 3]],
    center: [f32; 3],
    params: &BrushParams,
) -> Vec<(usize, f32)> {
    positions
        .iter()
        .enumerate()
        .filter_map(|(i, &p)| {
            let d = len3(sub3(p, center));
            let w = brush_falloff_weight(d, params.radius, params.falloff);
            if w > 0.0 {
                let w = w * params.strength;
                Some((i, w))
            } else {
                None
            }
        })
        .collect()
}

/// Direction multiplier: -1 when `params.invert` is true, else +1.
#[inline]
fn sign(params: &BrushParams) -> f32 {
    if params.invert {
        -1.0
    } else {
        1.0
    }
}

// ── Grab ──────────────────────────────────────────────────────────────────────

/// Move affected vertices by `delta * weight`.
///
/// This simulates pulling/pushing a region of the mesh.
#[allow(dead_code)]
pub fn brush_grab(
    mesh: &mut MeshBuffers,
    center: [f32; 3],
    delta: [f32; 3],
    params: &BrushParams,
) -> BrushStrokeResult {
    let affected = gather_affected(&mesh.positions, center, params);
    let s = sign(params);
    let mut max_disp = 0.0f32;

    for (i, w) in &affected {
        let d = scale3(delta, w * s);
        mesh.positions[*i] = add3(mesh.positions[*i], d);
        max_disp = max_disp.max(len3(d));
    }

    BrushStrokeResult {
        affected_count: affected.len(),
        max_displacement: max_disp,
    }
}

// ── Smooth ────────────────────────────────────────────────────────────────────

/// Move affected vertices toward the average position of their neighbours.
#[allow(dead_code)]
pub fn brush_smooth(
    mesh: &mut MeshBuffers,
    center: [f32; 3],
    params: &BrushParams,
) -> BrushStrokeResult {
    let affected = gather_affected(&mesh.positions, center, params);
    if affected.is_empty() {
        return BrushStrokeResult {
            affected_count: 0,
            max_displacement: 0.0,
        };
    }

    // Build adjacency on the fly from the index buffer.
    let adj = build_adjacency(&mesh.indices, mesh.positions.len());

    // Snapshot positions before modification so all vertices use the same base.
    let snapshot = mesh.positions.clone();
    let s = sign(params);
    let mut max_disp = 0.0f32;

    for (i, w) in &affected {
        let neighbours = &adj[*i];
        if neighbours.is_empty() {
            continue;
        }
        // Centroid of neighbours.
        let mut centroid = [0.0f32; 3];
        for &j in neighbours {
            centroid = add3(centroid, snapshot[j as usize]);
        }
        let n = neighbours.len() as f32;
        centroid = scale3(centroid, 1.0 / n);

        let diff = sub3(centroid, snapshot[*i]);
        let displacement = scale3(diff, w * s);
        mesh.positions[*i] = add3(mesh.positions[*i], displacement);
        max_disp = max_disp.max(len3(displacement));
    }

    BrushStrokeResult {
        affected_count: affected.len(),
        max_displacement: max_disp,
    }
}

// ── Inflate ───────────────────────────────────────────────────────────────────

/// Push affected vertices along their vertex normals.
#[allow(dead_code)]
pub fn brush_inflate(
    mesh: &mut MeshBuffers,
    center: [f32; 3],
    params: &BrushParams,
) -> BrushStrokeResult {
    let affected = gather_affected(&mesh.positions, center, params);
    let s = sign(params);
    let mut max_disp = 0.0f32;

    for (i, w) in &affected {
        // Use the stored per-vertex normal as inflation direction.
        let normal = normalize3(mesh.normals[*i]);
        let displacement = scale3(normal, w * s);
        mesh.positions[*i] = add3(mesh.positions[*i], displacement);
        max_disp = max_disp.max(len3(displacement));
    }

    BrushStrokeResult {
        affected_count: affected.len(),
        max_displacement: max_disp,
    }
}

// ── Pinch ─────────────────────────────────────────────────────────────────────

/// Pull affected vertices toward the brush centre.
#[allow(dead_code)]
pub fn brush_pinch(
    mesh: &mut MeshBuffers,
    center: [f32; 3],
    params: &BrushParams,
) -> BrushStrokeResult {
    let affected = gather_affected(&mesh.positions, center, params);
    let s = sign(params);
    let mut max_disp = 0.0f32;

    for (i, w) in &affected {
        let to_center = sub3(center, mesh.positions[*i]);
        let displacement = scale3(to_center, w * s);
        mesh.positions[*i] = add3(mesh.positions[*i], displacement);
        max_disp = max_disp.max(len3(displacement));
    }

    BrushStrokeResult {
        affected_count: affected.len(),
        max_displacement: max_disp,
    }
}

// ── Flatten ───────────────────────────────────────────────────────────────────

/// Project affected vertices onto the average plane of vertices in the brush
/// region.
///
/// The average plane is defined by the centroid of the affected vertices and
/// the average normal within the brush.
#[allow(dead_code)]
pub fn brush_flatten(
    mesh: &mut MeshBuffers,
    center: [f32; 3],
    params: &BrushParams,
) -> BrushStrokeResult {
    let affected = gather_affected(&mesh.positions, center, params);
    if affected.is_empty() {
        return BrushStrokeResult {
            affected_count: 0,
            max_displacement: 0.0,
        };
    }

    // Compute centroid and average normal of the affected region.
    let mut centroid = [0.0f32; 3];
    let mut avg_normal = [0.0f32; 3];
    let total_w: f32 = affected.iter().map(|(_, w)| w).sum();

    for (i, w) in &affected {
        centroid = add3(centroid, scale3(mesh.positions[*i], *w));
        avg_normal = add3(avg_normal, scale3(mesh.normals[*i], *w));
    }
    if total_w > 0.0 {
        centroid = scale3(centroid, 1.0 / total_w);
    }
    let plane_normal = normalize3(avg_normal);

    let s = sign(params);
    let mut max_disp = 0.0f32;

    for (i, w) in &affected {
        let p = mesh.positions[*i];
        // Signed distance from p to the average plane.
        let dist = dot3(sub3(p, centroid), plane_normal);
        // Project onto plane.
        let displacement = scale3(plane_normal, -dist * w * s);
        mesh.positions[*i] = add3(p, displacement);
        max_disp = max_disp.max(len3(displacement));
    }

    BrushStrokeResult {
        affected_count: affected.len(),
        max_displacement: max_disp,
    }
}

// ── Crease (internal) ────────────────────────────────────────────────────────

/// Sharpen features: push vertices toward or away from the brush centre
/// depending on which side of the average plane they are on.
fn brush_crease_impl(
    mesh: &mut MeshBuffers,
    center: [f32; 3],
    params: &BrushParams,
) -> BrushStrokeResult {
    let affected = gather_affected(&mesh.positions, center, params);
    if affected.is_empty() {
        return BrushStrokeResult {
            affected_count: 0,
            max_displacement: 0.0,
        };
    }

    // Average normal in the region.
    let mut avg_normal = [0.0f32; 3];
    for (i, _) in &affected {
        avg_normal = add3(avg_normal, mesh.normals[*i]);
    }
    let plane_normal = normalize3(avg_normal);
    let s = sign(params);
    let mut max_disp = 0.0f32;

    for (i, w) in &affected {
        let to_center = sub3(center, mesh.positions[*i]);
        // Component of to_center along the plane normal.
        let along_n = dot3(to_center, plane_normal);
        // Crease: move further toward centre (along normal component).
        let displacement = scale3(plane_normal, along_n * w * s);
        mesh.positions[*i] = add3(mesh.positions[*i], displacement);
        max_disp = max_disp.max(len3(displacement));
    }

    BrushStrokeResult {
        affected_count: affected.len(),
        max_displacement: max_disp,
    }
}

// ── Generic dispatcher ───────────────────────────────────────────────────────

/// Apply whichever brush is specified in `params.brush_type`.
///
/// `delta` is only used by the `Grab` brush; other brushes ignore it.
#[allow(dead_code)]
pub fn apply_brush(
    mesh: &mut MeshBuffers,
    center: [f32; 3],
    delta: [f32; 3],
    params: &BrushParams,
) -> BrushStrokeResult {
    match params.brush_type {
        BrushType::Grab => brush_grab(mesh, center, delta, params),
        BrushType::Smooth => brush_smooth(mesh, center, params),
        BrushType::Inflate => brush_inflate(mesh, center, params),
        BrushType::Pinch => brush_pinch(mesh, center, params),
        BrushType::Flatten => brush_flatten(mesh, center, params),
        BrushType::Crease => brush_crease_impl(mesh, center, params),
    }
}

// ── BrushStroke (undo) ────────────────────────────────────────────────────────

impl BrushStroke {
    /// Capture the current mesh state and return a stroke record.
    #[allow(dead_code)]
    pub fn record(
        mesh: &MeshBuffers,
        center: [f32; 3],
        delta: [f32; 3],
        params: BrushParams,
    ) -> Self {
        Self {
            params,
            center,
            delta,
            before_positions: mesh.positions.clone(),
        }
    }

    /// Restore the mesh to its state before this stroke.
    #[allow(dead_code)]
    pub fn undo(&self, mesh: &mut MeshBuffers) {
        let len = mesh.positions.len().min(self.before_positions.len());
        mesh.positions[..len].copy_from_slice(&self.before_positions[..len]);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// A small quad mesh (2 triangles, 4 vertices) in the XY plane.
    fn quad_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [-0.5, -0.5, 0.0],
                [0.5, -0.5, 0.0],
                [0.5, 0.5, 0.0],
                [-0.5, 0.5, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    /// A single-triangle mesh.
    fn tri_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    fn default_params(brush_type: BrushType) -> BrushParams {
        BrushParams {
            brush_type,
            radius: 2.0,
            strength: 1.0,
            falloff: 0.0, // hard falloff so all verts within radius are hit
            invert: false,
        }
    }

    // ── Test 1: falloff hard step ─────────────────────────────────────────────

    #[test]
    fn falloff_hard_inside_returns_one() {
        let w = brush_falloff_weight(0.5, 1.0, 0.0);
        assert!(
            (w - 1.0).abs() < 1e-5,
            "hard falloff inside radius should be 1.0, got {w}"
        );
    }

    // ── Test 2: falloff outside radius ────────────────────────────────────────

    #[test]
    fn falloff_outside_radius_returns_zero() {
        let w = brush_falloff_weight(1.5, 1.0, 0.0);
        assert_eq!(w, 0.0, "outside radius must be 0.0");
    }

    // ── Test 3: falloff smooth quadratic at centre ────────────────────────────

    #[test]
    fn falloff_smooth_at_centre_returns_one() {
        // At d=0, smooth = (1-0)^2 = 1
        let w = brush_falloff_weight(0.0, 1.0, 1.0);
        assert!(
            (w - 1.0).abs() < 1e-5,
            "smooth falloff at d=0 should be 1.0, got {w}"
        );
    }

    // ── Test 4: falloff smooth at edge ────────────────────────────────────────

    #[test]
    fn falloff_smooth_at_radius_boundary() {
        // At d exactly = radius, should return 0
        let w = brush_falloff_weight(1.0, 1.0, 1.0);
        assert_eq!(w, 0.0, "at radius boundary smooth falloff should be 0");
    }

    // ── Test 5: build_adjacency single triangle ───────────────────────────────

    #[test]
    fn adjacency_single_triangle_all_connected() {
        let indices = vec![0u32, 1, 2];
        let adj = build_adjacency(&indices, 3);
        assert!(adj[0].contains(&1), "0 must be adjacent to 1");
        assert!(adj[0].contains(&2), "0 must be adjacent to 2");
        assert!(adj[1].contains(&0), "1 must be adjacent to 0");
        assert!(adj[1].contains(&2), "1 must be adjacent to 2");
        assert!(adj[2].contains(&0), "2 must be adjacent to 0");
        assert!(adj[2].contains(&1), "2 must be adjacent to 1");
    }

    // ── Test 6: grab brush moves vertices ────────────────────────────────────

    #[test]
    fn grab_brush_displaces_nearby_vertices() {
        let mut mesh = quad_mesh();
        let before = mesh.positions.clone();
        let params = default_params(BrushType::Grab);
        let result = brush_grab(&mut mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 0.1], &params);

        assert!(
            result.affected_count > 0,
            "grab should affect some vertices"
        );
        assert!(
            result.max_displacement > 0.0,
            "max displacement should be > 0"
        );

        // At least one vertex should have moved in Z.
        let moved = mesh
            .positions
            .iter()
            .zip(before.iter())
            .any(|(a, b)| (a[2] - b[2]).abs() > 1e-6);
        assert!(moved, "some vertex should have a changed Z coordinate");
    }

    // ── Test 7: grab inverted reverses direction ───────────────────────────────

    #[test]
    fn grab_brush_invert_reverses_direction() {
        let mut mesh_fwd = quad_mesh();
        let mut mesh_inv = quad_mesh();
        let mut params = default_params(BrushType::Grab);
        let delta = [0.0, 0.0, 0.1];

        brush_grab(&mut mesh_fwd, [0.0, 0.0, 0.0], delta, &params);

        params.invert = true;
        brush_grab(&mut mesh_inv, [0.0, 0.0, 0.0], delta, &params);

        // Forward moves +Z, inverted moves -Z.
        for (f, i) in mesh_fwd.positions.iter().zip(mesh_inv.positions.iter()) {
            let quad_orig = quad_mesh();
            let _ = quad_orig; // silence unused
                               // They should differ in direction if vertex was affected
            if (f[2] - i[2]).abs() > 1e-7 {
                // f moved +, i moved -
                assert!(
                    f[2] > i[2],
                    "forward Z ({}) should be > inverted Z ({})",
                    f[2],
                    i[2]
                );
            }
        }
    }

    // ── Test 8: smooth brush reduces variance ────────────────────────────────

    #[test]
    fn smooth_brush_reduces_position_variance() {
        // Create a mesh where vertex 2 is displaced far.
        // Use a large radius so all vertices are inside the brush.
        let mut mesh = MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.0, 5.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        });
        // radius=10 ensures all three vertices are inside the brush
        let params = BrushParams {
            brush_type: BrushType::Smooth,
            radius: 10.0,
            strength: 1.0,
            falloff: 0.0,
            invert: false,
        };
        let before_z2 = mesh.positions[2][2];
        brush_smooth(&mut mesh, [0.5, 0.0, 1.5], &params);
        let after_z2 = mesh.positions[2][2];

        // Vertex 2 should move toward its neighbours (which have z=0).
        assert!(
            after_z2.abs() < before_z2.abs(),
            "smooth should pull vertex 2 toward z=0, before={before_z2}, after={after_z2}"
        );
    }

    // ── Test 9: inflate brush moves along normals ─────────────────────────────

    #[test]
    fn inflate_brush_moves_along_normal() {
        let mut mesh = quad_mesh(); // normals = (0,0,1)
        let before_z: Vec<f32> = mesh.positions.iter().map(|p| p[2]).collect();
        let params = default_params(BrushType::Inflate);
        let result = brush_inflate(&mut mesh, [0.0, 0.0, 0.0], &params);

        assert!(result.affected_count > 0);
        // All affected vertices should have moved in +Z (along normal).
        for (i, (after, &bz)) in mesh.positions.iter().zip(before_z.iter()).enumerate() {
            if after[2] != bz {
                assert!(
                    after[2] > bz,
                    "inflate should push vertex {i} in +Z, before={bz}, after={}",
                    after[2]
                );
            }
        }
    }

    // ── Test 10: pinch brush pulls toward center ──────────────────────────────

    #[test]
    fn pinch_brush_pulls_toward_center() {
        let mut mesh = quad_mesh();
        let params = default_params(BrushType::Pinch);
        let center = [0.0f32, 0.0, 0.0];
        brush_pinch(&mut mesh, center, &params);

        // All vertices should be closer to (or equal to) center after pinch.
        for p in &mesh.positions {
            let d_after = len3(sub3(*p, center));
            // Original positions are at max ~0.7 from center; they should shrink.
            assert!(
                d_after <= 0.8,
                "pinched vertex distance from center ({d_after}) should be <= 0.8"
            );
        }
    }

    // ── Test 11: flatten brush reduces Z spread ───────────────────────────────

    #[test]
    fn flatten_brush_reduces_z_spread() {
        let mut mesh = MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, -0.5],
                [1.0, 0.0, 0.5],
                [0.5, 1.0, 0.3],
                [0.5, 0.5, -0.4],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        });

        let z_spread_before = {
            let zs: Vec<f32> = mesh.positions.iter().map(|p| p[2]).collect();
            let mn = zs.iter().cloned().fold(f32::INFINITY, f32::min);
            let mx = zs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            mx - mn
        };

        let params = default_params(BrushType::Flatten);
        brush_flatten(&mut mesh, [0.5, 0.5, 0.0], &params);

        let z_spread_after = {
            let zs: Vec<f32> = mesh.positions.iter().map(|p| p[2]).collect();
            let mn = zs.iter().cloned().fold(f32::INFINITY, f32::min);
            let mx = zs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            mx - mn
        };

        assert!(
            z_spread_after < z_spread_before,
            "flatten should reduce Z spread: before={z_spread_before}, after={z_spread_after}"
        );
    }

    // ── Test 12: BrushStroke::undo restores positions ─────────────────────────

    #[test]
    fn brush_stroke_undo_restores_positions() {
        let mut mesh = quad_mesh();
        let params = default_params(BrushType::Grab);
        let center = [0.0f32, 0.0, 0.0];
        let delta = [0.0, 0.0, 0.2];

        let stroke = BrushStroke::record(&mesh, center, delta, params.clone());
        brush_grab(&mut mesh, center, delta, &params);

        // Positions should have changed.
        let changed = mesh
            .positions
            .iter()
            .zip(stroke.before_positions.iter())
            .any(|(a, b)| (a[2] - b[2]).abs() > 1e-6);
        assert!(changed, "grab should have changed positions");

        // Undo should restore.
        stroke.undo(&mut mesh);
        for (a, b) in mesh.positions.iter().zip(stroke.before_positions.iter()) {
            assert!(
                (a[0] - b[0]).abs() < 1e-6
                    && (a[1] - b[1]).abs() < 1e-6
                    && (a[2] - b[2]).abs() < 1e-6,
                "undo did not restore position: {:?} != {:?}",
                a,
                b
            );
        }
    }

    // ── Test 13: apply_brush dispatches Inflate correctly ─────────────────────

    #[test]
    fn apply_brush_inflate_via_dispatcher() {
        let mut mesh = quad_mesh();
        let params = BrushParams {
            brush_type: BrushType::Inflate,
            ..default_params(BrushType::Inflate)
        };
        let result = apply_brush(&mut mesh, [0.0, 0.0, 0.0], [0.0; 3], &params);
        assert!(
            result.affected_count > 0,
            "dispatcher inflate must affect verts"
        );
    }

    // ── Test 14: apply_brush dispatches Smooth correctly ──────────────────────

    #[test]
    fn apply_brush_smooth_via_dispatcher() {
        let mut mesh = tri_mesh();
        let params = BrushParams {
            brush_type: BrushType::Smooth,
            ..default_params(BrushType::Smooth)
        };
        let result = apply_brush(&mut mesh, [0.3, 0.3, 0.0], [0.0; 3], &params);
        assert!(
            result.affected_count > 0,
            "dispatcher smooth must affect verts"
        );
    }

    // ── Test 15: BrushParams default ─────────────────────────────────────────

    #[test]
    fn brush_params_default_values() {
        let p = BrushParams::default();
        assert_eq!(p.brush_type, BrushType::Grab);
        assert!((p.radius - 0.1).abs() < 1e-6);
        assert!((p.strength - 0.5).abs() < 1e-6);
        assert!((p.falloff - 1.0).abs() < 1e-6);
        assert!(!p.invert);
    }

    // ── Test 16: zero-radius brush affects nobody ─────────────────────────────

    #[test]
    fn zero_radius_brush_affects_nobody() {
        let mut mesh = quad_mesh();
        let params = BrushParams {
            radius: 0.0,
            ..default_params(BrushType::Grab)
        };
        let result = brush_grab(&mut mesh, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], &params);
        assert_eq!(
            result.affected_count, 0,
            "zero-radius brush must affect nobody"
        );
        assert_eq!(result.max_displacement, 0.0);
    }
}
