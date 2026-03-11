// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct SphVorticity {
    pub epsilon: f32,
}

#[allow(dead_code)]
pub fn new_sph_vorticity(epsilon: f32) -> SphVorticity {
    SphVorticity { epsilon }
}

#[allow(dead_code)]
pub fn sv2_compute_vorticity(vel_curl: [f32; 3]) -> f32 {
    (vel_curl[0].powi(2) + vel_curl[1].powi(2) + vel_curl[2].powi(2)).sqrt()
}

#[allow(dead_code)]
pub fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
pub fn sv2_confinement_force(v: &SphVorticity, vorticity: [f32; 3], eta: [f32; 3]) -> [f32; 3] {
    let c = cross3(eta, vorticity);
    [v.epsilon * c[0], v.epsilon * c[1], v.epsilon * c[2]]
}

#[allow(dead_code)]
pub fn sv2_epsilon(v: &SphVorticity) -> f32 {
    v.epsilon
}

#[allow(dead_code)]
pub fn sv2_set_epsilon(v: &mut SphVorticity, e: f32) {
    v.epsilon = e;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vorticity_of_zero() {
        let mag = sv2_compute_vorticity([0.0; 3]);
        assert!((mag).abs() < 1e-7);
    }

    #[test]
    fn test_vorticity_nonzero() {
        let mag = sv2_compute_vorticity([1.0, 0.0, 0.0]);
        assert!((mag - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cross3() {
        let c = cross3([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_confinement_force() {
        let v = new_sph_vorticity(1.0);
        let f = sv2_confinement_force(&v, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert!(f.iter().any(|x| x.abs() > 1e-6));
    }

    #[test]
    fn test_epsilon_getter() {
        let v = new_sph_vorticity(0.5);
        assert!((sv2_epsilon(&v) - 0.5).abs() < 1e-7);
    }

    #[test]
    fn test_set_epsilon() {
        let mut v = new_sph_vorticity(0.1);
        sv2_set_epsilon(&mut v, 2.0);
        assert!((sv2_epsilon(&v) - 2.0).abs() < 1e-7);
    }

    #[test]
    fn test_confinement_force_zero_epsilon() {
        let v = new_sph_vorticity(0.0);
        let f = sv2_confinement_force(&v, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((f[0]).abs() < 1e-7 && (f[1]).abs() < 1e-7 && (f[2]).abs() < 1e-7);
    }

    #[test]
    fn test_cross3_anticommutative() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [4.0f32, 5.0, 6.0];
        let c1 = cross3(a, b);
        let c2 = cross3(b, a);
        assert!((c1[0] + c2[0]).abs() < 1e-6);
    }
}
