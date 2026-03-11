// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Omega skinning deformer stub.

/// Omega skin blending mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OmegaMode {
    Linear,
    DualQuat,
    Blended,
}

/// Omega skin vertex.
#[derive(Debug, Clone)]
pub struct OmegaVertex {
    pub bone_index: usize,
    pub omega_weight: f32,
    pub lbs_weight: f32,
}

/// Omega skinning deformer.
#[derive(Debug, Clone)]
pub struct OmegaSkin {
    pub vertices: Vec<OmegaVertex>,
    pub mode: OmegaMode,
    pub blend_factor: f32,
}

impl OmegaSkin {
    pub fn new(vertex_count: usize) -> Self {
        OmegaSkin {
            vertices: (0..vertex_count)
                .map(|_| OmegaVertex {
                    bone_index: 0,
                    omega_weight: 1.0,
                    lbs_weight: 1.0,
                })
                .collect(),
            mode: OmegaMode::Blended,
            blend_factor: 0.5,
        }
    }
}

/// Create a new Omega skin.
pub fn new_omega_skin(vertex_count: usize) -> OmegaSkin {
    OmegaSkin::new(vertex_count)
}

/// Set the blend factor between LBS and DQS.
pub fn omega_set_blend(skin: &mut OmegaSkin, factor: f32) {
    skin.blend_factor = factor.clamp(0.0, 1.0);
}

/// Set the mode.
pub fn omega_set_mode(skin: &mut OmegaSkin, mode: OmegaMode) {
    skin.mode = mode;
}

/// Return vertex count.
pub fn omega_vertex_count(skin: &OmegaSkin) -> usize {
    skin.vertices.len()
}

/// Compute effective weight for a vertex given the blend factor.
pub fn omega_effective_weight(skin: &OmegaSkin, vertex: usize) -> f32 {
    if vertex >= skin.vertices.len() {
        return 0.0;
    }
    let v = &skin.vertices[vertex];
    match skin.mode {
        OmegaMode::Linear => v.lbs_weight,
        OmegaMode::DualQuat => v.omega_weight,
        OmegaMode::Blended => {
            v.lbs_weight * (1.0 - skin.blend_factor) + v.omega_weight * skin.blend_factor
        }
    }
}

/// Return a JSON-like string.
pub fn omega_to_json(skin: &OmegaSkin) -> String {
    format!(
        r#"{{"mode":"{}","blend":{:.4},"vertices":{}}}"#,
        match skin.mode {
            OmegaMode::Linear => "linear",
            OmegaMode::DualQuat => "dual_quat",
            OmegaMode::Blended => "blended",
        },
        skin.blend_factor,
        skin.vertices.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_omega_skin_vertex_count() {
        let s = new_omega_skin(10);
        assert_eq!(
            omega_vertex_count(&s),
            10, /* vertex count must match */
        );
    }

    #[test]
    fn test_default_mode_blended() {
        let s = new_omega_skin(5);
        assert_eq!(
            s.mode,
            OmegaMode::Blended, /* default mode should be Blended */
        );
    }

    #[test]
    fn test_set_blend_clamps() {
        let mut s = new_omega_skin(3);
        omega_set_blend(&mut s, 2.0);
        assert!((s.blend_factor - 1.0).abs() < 1e-5, /* blend factor clamped to 1 */);
    }

    #[test]
    fn test_set_mode_linear() {
        let mut s = new_omega_skin(3);
        omega_set_mode(&mut s, OmegaMode::Linear);
        assert_eq!(s.mode, OmegaMode::Linear /* mode should be Linear */,);
    }

    #[test]
    fn test_effective_weight_linear_mode() {
        let mut s = new_omega_skin(2);
        s.vertices[0].lbs_weight = 0.8;
        omega_set_mode(&mut s, OmegaMode::Linear);
        let w = omega_effective_weight(&s, 0);
        assert!((w - 0.8).abs() < 1e-5, /* linear mode uses lbs_weight */);
    }

    #[test]
    fn test_effective_weight_dq_mode() {
        let mut s = new_omega_skin(2);
        s.vertices[0].omega_weight = 0.9;
        omega_set_mode(&mut s, OmegaMode::DualQuat);
        let w = omega_effective_weight(&s, 0);
        assert!((w - 0.9).abs() < 1e-5 /* DQ mode uses omega_weight */,);
    }

    #[test]
    fn test_effective_weight_out_of_bounds() {
        let s = new_omega_skin(2);
        let w = omega_effective_weight(&s, 99);
        assert!((w).abs() < 1e-6 /* out-of-bounds vertex returns 0 */,);
    }

    #[test]
    fn test_to_json_contains_mode() {
        let s = new_omega_skin(3);
        let j = omega_to_json(&s);
        assert!(j.contains("mode") /* JSON must contain mode */,);
    }

    #[test]
    fn test_default_blend_factor_half() {
        let s = new_omega_skin(2);
        assert!((s.blend_factor - 0.5).abs() < 1e-5, /* default blend factor is 0.5 */);
    }

    #[test]
    fn test_blended_mode_midpoint() {
        let mut s = new_omega_skin(1);
        s.vertices[0].lbs_weight = 0.0;
        s.vertices[0].omega_weight = 1.0;
        omega_set_blend(&mut s, 0.5);
        let w = omega_effective_weight(&s, 0);
        assert!((w - 0.5).abs() < 1e-5, /* blended mode midpoint should be 0.5 */);
    }
}
