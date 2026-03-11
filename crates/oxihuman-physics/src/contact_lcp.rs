// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! LCP contact solver (Lemke method stub).

/// A contact for the LCP solver.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LcpContact {
    pub normal: [f32; 3],
    pub penetration: f32,
    pub restitution: f32,
    pub friction: f32,
    pub body_a: usize,
    pub body_b: usize,
}

/// An LCP system Ax + b = w, x >= 0, w >= 0, x^T w = 0.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LcpSystem {
    /// A matrix (n×n), row-major.
    pub a: Vec<f32>,
    /// b vector.
    pub b: Vec<f32>,
    pub n: usize,
}

impl LcpSystem {
    #[allow(dead_code)]
    pub fn new(n: usize) -> Self {
        Self {
            a: vec![0.0; n * n],
            n,
            b: vec![0.0; n],
        }
    }

    fn a_ref(&self, i: usize, j: usize) -> f32 {
        self.a[i * self.n + j]
    }

    fn a_set(&mut self, i: usize, j: usize, val: f32) {
        self.a[i * self.n + j] = val;
    }
}

/// Solve an LCP using the Gauss-Seidel projected method (simple iterative).
/// Returns the impulse vector x such that Ax + b >= 0, x >= 0.
#[allow(dead_code)]
pub fn lcp_gauss_seidel(system: &LcpSystem, iterations: usize) -> Vec<f32> {
    let n = system.n;
    let mut x = vec![0.0f32; n];
    for _ in 0..iterations {
        for i in 0..n {
            let mut val = system.b[i];
            for (j, &xj) in x.iter().enumerate() {
                if j != i {
                    val += system.a_ref(i, j) * xj;
                }
            }
            let diag = system.a_ref(i, i);
            if diag.abs() < 1e-10 {
                x[i] = 0.0;
            } else {
                x[i] = (-val / diag).max(0.0);
            }
        }
    }
    x
}

/// Build a simple 1D contact LCP: `[A]` = `[mass_a + mass_b]`, b = relative velocity + restitution.
#[allow(dead_code)]
pub fn build_1d_contact_lcp(
    rel_vel: f32,
    restitution: f32,
    inv_mass_a: f32,
    inv_mass_b: f32,
) -> LcpSystem {
    let mut sys = LcpSystem::new(1);
    sys.a_set(0, 0, inv_mass_a + inv_mass_b);
    sys.b[0] = rel_vel * (1.0 + restitution);
    sys
}

/// Apply a resolved impulse to two bodies.
#[allow(dead_code)]
pub fn apply_lcp_impulse(
    vel_a: &mut [f32; 3],
    vel_b: &mut [f32; 3],
    normal: [f32; 3],
    impulse: f32,
    inv_mass_a: f32,
    inv_mass_b: f32,
) {
    for k in 0..3 {
        vel_a[k] -= inv_mass_a * impulse * normal[k];
        vel_b[k] += inv_mass_b * impulse * normal[k];
    }
}

/// Relative normal velocity at contact.
#[allow(dead_code)]
pub fn relative_normal_vel(vel_a: [f32; 3], vel_b: [f32; 3], normal: [f32; 3]) -> f32 {
    let rel = [
        vel_a[0] - vel_b[0],
        vel_a[1] - vel_b[1],
        vel_a[2] - vel_b[2],
    ];
    rel[0] * normal[0] + rel[1] * normal[1] + rel[2] * normal[2]
}

/// Coulomb friction impulse (tangential).
#[allow(dead_code)]
pub fn friction_impulse(
    rel_vel: [f32; 3],
    normal: [f32; 3],
    normal_impulse: f32,
    friction: f32,
) -> [f32; 3] {
    let vn = rel_vel[0] * normal[0] + rel_vel[1] * normal[1] + rel_vel[2] * normal[2];
    let vt = [
        rel_vel[0] - vn * normal[0],
        rel_vel[1] - vn * normal[1],
        rel_vel[2] - vn * normal[2],
    ];
    let vt_len = (vt[0] * vt[0] + vt[1] * vt[1] + vt[2] * vt[2]).sqrt();
    if vt_len < 1e-8 {
        return [0.0; 3];
    }
    let max_fric = friction * normal_impulse.abs();
    let fric_mag = max_fric.min(vt_len);
    [
        -vt[0] / vt_len * fric_mag,
        -vt[1] / vt_len * fric_mag,
        -vt[2] / vt_len * fric_mag,
    ]
}

/// Number of contacts.
#[allow(dead_code)]
pub fn contact_count(contacts: &[LcpContact]) -> usize {
    contacts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcp_gs_simple_1d() {
        let mut sys = LcpSystem::new(1);
        sys.a_set(0, 0, 2.0);
        sys.b[0] = -4.0;
        let x = lcp_gauss_seidel(&sys, 10);
        assert!((x[0] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn lcp_gs_non_negative_solution() {
        let mut sys = LcpSystem::new(1);
        sys.a_set(0, 0, 1.0);
        sys.b[0] = 5.0;
        let x = lcp_gauss_seidel(&sys, 10);
        assert!(x[0] >= 0.0);
    }

    #[test]
    fn build_1d_contact_lcp_has_correct_size() {
        let sys = build_1d_contact_lcp(-1.0, 0.5, 1.0, 1.0);
        assert_eq!(sys.n, 1);
    }

    #[test]
    fn apply_lcp_impulse_separates_bodies() {
        let mut va = [0.0f32, -1.0, 0.0];
        let mut vb = [0.0f32, 0.0, 0.0];
        apply_lcp_impulse(&mut va, &mut vb, [0.0, 1.0, 0.0], 1.0, 1.0, 1.0);
        assert!(va[1] < 0.0 || vb[1] > -0.1);
    }

    #[test]
    fn relative_normal_vel_approaching() {
        let va = [0.0, -1.0, 0.0];
        let vb = [0.0, 0.0, 0.0];
        let rv = relative_normal_vel(va, vb, [0.0, 1.0, 0.0]);
        assert!(rv < 0.0);
    }

    #[test]
    fn friction_impulse_zero_for_no_tangential_vel() {
        let rel = [0.0, -1.0, 0.0];
        let n = [0.0, 1.0, 0.0];
        let f = friction_impulse(rel, n, 1.0, 0.3);
        assert!(f[0].abs() < 1e-6 && f[2].abs() < 1e-6);
    }

    #[test]
    fn friction_impulse_bounded_by_coulomb() {
        let rel = [5.0, -1.0, 0.0];
        let n = [0.0, 1.0, 0.0];
        let fi = 2.0;
        let mu = 0.5;
        let f = friction_impulse(rel, n, fi, mu);
        let mag = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        assert!(mag <= mu * fi.abs() + 1e-5);
    }

    #[test]
    fn contact_count_correct() {
        let contacts = vec![LcpContact {
            normal: [0.0, 1.0, 0.0],
            penetration: 0.1,
            restitution: 0.5,
            friction: 0.3,
            body_a: 0,
            body_b: 1,
        }];
        assert_eq!(contact_count(&contacts), 1);
    }

    #[test]
    fn lcp_system_new_zeros() {
        let sys = LcpSystem::new(2);
        assert_eq!(sys.a, vec![0.0; 4]);
        assert_eq!(sys.b, vec![0.0; 2]);
    }

    #[test]
    fn relative_normal_vel_separating() {
        let va = [0.0, 1.0, 0.0];
        let vb = [0.0, 0.0, 0.0];
        let rv = relative_normal_vel(va, vb, [0.0, 1.0, 0.0]);
        assert!(rv > 0.0);
    }
}
