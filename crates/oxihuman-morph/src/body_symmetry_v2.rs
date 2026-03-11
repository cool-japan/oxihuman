// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Enforce bilateral vertex symmetry by axis mirroring (v2).

/// Symmetry axis.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetryAxis {
    X,
    Y,
    Z,
}

/// Body symmetry v2 parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BodySymmetryV2Params {
    /// Symmetry blend weight: 0 = no correction, 1 = fully symmetric.
    pub symmetry_weight: f32,
    /// Axis of symmetry.
    pub axis: SymmetryAxis,
    /// Tolerance for matching mirror vertices.
    pub position_tolerance: f32,
    /// Which side is the master (true = positive side).
    pub positive_side_master: bool,
}

impl Default for BodySymmetryV2Params {
    fn default() -> Self {
        Self {
            symmetry_weight: 1.0,
            axis: SymmetryAxis::X,
            position_tolerance: 0.001,
            positive_side_master: true,
        }
    }
}

/// Create default params.
#[allow(dead_code)]
pub fn default_body_symmetry_v2_params() -> BodySymmetryV2Params {
    BodySymmetryV2Params::default()
}

/// Mirror a 3D position across the given axis.
#[allow(dead_code)]
pub fn mirror_position(pos: [f32; 3], axis: SymmetryAxis) -> [f32; 3] {
    match axis {
        SymmetryAxis::X => [-pos[0], pos[1], pos[2]],
        SymmetryAxis::Y => [pos[0], -pos[1], pos[2]],
        SymmetryAxis::Z => [pos[0], pos[1], -pos[2]],
    }
}

/// Check if two positions are mirror-symmetric within tolerance.
#[allow(dead_code)]
pub fn are_mirror_pair(a: [f32; 3], b: [f32; 3], axis: SymmetryAxis, tol: f32) -> bool {
    let mirrored = mirror_position(a, axis);
    let dx = (mirrored[0] - b[0]).abs();
    let dy = (mirrored[1] - b[1]).abs();
    let dz = (mirrored[2] - b[2]).abs();
    dx < tol && dy < tol && dz < tol
}

/// Compute the symmetrized position given master and slave.
#[allow(dead_code)]
pub fn symmetrize_position(
    master: [f32; 3],
    slave: [f32; 3],
    axis: SymmetryAxis,
    weight: f32,
) -> [f32; 3] {
    let w = weight.clamp(0.0, 1.0);
    let target = mirror_position(master, axis);
    [
        slave[0] + (target[0] - slave[0]) * w,
        slave[1] + (target[1] - slave[1]) * w,
        slave[2] + (target[2] - slave[2]) * w,
    ]
}

/// Apply symmetry correction to a buffer of vertex positions.
///
/// `pairs`: list of (master_idx, slave_idx) index pairs.
#[allow(dead_code)]
pub fn apply_symmetry(
    positions: &mut [[f32; 3]],
    pairs: &[(usize, usize)],
    params: &BodySymmetryV2Params,
) {
    for &(master, slave) in pairs {
        if master < positions.len() && slave < positions.len() {
            let m_pos = positions[master];
            let s_pos = positions[slave];
            let axis = params.axis;
            let w = params.symmetry_weight;
            if params.positive_side_master {
                positions[slave] = symmetrize_position(m_pos, s_pos, axis, w);
            } else {
                positions[master] = symmetrize_position(s_pos, m_pos, axis, w);
            }
        }
    }
}

/// Set symmetry weight.
#[allow(dead_code)]
pub fn set_symmetry_weight(params: &mut BodySymmetryV2Params, value: f32) {
    params.symmetry_weight = value.clamp(0.0, 1.0);
}

/// Reset to default.
#[allow(dead_code)]
pub fn reset_body_symmetry_v2(params: &mut BodySymmetryV2Params) {
    *params = BodySymmetryV2Params::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn body_symmetry_v2_to_json(params: &BodySymmetryV2Params) -> String {
    let axis = match params.axis {
        SymmetryAxis::X => "X",
        SymmetryAxis::Y => "Y",
        SymmetryAxis::Z => "Z",
    };
    format!(
        r#"{{"symmetry_weight":{:.4},"axis":"{}","tolerance":{:.6},"positive_master":{}}}"#,
        params.symmetry_weight, axis, params.position_tolerance, params.positive_side_master
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = BodySymmetryV2Params::default();
        assert!((p.symmetry_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mirror_x() {
        let m = mirror_position([1.0, 2.0, 3.0], SymmetryAxis::X);
        assert!((m[0] + 1.0).abs() < 1e-6);
        assert!((m[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_mirror_y() {
        let m = mirror_position([1.0, 2.0, 3.0], SymmetryAxis::Y);
        assert!((m[1] + 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_mirror_z() {
        let m = mirror_position([1.0, 2.0, 3.0], SymmetryAxis::Z);
        assert!((m[2] + 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_are_mirror_pair_true() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [-1.0f32, 0.0, 0.0];
        assert!(are_mirror_pair(a, b, SymmetryAxis::X, 0.001));
    }

    #[test]
    fn test_are_mirror_pair_false() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [1.0f32, 0.0, 0.0];
        assert!(!are_mirror_pair(a, b, SymmetryAxis::X, 0.001));
    }

    #[test]
    fn test_symmetrize_full() {
        let master = [1.0f32, 0.5, 0.5];
        let slave = [0.0f32, 0.5, 0.5];
        let result = symmetrize_position(master, slave, SymmetryAxis::X, 1.0);
        assert!((result[0] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_symmetry() {
        let mut positions = [[1.0f32, 0.0, 0.0], [0.5f32, 0.0, 0.0]];
        let params = BodySymmetryV2Params::default();
        let pairs = vec![(0usize, 1usize)];
        apply_symmetry(&mut positions, &pairs, &params);
        assert!((positions[1][0] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight_clamp() {
        let mut p = BodySymmetryV2Params::default();
        set_symmetry_weight(&mut p, 5.0);
        assert!((p.symmetry_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = body_symmetry_v2_to_json(&BodySymmetryV2Params::default());
        assert!(j.contains("symmetry_weight"));
        assert!(j.contains("axis"));
    }
}
