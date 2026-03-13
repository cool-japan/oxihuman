// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Volume preservation constraint for soft bodies.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumeConfig {
    pub rest_volume: f32,
    pub compliance: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumeConstraint {
    pub vertices: Vec<[f32; 3]>,
    pub lambda: f32,
    pub config: VolumeConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumeResult {
    pub delta_volume: f32,
    pub correction_scale: f32,
    pub satisfied: bool,
}

#[allow(dead_code)]
pub fn default_volume_config() -> VolumeConfig {
    VolumeConfig { rest_volume: 1.0, compliance: 0.001 }
}

#[allow(dead_code)]
pub fn new_volume_constraint(vertices: Vec<[f32; 3]>, config: VolumeConfig) -> VolumeConstraint {
    VolumeConstraint { vertices, lambda: 0.0, config }
}

fn signed_volume(verts: &[[f32; 3]]) -> f32 {
    if verts.len() < 4 {
        return 0.0;
    }
    let a = verts[0];
    let b = verts[1];
    let c = verts[2];
    let d = verts[3];
    // Signed volume of tetrahedron
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ad = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    (cross[0] * ad[0] + cross[1] * ad[1] + cross[2] * ad[2]) / 6.0
}

#[allow(dead_code)]
pub fn volume_compute(vc: &VolumeConstraint) -> f32 {
    signed_volume(&vc.vertices).abs()
}

#[allow(dead_code)]
pub fn volume_solve(vc: &mut VolumeConstraint, dt: f32) -> VolumeResult {
    let current = volume_compute(vc);
    let delta = current - vc.config.rest_volume;
    let compliance = vc.config.compliance / (dt * dt);
    let n = vc.vertices.len() as f32;
    let denom = n + compliance;
    let correction_scale = if denom.abs() > 1e-12 {
        (-delta - compliance * vc.lambda) / denom
    } else {
        0.0
    };
    vc.lambda += correction_scale;
    let satisfied = delta.abs() < 1e-4;
    VolumeResult { delta_volume: delta, correction_scale, satisfied }
}

#[allow(dead_code)]
pub fn volume_error(vc: &VolumeConstraint) -> f32 {
    (volume_compute(vc) - vc.config.rest_volume).abs()
}

#[allow(dead_code)]
pub fn volume_reset(vc: &mut VolumeConstraint) {
    vc.lambda = 0.0;
}

#[allow(dead_code)]
pub fn volume_compliance(vc: &VolumeConstraint) -> f32 {
    vc.config.compliance
}

#[allow(dead_code)]
pub fn volume_is_satisfied(vc: &VolumeConstraint, tol: f32) -> bool {
    volume_error(vc) < tol
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tet() -> VolumeConstraint {
        let verts = vec![
            [0.0, 0.0, 0.0f32],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let cfg = VolumeConfig { rest_volume: 1.0 / 6.0, compliance: 0.001 };
        new_volume_constraint(verts, cfg)
    }

    #[test]
    fn test_new_constraint() {
        let vc = unit_tet();
        assert_eq!(vc.vertices.len(), 4);
    }

    #[test]
    fn test_compute_volume_positive() {
        let vc = unit_tet();
        assert!(volume_compute(&vc) > 0.0);
    }

    #[test]
    fn test_error_near_zero_at_rest() {
        let vc = unit_tet();
        assert!(volume_error(&vc) < 1e-5);
    }

    #[test]
    fn test_is_satisfied_at_rest() {
        let vc = unit_tet();
        assert!(volume_is_satisfied(&vc, 1e-4));
    }

    #[test]
    fn test_reset_lambda() {
        let mut vc = unit_tet();
        vc.lambda = 5.0;
        volume_reset(&mut vc);
        assert!(vc.lambda.abs() < 1e-9);
    }

    #[test]
    fn test_compliance() {
        let vc = unit_tet();
        assert!((volume_compliance(&vc) - 0.001).abs() < 1e-9);
    }

    #[test]
    fn test_solve_returns_result() {
        let mut vc = unit_tet();
        let res = volume_solve(&mut vc, 0.016);
        let _ = res.delta_volume;
        let _ = res.correction_scale;
        let _ = res.satisfied;
    }

    #[test]
    fn test_volume_with_few_verts() {
        let verts = vec![[0.0, 0.0, 0.0f32], [1.0, 0.0, 0.0]];
        let cfg = default_volume_config();
        let vc = new_volume_constraint(verts, cfg);
        assert!((volume_compute(&vc) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_volume_error_when_deformed() {
        let mut vc = unit_tet();
        vc.vertices[3] = [0.0, 0.0, 2.0]; // scale z by 2
        let err = volume_error(&vc);
        assert!(err > 0.0);
    }
}
