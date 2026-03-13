// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face inset: shrink faces inward with a border ring.

/* ── legacy structs kept for any future lib.rs ── */

#[derive(Debug, Clone)]
pub struct InsetConfig {
    pub thickness: f32,
    pub depth: f32,
    pub individual: bool,
}

#[derive(Debug, Clone)]
pub struct InsetResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub inset_face_count: usize,
}

pub fn default_inset_config() -> InsetConfig {
    InsetConfig {
        thickness: 0.1,
        depth: 0.0,
        individual: false,
    }
}

pub fn inset_validate_config(config: &InsetConfig) -> bool {
    config.thickness >= 0.0
}

pub fn inset_face_center(positions: &[[f32; 3]], face: &[u32]) -> [f32; 3] {
    let n = face.len().max(1) as f32;
    let (mut cx, mut cy, mut cz) = (0.0f32, 0.0f32, 0.0f32);
    for &vi in face {
        let p = positions[vi as usize];
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    [cx / n, cy / n, cz / n]
}

pub fn inset_result_face_count(face_selection: &[usize]) -> usize {
    face_selection.len()
}

pub fn inset_vertex_count(positions: &[[f32; 3]], face_selection: &[usize]) -> usize {
    positions.len() + face_selection.len() * 3
}

pub fn inset_faces(
    positions: &[[f32; 3]],
    indices: &[u32],
    face_selection: &[usize],
    config: &InsetConfig,
) -> InsetResult {
    let mut out_pos = positions.to_vec();
    let mut out_idx = indices.to_vec();
    let t = config.thickness.clamp(0.0, 1.0);
    for &fi in face_selection {
        let base = fi * 3;
        if base + 2 >= indices.len() {
            continue;
        }
        let a = indices[base] as usize;
        let b = indices[base + 1] as usize;
        let c = indices[base + 2] as usize;
        let face_verts = [a as u32, b as u32, c as u32];
        let center = inset_face_center(positions, &face_verts);
        let new_base = out_pos.len() as u32;
        for &vi in &[a, b, c] {
            let p = positions[vi];
            out_pos.push([
                p[0] + (center[0] - p[0]) * t,
                p[1] + (center[1] - p[1]) * t,
                p[2] + (center[2] - p[2]) * t - config.depth,
            ]);
        }
        out_idx.extend_from_slice(&[new_base, new_base + 1, new_base + 2]);
    }
    let inset_face_count = face_selection.len();
    InsetResult {
        positions: out_pos,
        indices: out_idx,
        inset_face_count,
    }
}

pub fn inset_to_json(result: &InsetResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"inset_face_count\":{}}}",
        result.positions.len(),
        result.inset_face_count
    )
}

/* ── spec functions (wave 150B) ── */

/// Inset a single triangular face by `amount`. Returns new 3 inset positions.
pub fn inset_face(positions: &[[f32; 3]], face: &[usize], amount: f32) -> Vec<[f32; 3]> {
    let n = face.len();
    if n == 0 {
        return vec![];
    }
    let cx: f32 = face.iter().map(|&i| positions[i][0]).sum::<f32>() / n as f32;
    let cy: f32 = face.iter().map(|&i| positions[i][1]).sum::<f32>() / n as f32;
    let cz: f32 = face.iter().map(|&i| positions[i][2]).sum::<f32>() / n as f32;
    face.iter()
        .map(|&i| {
            let p = positions[i];
            [
                p[0] + (cx - p[0]) * amount,
                p[1] + (cy - p[1]) * amount,
                p[2] + (cz - p[2]) * amount,
            ]
        })
        .collect()
}

/// Centroid of a face given vertex indices.
pub fn inset_centroid(positions: &[[f32; 3]], face: &[usize]) -> [f32; 3] {
    let n = face.len();
    if n == 0 {
        return [0.0; 3];
    }
    let cx: f32 = face.iter().map(|&i| positions[i][0]).sum::<f32>() / n as f32;
    let cy: f32 = face.iter().map(|&i| positions[i][1]).sum::<f32>() / n as f32;
    let cz: f32 = face.iter().map(|&i| positions[i][2]).sum::<f32>() / n as f32;
    [cx, cy, cz]
}

/// Thickness of an inset (distance from original to inset vertex).
pub fn inset_face_thickness(positions: &[[f32; 3]], face: &[usize], amount: f32) -> f32 {
    if face.is_empty() {
        return 0.0;
    }
    let inset = inset_face(positions, face, amount);
    let p = positions[face[0]];
    let q = inset[0];
    let dx = q[0] - p[0];
    let dy = q[1] - p[1];
    let dz = q[2] - p[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Inset each face individually.
pub fn inset_individual_faces(
    positions: &[[f32; 3]],
    faces: &[Vec<usize>],
    amount: f32,
) -> Vec<Vec<[f32; 3]>> {
    faces
        .iter()
        .map(|face| inset_face(positions, face, amount))
        .collect()
}

/// Returns true if an inset face has more than 2 vertices (trivially valid).
pub fn inset_creates_valid_face(face: &[usize]) -> bool {
    face.len() >= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]]
    }

    #[test]
    fn test_inset_face_moves_toward_center() {
        /* amount=1.0 should land exactly on centroid */
        let pos = tri_positions();
        let face = vec![0, 1, 2];
        let inset = inset_face(&pos, &face, 1.0);
        let c = inset_centroid(&pos, &face);
        for p in &inset {
            assert!((p[0] - c[0]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_inset_centroid() {
        let pos = tri_positions();
        let face = vec![0, 1, 2];
        let c = inset_centroid(&pos, &face);
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_inset_face_thickness_zero() {
        /* amount=0 → thickness=0 */
        let pos = tri_positions();
        let t = inset_face_thickness(&pos, &[0, 1, 2], 0.0);
        assert!(t < 1e-5);
    }

    #[test]
    fn test_inset_individual_faces() {
        let pos = tri_positions();
        let faces = vec![vec![0, 1, 2]];
        let result = inset_individual_faces(&pos, &faces, 0.5);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
    }

    #[test]
    fn test_inset_creates_valid_face_true() {
        assert!(inset_creates_valid_face(&[0, 1, 2]));
    }

    #[test]
    fn test_inset_creates_valid_face_false() {
        assert!(!inset_creates_valid_face(&[0, 1]));
    }
}
