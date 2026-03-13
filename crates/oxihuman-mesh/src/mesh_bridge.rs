// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Bridge / loft operations: connect two open edge loops with a band of triangles.

use crate::connectivity::find_boundary_loops;
use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    add3(scale3(a, 1.0 - t), scale3(b, t))
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = length3(v);
    if len > 1e-12 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    length3(sub3(a, b))
}

/// Smooth-step (Hermite) interpolation: 3t^2 - 2t^3
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Cubic Bezier interpolation between p0 and p3 with auto tangents.
fn bezier_interp(p0: [f32; 3], p3: [f32; 3], t: f32) -> [f32; 3] {
    // Auto-compute control points: 1/3 and 2/3 of the way
    let p1 = lerp3(p0, p3, 1.0 / 3.0);
    let p2 = lerp3(p0, p3, 2.0 / 3.0);
    let mt = 1.0 - t;
    // B(t) = (1-t)^3 p0 + 3(1-t)^2 t p1 + 3(1-t)t^2 p2 + t^3 p3
    let c0 = mt * mt * mt;
    let c1 = 3.0 * mt * mt * t;
    let c2 = 3.0 * mt * t * t;
    let c3 = t * t * t;
    [
        c0 * p0[0] + c1 * p1[0] + c2 * p2[0] + c3 * p3[0],
        c0 * p0[1] + c1 * p1[1] + c2 * p2[1] + c3 * p3[1],
        c0 * p0[2] + c1 * p1[2] + c2 * p2[2] + c3 * p3[2],
    ]
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// An ordered loop of vertex indices representing one open boundary.
#[derive(Debug, Clone)]
pub struct EdgeLoop {
    /// Ordered vertex indices into a MeshBuffers.
    pub vertices: Vec<u32>,
    /// true if loop forms a closed ring.
    pub closed: bool,
}

/// Interpolation mode for the bridge cross-sections.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BridgeInterpolation {
    /// Straight linear interpolation.
    Linear,
    /// Smooth Hermite (smoothstep) interpolation.
    Smooth,
    /// Cubic Bezier with auto-computed tangents.
    Bezier,
}

/// Configuration for bridge_loops.
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Number of cross-sections between the two loops (default 1).
    pub segments: u32,
    /// How to interpolate intermediate positions.
    pub interpolation: BridgeInterpolation,
    /// Twist angle in radians applied to loop B before connecting.
    pub twist: f32,
    /// Reverse winding of loop B faces.
    pub flip_loop_b: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        BridgeConfig {
            segments: 1,
            interpolation: BridgeInterpolation::Linear,
            twist: 0.0,
            flip_loop_b: false,
        }
    }
}

/// Result of a bridge operation.
#[derive(Debug, Clone)]
pub struct BridgeResult {
    /// The bridge band mesh (just the new faces).
    pub mesh: MeshBuffers,
    pub vertex_count: usize,
    pub face_count: usize,
}

// ---------------------------------------------------------------------------
// loop_centroid
// ---------------------------------------------------------------------------

