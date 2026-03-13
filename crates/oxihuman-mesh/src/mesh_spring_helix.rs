// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Helical spring mesh generator (coil spring).

/// Parameters for a helical spring mesh.
#[derive(Debug, Clone)]
pub struct SpringHelixParams {
    /// Coil radius (distance from axis to wire center).
    pub coil_radius: f32,
    /// Wire cross-section radius.
    pub wire_radius: f32,
    /// Number of complete turns.
    pub turns: f32,
    /// Total height of the spring.
    pub height: f32,
    /// Segments per full turn.
    pub segments_per_turn: usize,
    /// Segments around wire cross-section.
    pub wire_segments: usize,
}

impl Default for SpringHelixParams {
    fn default() -> Self {
        Self {
            coil_radius: 0.15,
            wire_radius: 0.02,
            turns: 6.0,
            height: 0.6,
            segments_per_turn: 24,
            wire_segments: 8,
        }
    }
}

/// Result mesh for a helical spring.
#[derive(Debug, Clone)]
pub struct SpringHelixMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl SpringHelixMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Generate a helical spring mesh.
pub fn build_spring_helix(params: &SpringHelixParams) -> SpringHelixMesh {
    let spine = helix_spine(params);
    let ws = params.wire_segments.max(3);
    let n = spine.len();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    for (si, &center) in spine.iter().enumerate() {
        let fwd = helix_forward(&spine, si);
        let (tu, tv) = frame_from_forward(fwd);
        for j in 0..ws {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / ws as f32;
            let (s, c) = angle.sin_cos();
            let nrm = [
                tu[0] * c + tv[0] * s,
                tu[1] * c + tv[1] * s,
                tu[2] * c + tv[2] * s,
            ];
            positions.push([
                center[0] + nrm[0] * params.wire_radius,
                center[1] + nrm[1] * params.wire_radius,
                center[2] + nrm[2] * params.wire_radius,
            ]);
            normals.push(nrm);
        }
    }
    let mut indices = Vec::new();
    for i in 0..(n as u32 - 1) {
        for j in 0..ws as u32 {
            let a = i * ws as u32 + j;
            let b = i * ws as u32 + (j + 1) % ws as u32;
            let c = (i + 1) * ws as u32 + j;
            let d = (i + 1) * ws as u32 + (j + 1) % ws as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    SpringHelixMesh {
        positions,
        normals,
        indices,
    }
}

/// Generate spine points along the helix.
fn helix_spine(params: &SpringHelixParams) -> Vec<[f32; 3]> {
    let total_seg = (params.turns * params.segments_per_turn as f32).round() as usize;
    let total_seg = total_seg.max(4);
    let pitch = params.height / params.turns;
    (0..=total_seg)
        .map(|i| {
            let t = i as f32 / total_seg as f32;
            let angle = t * params.turns * 2.0 * std::f32::consts::PI;
            [
                params.coil_radius * angle.cos(),
                t * params.turns * pitch,
                params.coil_radius * angle.sin(),
            ]
        })
        .collect()
}

/// Forward direction at spine index.
fn helix_forward(spine: &[[f32; 3]], i: usize) -> [f32; 3] {
    let (a, b) = if i + 1 < spine.len() {
        (spine[i], spine[i + 1])
    } else {
        (spine[i - 1], spine[i])
    };
    normalize3([b[0] - a[0], b[1] - a[1], b[2] - a[2]])
}

/// Compute a stable frame from forward vector.
fn frame_from_forward(fwd: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let up = if fwd[1].abs() < 0.9 {
        [0.0f32, 1.0, 0.0]
    } else {
        [1.0f32, 0.0, 0.0]
    };
    let tu = normalize3(cross3(fwd, up));
    let tv = cross3(fwd, tu);
    (tu, tv)
}

/// Spring coil arc length (approximate).
pub fn spring_arc_length(params: &SpringHelixParams) -> f32 {
    let pitch = params.height / params.turns;
    let circum = 2.0 * std::f32::consts::PI * params.coil_radius;
    params.turns * (circum * circum + pitch * pitch).sqrt()
}

/// Validate spring parameters.
pub fn validate_spring_params(p: &SpringHelixParams) -> bool {
    p.coil_radius > p.wire_radius
        && p.wire_radius > 0.0
        && p.turns > 0.0
        && p.height > 0.0
        && p.segments_per_turn >= 4
        && p.wire_segments >= 3
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [1.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_has_vertices() {
        /* spring should have more than zero vertices */
        let m = build_spring_helix(&SpringHelixParams::default());
        assert!(m.vertex_count() > 0);
    }

    #[test]
    fn spring_has_triangles() {
        let m = build_spring_helix(&SpringHelixParams::default());
        assert!(m.triangle_count() > 0);
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_spring_helix(&SpringHelixParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn normals_match_positions() {
        let m = build_spring_helix(&SpringHelixParams::default());
        assert_eq!(m.normals.len(), m.positions.len());
    }

    #[test]
    fn arc_length_positive() {
        /* arc length must be positive */
        assert!(spring_arc_length(&SpringHelixParams::default()) > 0.0);
    }

    #[test]
    fn validate_ok() {
        assert!(validate_spring_params(&SpringHelixParams::default()));
    }

    #[test]
    fn validate_bad_radii() {
        let p = SpringHelixParams {
            wire_radius: 0.2,
            coil_radius: 0.1,
            ..SpringHelixParams::default()
        };
        assert!(!validate_spring_params(&p));
    }

    #[test]
    fn validate_zero_turns() {
        let p = SpringHelixParams {
            turns: 0.0,
            ..SpringHelixParams::default()
        };
        assert!(!validate_spring_params(&p));
    }

    #[test]
    fn helix_spine_length() {
        /* turns=6 segments_per_turn=24 → 145 spine points (144+1) */
        let p = SpringHelixParams::default();
        let spine = helix_spine(&p);
        assert!(spine.len() > 100);
    }
}
