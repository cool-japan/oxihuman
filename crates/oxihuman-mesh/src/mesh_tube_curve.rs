// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tube geometry along an arbitrary 3-D curve (generalized pipe).

/// Parameters for tube generation along a curve.
#[derive(Debug, Clone)]
pub struct TubeCurveParams {
    /// Number of sides around the tube cross-section.
    pub sides: usize,
    /// Radius of the tube.
    pub radius: f32,
    /// Whether to cap the tube ends.
    pub cap_ends: bool,
}

impl Default for TubeCurveParams {
    fn default() -> Self {
        Self {
            sides: 8,
            radius: 0.05,
            cap_ends: true,
        }
    }
}

/// A generated tube mesh.
#[derive(Debug, Clone)]
pub struct TubeMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
}

impl TubeMesh {
    /// Number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Generate a tube mesh along a path of spine points.
pub fn tube_along_curve(spine: &[[f32; 3]], params: &TubeCurveParams) -> TubeMesh {
    if spine.len() < 2 {
        return TubeMesh {
            positions: vec![],
            normals: vec![],
            indices: vec![],
            uvs: vec![],
        };
    }
    let sides = params.sides.max(3);
    let n = spine.len();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for (si, &center) in spine.iter().enumerate() {
        let t_param = si as f32 / (n - 1) as f32;
        let fwd = spine_forward(spine, si);
        let (tang_u, tang_v) = compute_frame(fwd);
        for j in 0..sides {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / sides as f32;
            let (s, c) = angle.sin_cos();
            let n_vec = [
                tang_u[0] * c + tang_v[0] * s,
                tang_u[1] * c + tang_v[1] * s,
                tang_u[2] * c + tang_v[2] * s,
            ];
            positions.push([
                center[0] + n_vec[0] * params.radius,
                center[1] + n_vec[1] * params.radius,
                center[2] + n_vec[2] * params.radius,
            ]);
            normals.push(n_vec);
            uvs.push([j as f32 / sides as f32, t_param]);
        }
    }
    let mut indices = Vec::new();
    for i in 0..(n as u32 - 1) {
        for j in 0..sides as u32 {
            let a = i * sides as u32 + j;
            let b = i * sides as u32 + (j + 1) % sides as u32;
            let c = (i + 1) * sides as u32 + j;
            let d = (i + 1) * sides as u32 + (j + 1) % sides as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    TubeMesh {
        positions,
        normals,
        indices,
        uvs,
    }
}

/// Compute forward direction at spine index `i`.
fn spine_forward(spine: &[[f32; 3]], i: usize) -> [f32; 3] {
    let (a, b) = if i + 1 < spine.len() {
        (spine[i], spine[i + 1])
    } else {
        (spine[i - 1], spine[i])
    };
    normalize3([b[0] - a[0], b[1] - a[1], b[2] - a[2]])
}

/// Compute a local frame (two vectors perpendicular to forward).
fn compute_frame(fwd: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let up = if fwd[1].abs() < 0.9 {
        [0.0f32, 1.0, 0.0]
    } else {
        [1.0f32, 0.0, 0.0]
    };
    let tang_u = normalize3(cross3(fwd, up));
    let tang_v = cross3(fwd, tang_u);
    (tang_u, tang_v)
}

/// Estimate expected vertex count.
pub fn expected_vertex_count(spine_len: usize, sides: usize) -> usize {
    spine_len * sides
}

/// Estimate expected triangle count (without caps).
pub fn expected_triangle_count(spine_len: usize, sides: usize) -> usize {
    (spine_len.saturating_sub(1)) * sides * 2
}

/// Validate tube params.
pub fn validate_tube_params(params: &TubeCurveParams) -> bool {
    params.sides >= 3 && params.radius > 0.0
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

    fn line_spine() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]]
    }

    #[test]
    fn vertex_count_correct() {
        /* 3 spine points, 8 sides → 24 verts */
        let t = tube_along_curve(&line_spine(), &TubeCurveParams::default());
        assert_eq!(t.positions.len(), 24);
    }

    #[test]
    fn triangle_count_correct() {
        /* 2 segments, 8 sides → 32 triangles */
        let t = tube_along_curve(&line_spine(), &TubeCurveParams::default());
        assert_eq!(t.triangle_count(), 32);
    }

    #[test]
    fn short_spine_empty() {
        /* single-point spine → empty mesh */
        let t = tube_along_curve(&[[0.0; 3]], &TubeCurveParams::default());
        assert!(t.positions.is_empty());
    }

    #[test]
    fn normals_count_matches_positions() {
        let t = tube_along_curve(&line_spine(), &TubeCurveParams::default());
        assert_eq!(t.normals.len(), t.positions.len());
    }

    #[test]
    fn uvs_count_matches_positions() {
        let t = tube_along_curve(&line_spine(), &TubeCurveParams::default());
        assert_eq!(t.uvs.len(), t.positions.len());
    }

    #[test]
    fn expected_vertex_count_formula() {
        /* 5 spine × 8 sides = 40 */
        assert_eq!(expected_vertex_count(5, 8), 40);
    }

    #[test]
    fn expected_triangle_count_formula() {
        /* (5-1) × 8 × 2 = 64 */
        assert_eq!(expected_triangle_count(5, 8), 64);
    }

    #[test]
    fn validate_params_ok() {
        assert!(validate_tube_params(&TubeCurveParams::default()));
    }

    #[test]
    fn validate_params_bad_sides() {
        /* less than 3 sides is invalid */
        let p = TubeCurveParams {
            sides: 2,
            radius: 0.05,
            cap_ends: false,
        };
        assert!(!validate_tube_params(&p));
    }

    #[test]
    fn validate_params_zero_radius() {
        let p = TubeCurveParams {
            sides: 8,
            radius: 0.0,
            cap_ends: false,
        };
        assert!(!validate_tube_params(&p));
    }
}