/// Compute the average (centroid) position of all vertices in a loop.
pub fn loop_centroid(loop_: &EdgeLoop, mesh: &MeshBuffers) -> [f32; 3] {
    if loop_.vertices.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut sum = [0.0f32; 3];
    let mut count = 0usize;
    for &vi in &loop_.vertices {
        if (vi as usize) < mesh.positions.len() {
            let p = mesh.positions[vi as usize];
            sum[0] += p[0];
            sum[1] += p[1];
            sum[2] += p[2];
            count += 1;
        }
    }
    if count == 0 {
        return [0.0, 0.0, 0.0];
    }
    let n = count as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

// ---------------------------------------------------------------------------
// align_loops
// ---------------------------------------------------------------------------

/// Reorder loop_b so that its starting vertex minimises total edge length to loop_a.
///
/// Returns new EdgeLoop values for both loops (loop_a unchanged, loop_b rotated).
pub fn align_loops(
    loop_a: &EdgeLoop,
    loop_b: &EdgeLoop,
    base: &MeshBuffers,
) -> (EdgeLoop, EdgeLoop) {
    if loop_a.vertices.is_empty() || loop_b.vertices.is_empty() {
        return (loop_a.clone(), loop_b.clone());
    }

    let n = loop_b.vertices.len();
    let a_first_pos = base
        .positions
        .get(loop_a.vertices[0] as usize)
        .copied()
        .unwrap_or([0.0; 3]);

    // Find rotation offset in loop_b that puts nearest vertex to loop_a[0] first.
    let mut best_offset = 0usize;
    let mut best_dist = f32::MAX;
    for offset in 0..n {
        let b_vi = loop_b.vertices[offset] as usize;
        if b_vi >= base.positions.len() {
            continue;
        }
        let d = dist3(a_first_pos, base.positions[b_vi]);
        if d < best_dist {
            best_dist = d;
            best_offset = offset;
        }
    }

    // Rotate loop_b by best_offset.
    let mut new_b_verts = Vec::with_capacity(n);
    for i in 0..n {
        new_b_verts.push(loop_b.vertices[(best_offset + i) % n]);
    }

    let new_loop_b = EdgeLoop {
        vertices: new_b_verts,
        closed: loop_b.closed,
    };

    (loop_a.clone(), new_loop_b)
}

// ---------------------------------------------------------------------------
// loop_from_boundary
// ---------------------------------------------------------------------------

/// Extract boundary loops from a mesh by wrapping connectivity::find_boundary_loops.
pub fn loop_from_boundary(mesh: &MeshBuffers) -> Vec<EdgeLoop> {
    let raw_loops = find_boundary_loops(mesh);
    raw_loops
        .into_iter()
        .map(|verts| EdgeLoop {
            closed: true,
            vertices: verts,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// open_cylinder (test helper)
// ---------------------------------------------------------------------------

/// Build an open-ended cylinder and return its mesh plus top and bottom edge loops.
///
/// The cylinder has no caps, so its top and bottom boundaries are open loops.
pub fn open_cylinder(radius: f32, height: f32, segments: u32) -> (MeshBuffers, EdgeLoop, EdgeLoop) {
    let n = segments.max(3) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n * 2);
    let mut indices: Vec<u32> = Vec::new();

    // Bottom ring: y = 0, Top ring: y = height
    for i in 0..n {
        let angle = std::f32::consts::TAU * (i as f32) / (n as f32);
        let (s, c) = angle.sin_cos();
        let nx = c;
        let nz = s;
        positions.push([radius * c, 0.0, radius * s]);
        normals.push([nx, 0.0, nz]);
        uvs.push([(i as f32) / (n as f32), 0.0]);
    }
    for i in 0..n {
        let angle = std::f32::consts::TAU * (i as f32) / (n as f32);
        let (s, c) = angle.sin_cos();
        let nx = c;
        let nz = s;
        positions.push([radius * c, height, radius * s]);
        normals.push([nx, 0.0, nz]);
        uvs.push([(i as f32) / (n as f32), 1.0]);
    }

    // Build quads between rings (2 tris each)
    for i in 0..n {
        let i0 = i as u32;
        let i1 = ((i + 1) % n) as u32;
        let i2 = (n + (i + 1) % n) as u32;
        let i3 = (n + i) as u32;
        // tri 1: i0, i1, i3
        indices.push(i0);
        indices.push(i1);
        indices.push(i3);
        // tri 2: i1, i2, i3
        indices.push(i1);
        indices.push(i2);
        indices.push(i3);
    }

    let tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; positions.len()];
    let mesh = MeshBuffers {
        positions,
        normals,
        tangents,
        uvs,
        indices,
        colors: None,
        has_suit: false,
    };

    let bottom_loop = EdgeLoop {
        vertices: (0..n as u32).collect(),
        closed: true,
    };
    let top_loop = EdgeLoop {
        vertices: (n as u32..(2 * n) as u32).collect(),
        closed: true,
    };

    (mesh, bottom_loop, top_loop)
}

// ---------------------------------------------------------------------------
// bridge_loops
// ---------------------------------------------------------------------------

/// Bridge two edge loops with a band of triangles.
///
/// # Errors
/// Returns an error if either loop has fewer than 2 vertices, or if the loops
/// have different vertex counts.
#[allow(clippy::too_many_arguments)]
pub fn bridge_loops(
    base: &MeshBuffers,
    loop_a: &EdgeLoop,
    loop_b: &EdgeLoop,
    config: &BridgeConfig,
) -> anyhow::Result<BridgeResult> {
    // Validate
    if loop_a.vertices.len() < 2 {
        anyhow::bail!(
            "loop_a must have at least 2 vertices, got {}",
            loop_a.vertices.len()
        );
    }
    if loop_b.vertices.len() < 2 {
        anyhow::bail!(
            "loop_b must have at least 2 vertices, got {}",
            loop_b.vertices.len()
        );
    }
    if loop_a.vertices.len() != loop_b.vertices.len() {
        anyhow::bail!(
            "loop_a and loop_b must have the same vertex count: {} != {}",
            loop_a.vertices.len(),
            loop_b.vertices.len()
        );
    }

    let n = loop_a.vertices.len();
    let segs = config.segments.max(1) as usize;

    // Gather positions from base mesh for loop_a and loop_b
    let mut pos_a: Vec<[f32; 3]> = Vec::with_capacity(n);
    for &vi in &loop_a.vertices {
        let vi = vi as usize;
        if vi >= base.positions.len() {
            anyhow::bail!(
                "loop_a vertex index {} out of range (mesh has {} verts)",
                vi,
                base.positions.len()
            );
        }
        pos_a.push(base.positions[vi]);
    }

    let mut pos_b: Vec<[f32; 3]> = Vec::with_capacity(n);
    for &vi in &loop_b.vertices {
        let vi = vi as usize;
        if vi >= base.positions.len() {
            anyhow::bail!(
                "loop_b vertex index {} out of range (mesh has {} verts)",
                vi,
                base.positions.len()
            );
        }
        pos_b.push(base.positions[vi]);
    }

    // Apply twist to loop_b: rotate each vertex around the centroid by `twist` radians.
    if config.twist.abs() > 1e-9 {
        // Compute centroid of loop_b positions.
        let centroid_b = {
            let mut s = [0.0f32; 3];
            for &p in &pos_b {
                s[0] += p[0];
                s[1] += p[1];
                s[2] += p[2];
            }
            let inv = 1.0 / n as f32;
            [s[0] * inv, s[1] * inv, s[2] * inv]
        };

        // Compute the average up-axis of loop_b relative to loop_a (centroid direction)
        let centroid_a = {
            let mut s = [0.0f32; 3];
            for &p in &pos_a {
                s[0] += p[0];
                s[1] += p[1];
                s[2] += p[2];
            }
            let inv = 1.0 / n as f32;
            [s[0] * inv, s[1] * inv, s[2] * inv]
        };
        let axis = normalize3(sub3(centroid_b, centroid_a));

        let angle = config.twist;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        for p in &mut pos_b {
            // Rodrigues' rotation around `axis`
            let v = sub3(*p, centroid_b);
            let v_rot = add3(
                add3(scale3(v, cos_a), scale3(cross3(axis, v), sin_a)),
                scale3(axis, dot3(axis, v) * (1.0 - cos_a)),
            );
            *p = add3(centroid_b, v_rot);
        }
    }

    // If flip_loop_b, reverse pos_b order (excluding first element to maintain shared edge direction)
    let pos_b = if config.flip_loop_b {
        let mut flipped = pos_b.clone();
        flipped.reverse();
        flipped
    } else {
        pos_b
    };

    // Build rows of vertices: (segs+1) rows, each with n vertices.
    // Row 0 = loop_a positions, Row segs = loop_b positions.
    let num_rows = segs + 1;
    let mut all_positions: Vec<[f32; 3]> = Vec::with_capacity(num_rows * n);

    for row in 0..num_rows {
        let t_raw = row as f32 / segs as f32;
        let t = match config.interpolation {
            BridgeInterpolation::Linear => t_raw,
            BridgeInterpolation::Smooth => smoothstep(t_raw),
            BridgeInterpolation::Bezier => t_raw, // bezier per-vertex below
        };

        for vi in 0..n {
            let p = match config.interpolation {
                BridgeInterpolation::Bezier => bezier_interp(pos_a[vi], pos_b[vi], t_raw),
                _ => lerp3(pos_a[vi], pos_b[vi], t),
            };
            all_positions.push(p);
        }
    }

    let total_verts = num_rows * n;

    // Build UVs
    let mut all_uvs: Vec<[f32; 2]> = Vec::with_capacity(total_verts);
    for row in 0..num_rows {
        let v_coord = row as f32 / segs as f32;
        for vi in 0..n {
            let u_coord = vi as f32 / n as f32;
            all_uvs.push([u_coord, v_coord]);
        }
    }

    // Build triangles: quads between adjacent rows, split into 2 tris
    let mut indices: Vec<u32> = Vec::new();

    for seg in 0..segs {
        for vi in 0..n {
            let next_vi = (vi + 1) % n;
            // Row indices
            let r0 = seg;
            let r1 = seg + 1;
            // Quad corners
            let i00 = (r0 * n + vi) as u32;
            let i01 = (r0 * n + next_vi) as u32;
            let i10 = (r1 * n + vi) as u32;
            let i11 = (r1 * n + next_vi) as u32;
            // Two triangles (CCW winding)
            // Tri 1: i00, i01, i10
            indices.push(i00);
            indices.push(i01);
            indices.push(i10);
            // Tri 2: i01, i11, i10
            indices.push(i01);
            indices.push(i11);
            indices.push(i10);
        }
    }

    let face_count = indices.len() / 3;

    // Compute per-vertex normals by accumulating face normals
    let mut normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; total_verts];
    for tri in indices.chunks_exact(3) {
        let (ia, ib, ic) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = all_positions[ia];
        let pb = all_positions[ib];
        let pc = all_positions[ic];
        let n_vec = cross3(sub3(pb, pa), sub3(pc, pa));
        normals[ia] = add3(normals[ia], n_vec);
        normals[ib] = add3(normals[ib], n_vec);
        normals[ic] = add3(normals[ic], n_vec);
    }
    for n_vec in &mut normals {
        *n_vec = normalize3(*n_vec);
    }

    let tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; total_verts];

    let mesh = MeshBuffers {
        positions: all_positions,
        normals,
        tangents,
        uvs: all_uvs,
        indices,
        colors: None,
        has_suit: false,
    };

    Ok(BridgeResult {
        vertex_count: total_verts,
        face_count,
        mesh,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a simple MeshBuffers from positions only (for testing).
    fn mesh_from_positions(positions: Vec<[f32; 3]>) -> MeshBuffers {
        let n = positions.len();
        MeshBuffers {
            positions,
            normals: vec![[0.0, 1.0, 0.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices: vec![],
            colors: None,
            has_suit: false,
        }
    }

    /// Make a square loop at y=0: 4 vertices.
    fn square_loop_bottom(mesh: &mut MeshBuffers) -> EdgeLoop {
        let base = mesh.positions.len() as u32;
        mesh.positions.push([-1.0, 0.0, -1.0]);
        mesh.positions.push([1.0, 0.0, -1.0]);
        mesh.positions.push([1.0, 0.0, 1.0]);
        mesh.positions.push([-1.0, 0.0, 1.0]);
        mesh.normals.extend_from_slice(&[[0.0, -1.0, 0.0]; 4]);
        mesh.tangents.extend_from_slice(&[[1.0, 0.0, 0.0, 1.0]; 4]);
        mesh.uvs.extend_from_slice(&[[0.0, 0.0]; 4]);
        EdgeLoop {
            vertices: (base..base + 4).collect(),
            closed: true,
        }
    }

    /// Make a square loop at y=2: 4 vertices.
    fn square_loop_top(mesh: &mut MeshBuffers) -> EdgeLoop {
        let base = mesh.positions.len() as u32;
        mesh.positions.push([-1.0, 2.0, -1.0]);
        mesh.positions.push([1.0, 2.0, -1.0]);
        mesh.positions.push([1.0, 2.0, 1.0]);
        mesh.positions.push([-1.0, 2.0, 1.0]);
        mesh.normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
        mesh.tangents.extend_from_slice(&[[1.0, 0.0, 0.0, 1.0]; 4]);
        mesh.uvs.extend_from_slice(&[[0.0, 0.0]; 4]);
        EdgeLoop {
            vertices: (base..base + 4).collect(),
            closed: true,
        }
    }

    // -----------------------------------------------------------------------
    // Test 1: bridge_loops basic - 4-vertex loops, segments=1 → 8 tris (4 quads)
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_basic_quad_count() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        let lt = square_loop_top(&mut mesh);
        let cfg = BridgeConfig::default();
        let result = bridge_loops(&mesh, &la, &lt, &cfg).expect("should succeed");
        // 4 quads → 8 triangles
        assert_eq!(result.face_count, 8);
    }

    // -----------------------------------------------------------------------
    // Test 2: bridge_loops vertex count with segments=1
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_vertex_count_segments1() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        let lt = square_loop_top(&mut mesh);
        let cfg = BridgeConfig::default();
        let result = bridge_loops(&mesh, &la, &lt, &cfg).expect("should succeed");
        // 2 rows × 4 verts = 8 verts
        assert_eq!(result.vertex_count, 8);
    }

    // -----------------------------------------------------------------------
    // Test 3: bridge_loops segments=3 → 4 rows, 16 verts, 24 tris
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_segments_3() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        let lt = square_loop_top(&mut mesh);
        let cfg = BridgeConfig {
            segments: 3,
            ..BridgeConfig::default()
        };
        let result = bridge_loops(&mesh, &la, &lt, &cfg).expect("should succeed");
        // 4 rows × 4 verts = 16 verts
        assert_eq!(result.vertex_count, 16);
        // 3 segs × 4 pairs × 2 tris = 24 tris
        assert_eq!(result.face_count, 24);
    }

    // -----------------------------------------------------------------------
    // Test 4: error on mismatched loop sizes
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_error_mismatched_loops() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        // Add a 3-vert loop
        let base = mesh.positions.len() as u32;
        mesh.positions.push([0.0, 2.0, 0.0]);
        mesh.positions.push([1.0, 2.0, 0.0]);
        mesh.positions.push([0.5, 2.0, 1.0]);
        mesh.normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 3]);
        mesh.tangents.extend_from_slice(&[[1.0, 0.0, 0.0, 1.0]; 3]);
        mesh.uvs.extend_from_slice(&[[0.0, 0.0]; 3]);
        let lb = EdgeLoop {
            vertices: (base..base + 3).collect(),
            closed: true,
        };
        let cfg = BridgeConfig::default();
        assert!(bridge_loops(&mesh, &la, &lb, &cfg).is_err());
    }

    // -----------------------------------------------------------------------
    // Test 5: error on loop with fewer than 2 vertices
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_error_too_few_vertices() {
        let mesh = mesh_from_positions(vec![[0.0, 0.0, 0.0]]);
        let la = EdgeLoop {
            vertices: vec![0],
            closed: false,
        };
        let lb = EdgeLoop {
            vertices: vec![0],
            closed: false,
        };
        let cfg = BridgeConfig::default();
        assert!(bridge_loops(&mesh, &la, &lb, &cfg).is_err());
    }

    // -----------------------------------------------------------------------
    // Test 6: smooth interpolation produces different results than linear
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_smooth_interp_differs_from_linear() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        let lt = square_loop_top(&mut mesh);

        let cfg_lin = BridgeConfig {
            segments: 4,
            interpolation: BridgeInterpolation::Linear,
            ..BridgeConfig::default()
        };
        let cfg_smo = BridgeConfig {
            segments: 4,
            interpolation: BridgeInterpolation::Smooth,
            ..BridgeConfig::default()
        };
        let r_lin = bridge_loops(&mesh, &la, &lt, &cfg_lin).expect("should succeed");
        let r_smo = bridge_loops(&mesh, &la, &lt, &cfg_smo).expect("should succeed");
        // Same topology
        assert_eq!(r_lin.face_count, r_smo.face_count);
        assert_eq!(r_lin.vertex_count, r_smo.vertex_count);
        // Interior row positions should differ
        let mid_lin = r_lin.mesh.positions[4]; // row 1, vert 0
        let mid_smo = r_smo.mesh.positions[4];
        // They differ because smoothstep curves away from linear mid-point
        let diff = (mid_lin[1] - mid_smo[1]).abs();
        assert!(
            diff > 1e-4,
            "smooth and linear should differ at interior rows, diff={}",
            diff
        );
    }

    // -----------------------------------------------------------------------
    // Test 7: bezier interpolation produces same vertex count
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_bezier_interp_same_topology() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        let lt = square_loop_top(&mut mesh);
        let cfg = BridgeConfig {
            segments: 2,
            interpolation: BridgeInterpolation::Bezier,
            ..BridgeConfig::default()
        };
        let result = bridge_loops(&mesh, &la, &lt, &cfg).expect("should succeed");
        assert_eq!(result.vertex_count, 3 * 4); // 3 rows × 4 verts
        assert_eq!(result.face_count, 2 * 4 * 2); // 2 segs × 4 pairs × 2 tris
    }

    // -----------------------------------------------------------------------
    // Test 8: twist parameter changes positions
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_twist_changes_positions() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        let lt = square_loop_top(&mut mesh);

        let cfg_no = BridgeConfig::default();
        let cfg_tw = BridgeConfig {
            twist: std::f32::consts::PI / 4.0, // 45 degrees
            ..BridgeConfig::default()
        };
        let r_no = bridge_loops(&mesh, &la, &lt, &cfg_no).expect("should succeed");
        let r_tw = bridge_loops(&mesh, &la, &lt, &cfg_tw).expect("should succeed");
        // Top row positions (row 1) should differ due to twist
        let p_no = r_no.mesh.positions[4]; // row 1, vert 0
        let p_tw = r_tw.mesh.positions[4];
        let diff = dist3(p_no, p_tw);
        assert!(
            diff > 1e-4,
            "twist should move top-row vertices, diff={}",
            diff
        );
    }

    // -----------------------------------------------------------------------
    // Test 9: flip_loop_b reverses connectivity
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_flip_loop_b() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        let lt = square_loop_top(&mut mesh);

        let cfg_no = BridgeConfig::default();
        let cfg_fl = BridgeConfig {
            flip_loop_b: true,
            ..BridgeConfig::default()
        };
        let r_no = bridge_loops(&mesh, &la, &lt, &cfg_no).expect("should succeed");
        let r_fl = bridge_loops(&mesh, &la, &lt, &cfg_fl).expect("should succeed");
        // Same topology counts
        assert_eq!(r_no.face_count, r_fl.face_count);
        // But top row positions differ (reversed order changes which verts land where)
        let top_no: Vec<_> = (4..8).map(|i| r_no.mesh.positions[i]).collect();
        let top_fl: Vec<_> = (4..8).map(|i| r_fl.mesh.positions[i]).collect();
        assert_ne!(top_no, top_fl);
    }

    // -----------------------------------------------------------------------
    // Test 10: loop_centroid accuracy
    // -----------------------------------------------------------------------
    #[test]
    fn loop_centroid_basic() {
        let mesh = mesh_from_positions(vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 0.0, 2.0],
            [0.0, 0.0, 2.0],
        ]);
        let loop_ = EdgeLoop {
            vertices: vec![0, 1, 2, 3],
            closed: true,
        };
        let c = loop_centroid(&loop_, &mesh);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 0.0).abs() < 1e-6);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // Test 11: loop_centroid with empty loop
    // -----------------------------------------------------------------------
    #[test]
    fn loop_centroid_empty() {
        let mesh = mesh_from_positions(vec![]);
        let loop_ = EdgeLoop {
            vertices: vec![],
            closed: false,
        };
        let c = loop_centroid(&loop_, &mesh);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    // -----------------------------------------------------------------------
    // Test 12: align_loops selects nearest start
    // -----------------------------------------------------------------------
    #[test]
    fn align_loops_selects_nearest() {
        // loop_a starts at (-1,0,-1), loop_b is rotated 90 degrees
        let mut mesh = mesh_from_positions(vec![
            [-1.0, 0.0, -1.0], // 0: loop_a[0]
            [1.0, 0.0, -1.0],  // 1: loop_a[1]
            [1.0, 0.0, 1.0],   // 2: loop_a[2]
            [-1.0, 0.0, 1.0],  // 3: loop_a[3]
            // loop_b rotated: starts at (1,2,-1)
            [1.0, 2.0, -1.0],  // 4: loop_b rotated[0]
            [1.0, 2.0, 1.0],   // 5: loop_b rotated[1]
            [-1.0, 2.0, 1.0],  // 6: loop_b rotated[2]
            [-1.0, 2.0, -1.0], // 7: loop_b rotated[3] -- nearest to loop_a[0]
        ]);
        mesh.normals.resize(8, [0.0, 1.0, 0.0]);
        mesh.tangents.resize(8, [1.0, 0.0, 0.0, 1.0]);
        mesh.uvs.resize(8, [0.0, 0.0]);

        let la = EdgeLoop {
            vertices: vec![0, 1, 2, 3],
            closed: true,
        };
        let lb = EdgeLoop {
            vertices: vec![4, 5, 6, 7],
            closed: true,
        };
        let (_la2, lb2) = align_loops(&la, &lb, &mesh);
        // The nearest vertex in loop_b to loop_a[0]=(-1,0,-1) is index 7=(-1,2,-1)
        // After alignment, lb2.vertices[0] should be 7
        assert_eq!(lb2.vertices[0], 7u32);
    }

    // -----------------------------------------------------------------------
    // Test 13: loop_from_boundary extracts loops
    // -----------------------------------------------------------------------
    #[test]
    fn loop_from_boundary_open_cylinder() {
        let (mesh, _bottom, _top) = open_cylinder(1.0, 2.0, 8);
        let loops = loop_from_boundary(&mesh);
        // Open cylinder has 2 boundary loops (top and bottom circles)
        assert_eq!(
            loops.len(),
            2,
            "expected 2 boundary loops, got {}",
            loops.len()
        );
        for l in &loops {
            assert_eq!(l.vertices.len(), 8, "each loop should have 8 vertices");
            assert!(l.closed, "boundary loops should be marked closed");
        }
    }

    // -----------------------------------------------------------------------
    // Test 14: open_cylinder mesh has correct structure
    // -----------------------------------------------------------------------
    #[test]
    fn open_cylinder_structure() {
        let (mesh, bottom, top) = open_cylinder(1.0, 3.0, 6);
        assert_eq!(mesh.positions.len(), 12); // 2 rings × 6
        assert_eq!(mesh.indices.len(), 36); // 6 quads × 2 tris × 3 indices
        assert_eq!(bottom.vertices.len(), 6);
        assert_eq!(top.vertices.len(), 6);
    }

    // -----------------------------------------------------------------------
    // Test 15: open_cylinder radius check
    // -----------------------------------------------------------------------
    #[test]
    fn open_cylinder_radius() {
        let radius = 2.5f32;
        let (mesh, _bottom, _top) = open_cylinder(radius, 1.0, 12);
        for p in &mesh.positions {
            let r = (p[0] * p[0] + p[2] * p[2]).sqrt();
            assert!(
                (r - radius).abs() < 1e-5,
                "radius mismatch: {} vs {}",
                r,
                radius
            );
        }
    }

    // -----------------------------------------------------------------------
    // Test 16: bridge result mesh has valid indices
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_valid_indices() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        let lt = square_loop_top(&mut mesh);
        let cfg = BridgeConfig {
            segments: 2,
            ..BridgeConfig::default()
        };
        let result = bridge_loops(&mesh, &la, &lt, &cfg).expect("should succeed");
        let vc = result.vertex_count as u32;
        for &idx in &result.mesh.indices {
            assert!(idx < vc, "index {} out of range (vertex_count={})", idx, vc);
        }
    }

    // -----------------------------------------------------------------------
    // Test 17: 2-vertex degenerate loops (minimum case)
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_two_vertex_loops() {
        let mesh = mesh_from_positions(vec![
            [0.0, 0.0, 0.0], // 0: loop_a[0]
            [1.0, 0.0, 0.0], // 1: loop_a[1]
            [0.0, 2.0, 0.0], // 2: loop_b[0]
            [1.0, 2.0, 0.0], // 3: loop_b[1]
        ]);
        let la = EdgeLoop {
            vertices: vec![0, 1],
            closed: false,
        };
        let lb = EdgeLoop {
            vertices: vec![2, 3],
            closed: false,
        };
        let cfg = BridgeConfig::default();
        let result = bridge_loops(&mesh, &la, &lb, &cfg).expect("should succeed");
        // 2 rows × 2 verts = 4 verts, 1 seg × 2 pairs × 2 tris = 4 tris
        assert_eq!(result.vertex_count, 4);
        assert_eq!(result.face_count, 4);
    }

    // -----------------------------------------------------------------------
    // Test 18: bridge normals are unit length
    // -----------------------------------------------------------------------
    #[test]
    fn bridge_normals_unit_length() {
        let mut mesh = mesh_from_positions(vec![]);
        let la = square_loop_bottom(&mut mesh);
        let lt = square_loop_top(&mut mesh);
        let cfg = BridgeConfig::default();
        let result = bridge_loops(&mesh, &la, &lt, &cfg).expect("should succeed");
        for n_vec in &result.mesh.normals {
            let len = (n_vec[0] * n_vec[0] + n_vec[1] * n_vec[1] + n_vec[2] * n_vec[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-5,
                "normal not unit length: len={}",
                len
            );
        }
    }
}
