// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;
use std::f32::consts::PI;

// ─── helpers ────────────────────────────────────────────────────────────────

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

// ─── UV sphere ──────────────────────────────────────────────────────────────

/// Generate a UV sphere mesh.
/// `stacks`: number of horizontal rings (min 2). `slices`: number of vertical segments (min 3).
pub fn sphere(radius: f32, stacks: usize, slices: usize) -> MeshBuffers {
    let stacks = stacks.max(2);
    let slices = slices.max(3);

    let mut positions = Vec::new();
    let mut uvs = Vec::new();

    for i in 0..=stacks {
        let phi = PI * i as f32 / stacks as f32;
        let sp = phi.sin();
        let cp = phi.cos();
        for j in 0..=slices {
            let theta = 2.0 * PI * j as f32 / slices as f32;
            let x = radius * sp * theta.cos();
            let y = radius * cp;
            let z = radius * sp * theta.sin();
            positions.push([x, y, z]);
            uvs.push([j as f32 / slices as f32, i as f32 / stacks as f32]);
        }
    }

    let row = slices + 1;
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

// ─── box ────────────────────────────────────────────────────────────────────

/// Generate an axis-aligned box (cuboid) mesh.
/// `size`: [width, height, depth] half-extents from center.
#[allow(clippy::type_complexity)]
pub fn box_mesh(size: [f32; 3]) -> MeshBuffers {
    let [hx, hy, hz] = size;

    // Each face: 4 positions, 4 UVs, 2 triangles.
    // Faces: +Z, -Z, -X, +X, +Y, -Y
    #[allow(clippy::type_complexity)]
    #[rustfmt::skip]
    let face_data: &[([f32; 3], [[f32; 3]; 4], [[f32; 2]; 4])] = &[
        // +Z front
        ([0.0, 0.0, 1.0], [[-hx,-hy, hz],[hx,-hy, hz],[hx, hy, hz],[-hx, hy, hz]],
         [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
        // -Z back
        ([0.0, 0.0,-1.0], [[ hx,-hy,-hz],[-hx,-hy,-hz],[-hx, hy,-hz],[ hx, hy,-hz]],
         [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
        // -X left
        ([-1.0,0.0, 0.0], [[-hx,-hy,-hz],[-hx,-hy, hz],[-hx, hy, hz],[-hx, hy,-hz]],
         [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
        // +X right
        ([ 1.0,0.0, 0.0], [[ hx,-hy, hz],[ hx,-hy,-hz],[ hx, hy,-hz],[ hx, hy, hz]],
         [[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]),
        // +Y top
        ([0.0, 1.0, 0.0], [[-hx, hy, hz],[ hx, hy, hz],[ hx, hy,-hz],[-hx, hy,-hz]],
         [[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0]]),
        // -Y bottom
        ([0.0,-1.0, 0.0], [[-hx,-hy,-hz],[ hx,-hy,-hz],[ hx,-hy, hz],[-hx,-hy, hz]],
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
        // Two triangles: (0,1,2) and (0,2,3)
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

// ─── cylinder ───────────────────────────────────────────────────────────────

/// Generate a cylinder mesh (open top and bottom).
/// `radius`: cylinder radius. `height`: total height. `segments`: radial segments (min 3).
pub fn cylinder(radius: f32, height: f32, segments: usize) -> MeshBuffers {
    let segments = segments.max(3);
    let half_h = height * 0.5;

    let mut positions = Vec::new();
    let mut uvs = Vec::new();

    // Two rings: bottom (y = -half_h) and top (y = half_h)
    for ring in 0..=1 {
        let y = if ring == 0 { -half_h } else { half_h };
        for j in 0..=segments {
            let theta = 2.0 * PI * j as f32 / segments as f32;
            let x = radius * theta.cos();
            let z = radius * theta.sin();
            positions.push([x, y, z]);
            uvs.push([j as f32 / segments as f32, ring as f32]);
        }
    }

    let row = segments + 1;
    let mut indices = Vec::new();
    for j in 0..segments {
        let a = j as u32;
        let b = (j + 1) as u32;
        let c = (row + j) as u32;
        let d = (row + j + 1) as u32;
        indices.extend_from_slice(&[a, c, d, a, d, b]);
    }

    make_mesh(positions, uvs, indices)
}

// ─── capsule ─────────────────────────────────────────────────────────────────

/// Generate a capsule mesh (cylinder + hemisphere caps).
/// `radius`: capsule radius. `height`: cylinder portion height (total = height + 2*radius).
/// `segments`: radial segments. `rings`: hemisphere rings per cap (min 2).
pub fn capsule(radius: f32, height: f32, segments: usize, rings: usize) -> MeshBuffers {
    let segments = segments.max(3);
    let rings = rings.max(2);
    let half_h = height * 0.5;
    let row = segments + 1;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // ── top hemisphere (stacks 0..rings, phi: 0 → PI/2) ──────────────────
    for i in 0..=rings {
        let phi = PI * 0.5 * i as f32 / rings as f32; // 0 (pole) → PI/2 (equator)
        let sp = phi.sin();
        let cp = phi.cos();
        for j in 0..=segments {
            let theta = 2.0 * PI * j as f32 / segments as f32;
            let x = radius * sp * theta.cos();
            let y = radius * cp + half_h;
            let z = radius * sp * theta.sin();
            positions.push([x, y, z]);
            uvs.push([
                j as f32 / segments as f32,
                i as f32 / (2 * rings + 1) as f32,
            ]);
        }
    }
    // top hemisphere indices
    for i in 0..rings {
        for j in 0..segments {
            let a = (i * row + j) as u32;
            let b = (i * row + j + 1) as u32;
            let c = ((i + 1) * row + j) as u32;
            let d = ((i + 1) * row + j + 1) as u32;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }

    // ── cylinder body (2 rings, equator at top-hemi bottom and bottom-hemi top) ──
    // Top-hemi equator row already in positions at offset rings*row .. (rings+1)*row
    // We reuse those as the top ring of the cylinder, then add the bottom ring.
    let cyl_top_offset = (rings * row) as u32; // index where top equator starts
    let cyl_bot_offset = positions.len() as u32;

    for j in 0..=segments {
        let theta = 2.0 * PI * j as f32 / segments as f32;
        let x = radius * theta.cos();
        let y = -half_h;
        let z = radius * theta.sin();
        positions.push([x, y, z]);
        uvs.push([
            j as f32 / segments as f32,
            (rings + 1) as f32 / (2 * rings + 1) as f32,
        ]);
    }

    for j in 0..segments {
        let a = cyl_top_offset + j as u32;
        let b = cyl_top_offset + j as u32 + 1;
        let c = cyl_bot_offset + j as u32;
        let d = cyl_bot_offset + j as u32 + 1;
        indices.extend_from_slice(&[a, c, d, a, d, b]);
    }

    // ── bottom hemisphere (stacks rings..0, phi: PI/2 → PI) ──────────────
    let bot_offset = positions.len() as u32;
    // We reuse the cyl bottom ring as the equator of the bottom hemisphere,
    // so only add rings-1 additional latitude rings + the south pole row.
    for i in 1..=rings {
        let phi = PI * 0.5 + PI * 0.5 * i as f32 / rings as f32; // PI/2 → PI
        let sp = phi.sin();
        let cp = phi.cos();
        for j in 0..=segments {
            let theta = 2.0 * PI * j as f32 / segments as f32;
            let x = radius * sp * theta.cos();
            let y = radius * cp - half_h;
            let z = radius * sp * theta.sin();
            positions.push([x, y, z]);
            uvs.push([
                j as f32 / segments as f32,
                (rings + 1 + i) as f32 / (2 * rings + 1) as f32,
            ]);
        }
    }

    // Connect cyl bottom ring → bottom hemisphere rings
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
        for j in 0..segments {
            let a = top_base + j as u32;
            let b = top_base + j as u32 + 1;
            let c = bot_base + j as u32;
            let d = bot_base + j as u32 + 1;
            indices.extend_from_slice(&[a, c, d, a, d, b]);
        }
    }

    make_mesh(positions, uvs, indices)
}

// ─── quad ────────────────────────────────────────────────────────────────────

/// Generate a flat quad (two triangles) in the XZ plane centered at origin.
/// `width`: x extent. `depth`: z extent.
pub fn quad(width: f32, depth: f32) -> MeshBuffers {
    let hx = width * 0.5;
    let hz = depth * 0.5;

    let positions = vec![
        [-hx, 0.0, -hz],
        [hx, 0.0, -hz],
        [hx, 0.0, hz],
        [-hx, 0.0, hz],
    ];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let indices = vec![0, 1, 2, 0, 2, 3];

    make_mesh(positions, uvs, indices)
}

// ─── cone ────────────────────────────────────────────────────────────────────

/// Generate a cone mesh.
/// `radius`: base radius. `height`: cone height. `segments`: base radial segments.
pub fn cone(radius: f32, height: f32, segments: usize) -> MeshBuffers {
    let segments = segments.max(3);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // ── lateral surface ──────────────────────────────────────────────────
    // apex + base ring, one strip per segment (apex duplicated per segment for normals)
    // Simpler: shared apex + base ring, accept slightly blurred normals from compute_normals.
    let apex = [0.0f32, height, 0.0];
    let base_y = 0.0f32;

    // base ring
    let base_start = positions.len() as u32;
    for j in 0..=segments {
        let theta = 2.0 * PI * j as f32 / segments as f32;
        let x = radius * theta.cos();
        let z = radius * theta.sin();
        positions.push([x, base_y, z]);
        uvs.push([j as f32 / segments as f32, 1.0]);
    }

    // apex ring (one apex per segment for UV variation)
    let apex_start = positions.len() as u32;
    for j in 0..=segments {
        positions.push(apex);
        uvs.push([(j as f32 + 0.5) / segments as f32, 0.0]);
    }

    // lateral triangles
    for j in 0..segments {
        let b0 = base_start + j as u32;
        let b1 = base_start + j as u32 + 1;
        let ap = apex_start + j as u32;
        indices.extend_from_slice(&[b0, b1, ap]);
    }

    // ── base disk ────────────────────────────────────────────────────────
    let center_idx = positions.len() as u32;
    positions.push([0.0, base_y, 0.0]);
    uvs.push([0.5, 0.5]);

    for j in 0..segments {
        let b0 = base_start + j as u32;
        let b1 = base_start + j as u32 + 1;
        // Winding: center → b1 → b0 (facing -Y for bottom)
        indices.extend_from_slice(&[center_idx, b1, b0]);
    }

    make_mesh(positions, uvs, indices)
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_vertex_count() {
        let s = sphere(1.0, 8, 8);
        assert_eq!(s.positions.len(), (8 + 1) * (8 + 1));
    }

    #[test]
    fn sphere_index_bounds() {
        let s = sphere(1.0, 8, 16);
        let n = s.positions.len() as u32;
        for &idx in &s.indices {
            assert!(idx < n, "index {} out of bounds (n={})", idx, n);
        }
    }

    #[test]
    fn box_face_count() {
        let b = box_mesh([1.0, 1.0, 1.0]);
        assert_eq!(b.indices.len() / 3, 12);
    }

    #[test]
    fn cylinder_index_bounds() {
        let c = cylinder(1.0, 2.0, 16);
        let n = c.positions.len() as u32;
        for &idx in &c.indices {
            assert!(idx < n, "index {} out of bounds (n={})", idx, n);
        }
    }

    #[test]
    fn capsule_has_more_faces_than_cylinder() {
        let segs = 16;
        let rings = 4;
        let cap = capsule(1.0, 2.0, segs, rings);
        let cyl = cylinder(1.0, 2.0, segs);
        assert!(
            cap.face_count() > cyl.face_count(),
            "capsule faces ({}) should exceed cylinder faces ({})",
            cap.face_count(),
            cyl.face_count()
        );
    }

    #[test]
    fn quad_has_2_faces() {
        let q = quad(1.0, 1.0);
        assert_eq!(q.indices.len() / 3, 2);
    }

    #[test]
    fn cone_index_bounds() {
        let c = cone(1.0, 2.0, 16);
        let n = c.positions.len() as u32;
        for &idx in &c.indices {
            assert!(idx < n, "index {} out of bounds (n={})", idx, n);
        }
    }

    #[test]
    fn sphere_positions_on_surface() {
        let r = 1.5;
        let s = sphere(r, 8, 8);
        for p in &s.positions {
            let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(
                (len - r).abs() < 1e-4,
                "position {:?} has length {}, expected {}",
                p,
                len,
                r
            );
        }
    }
}
