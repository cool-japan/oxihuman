// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! XPBD volume-preservation constraint for soft bodies.

/// A volume constraint over 4 particles (tetrahedron).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumeConstraint {
    pub indices: [usize; 4],
    /// Rest volume.
    pub rest_volume: f32,
    pub compliance: f32,
    /// Accumulated Lagrange multiplier (reset each outer time step).
    pub lambda: f32,
}

#[allow(dead_code)]
impl VolumeConstraint {
    pub fn new(indices: [usize; 4], rest_volume: f32, compliance: f32) -> Self {
        Self {
            indices,
            rest_volume,
            compliance,
            lambda: 0.0,
        }
    }
}

/// Compute signed tet volume given 4 positions.
pub fn tet_signed_volume(p: &[[f32; 3]; 4]) -> f32 {
    let v0 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
    let v1 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
    let v2 = [p[3][0] - p[0][0], p[3][1] - p[0][1], p[3][2] - p[0][2]];
    // Triple product / 6.
    let cross = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];
    (v0[0] * cross[0] + v0[1] * cross[1] + v0[2] * cross[2]) / 6.0
}

// ---------------------------------------------------------------------------
// Vector math helpers (f32, 3-component, array-based).
// ---------------------------------------------------------------------------

#[inline(always)]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline(always)]
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline(always)]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline(always)]
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline(always)]
fn len3_sq(a: [f32; 3]) -> f32 {
    dot3(a, a)
}

/// XPBD volume system.
#[allow(dead_code)]
pub struct XpbdVolume {
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub inv_masses: Vec<f32>,
    pub constraints: Vec<VolumeConstraint>,
    pub gravity: [f32; 3],
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl XpbdVolume {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            inv_masses: Vec::new(),
            constraints: Vec::new(),
            gravity: [0.0, -9.81, 0.0],
            time: 0.0,
            steps: 0,
        }
    }

    pub fn add_particle(&mut self, pos: [f32; 3], mass: f32) -> usize {
        let id = self.positions.len();
        self.positions.push(pos);
        self.velocities.push([0.0; 3]);
        self.inv_masses
            .push(if mass > 0.0 { 1.0 / mass } else { 0.0 });
        id
    }

    pub fn add_volume_constraint(&mut self, indices: [usize; 4], compliance: f32) {
        let pts: [[f32; 3]; 4] = [
            self.positions[indices[0]],
            self.positions[indices[1]],
            self.positions[indices[2]],
            self.positions[indices[3]],
        ];
        let rv = tet_signed_volume(&pts).abs();
        self.constraints
            .push(VolumeConstraint::new(indices, rv, compliance));
    }

    pub fn step(&mut self, dt: f32, sub_steps: u32) {
        // Reset accumulated Lagrange multipliers once per outer time step (XPBD standard).
        for c in &mut self.constraints {
            c.lambda = 0.0;
        }
        let sub_dt = dt / sub_steps as f32;
        for _ in 0..sub_steps {
            self.substep(sub_dt);
        }
        self.time += dt;
        self.steps += 1;
    }

    #[allow(clippy::needless_range_loop)]
    fn substep(&mut self, dt: f32) {
        let n = self.positions.len();
        let mut prev = vec![[0.0f32; 3]; n];
        for i in 0..n {
            if self.inv_masses[i] < 1e-9 {
                prev[i] = self.positions[i];
                continue;
            }
            prev[i] = self.positions[i];
            self.velocities[i][0] += self.gravity[0] * dt;
            self.velocities[i][1] += self.gravity[1] * dt;
            self.velocities[i][2] += self.gravity[2] * dt;
            self.positions[i][0] += self.velocities[i][0] * dt;
            self.positions[i][1] += self.velocities[i][1] * dt;
            self.positions[i][2] += self.velocities[i][2] * dt;
        }
        // Volume constraints — real XPBD tetrahedral volume gradient projection.
        //
        // For a tet (p0,p1,p2,p3):
        //   V  = (1/6) * (p1-p0) · ((p2-p0) × (p3-p0))
        //   C  = V - V_rest
        //
        // Analytic constraint gradients (face normals scaled by 1/6):
        //   ∇₁C = (1/6) * (p2-p0) × (p3-p0)
        //   ∇₂C = (1/6) * (p3-p0) × (p1-p0)
        //   ∇₃C = (1/6) * (p1-p0) × (p2-p0)
        //   ∇₀C = -(∇₁C + ∇₂C + ∇₃C)   [linear-momentum conservation]
        //
        // XPBD update:
        //   α̃    = compliance / dt²
        //   w_sum = Σᵢ wᵢ |∇ᵢC|²
        //   Δλ   = -(C + α̃·λ) / (w_sum + α̃)
        //   λ    += Δλ
        //   Δpᵢ  = wᵢ · ∇ᵢC · Δλ

        // We need mutable access to both `constraints` and `positions`/`inv_masses`.
        // Borrow positions and inv_masses as raw-pointer copies to allow simultaneous
        // mutable iteration over constraints and mutation of positions.
        // Safety: constraints never alias positions/inv_masses (they are separate Vec fields).
        let n_constraints = self.constraints.len();
        for ci in 0..n_constraints {
            let [i0, i1, i2, i3] = self.constraints[ci].indices;

            let p0 = self.positions[i0];
            let p1 = self.positions[i1];
            let p2 = self.positions[i2];
            let p3 = self.positions[i3];

            // Edge vectors from p0.
            let e1 = sub3(p1, p0); // p1 - p0
            let e2 = sub3(p2, p0); // p2 - p0
            let e3 = sub3(p3, p0); // p3 - p0

            // Signed volume  V = (1/6) e1 · (e2 × e3)
            let e2_cross_e3 = cross(e2, e3);
            let vol = dot3(e1, e2_cross_e3) / 6.0;

            let rest_volume = self.constraints[ci].rest_volume;
            let constraint_val = vol - rest_volume;

            // Analytic gradients (each is a face-normal / 6).
            let grad1 = scale3(cross(e2, e3), 1.0 / 6.0); // (p2-p0) × (p3-p0) / 6
            let grad2 = scale3(cross(e3, e1), 1.0 / 6.0); // (p3-p0) × (p1-p0) / 6
            let grad3 = scale3(cross(e1, e2), 1.0 / 6.0); // (p1-p0) × (p2-p0) / 6
                                                          // ∇₀C = -(∇₁C + ∇₂C + ∇₃C) by momentum conservation.
            let grad0 = [
                -(grad1[0] + grad2[0] + grad3[0]),
                -(grad1[1] + grad2[1] + grad3[1]),
                -(grad1[2] + grad2[2] + grad3[2]),
            ];

            let w0 = self.inv_masses[i0];
            let w1 = self.inv_masses[i1];
            let w2 = self.inv_masses[i2];
            let w3 = self.inv_masses[i3];

            // Weighted sum of squared gradient magnitudes.
            let w_sum = w0 * len3_sq(grad0)
                + w1 * len3_sq(grad1)
                + w2 * len3_sq(grad2)
                + w3 * len3_sq(grad3);

            let alpha_tilde = self.constraints[ci].compliance / (dt * dt);

            // Degenerate check: skip if the tet is flat or all particles are static.
            if w_sum + alpha_tilde < 1e-12 {
                continue;
            }

            let lambda_prev = self.constraints[ci].lambda;
            let delta_lambda =
                -(constraint_val + alpha_tilde * lambda_prev) / (w_sum + alpha_tilde);
            self.constraints[ci].lambda += delta_lambda;

            // Apply positional corrections (only to dynamic particles, wᵢ > 0).
            if w0 > 0.0 {
                let dp = scale3(grad0, w0 * delta_lambda);
                self.positions[i0][0] += dp[0];
                self.positions[i0][1] += dp[1];
                self.positions[i0][2] += dp[2];
            }
            if w1 > 0.0 {
                let dp = scale3(grad1, w1 * delta_lambda);
                self.positions[i1][0] += dp[0];
                self.positions[i1][1] += dp[1];
                self.positions[i1][2] += dp[2];
            }
            if w2 > 0.0 {
                let dp = scale3(grad2, w2 * delta_lambda);
                self.positions[i2][0] += dp[0];
                self.positions[i2][1] += dp[1];
                self.positions[i2][2] += dp[2];
            }
            if w3 > 0.0 {
                let dp = scale3(grad3, w3 * delta_lambda);
                self.positions[i3][0] += dp[0];
                self.positions[i3][1] += dp[1];
                self.positions[i3][2] += dp[2];
            }
        }
        // Update velocities.
        let inv_dt = 1.0 / dt.max(1e-9);
        for i in 0..n {
            if self.inv_masses[i] < 1e-9 {
                continue;
            }
            self.velocities[i][0] = (self.positions[i][0] - prev[i][0]) * inv_dt;
            self.velocities[i][1] = (self.positions[i][1] - prev[i][1]) * inv_dt;
            self.velocities[i][2] = (self.positions[i][2] - prev[i][2]) * inv_dt;
        }
    }

    pub fn particle_count(&self) -> usize {
        self.positions.len()
    }

    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.inv_masses.clear();
        self.constraints.clear();
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for XpbdVolume {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_xpbd_volume() -> XpbdVolume {
    XpbdVolume::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tet_volume_positive() {
        let pts = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let v = tet_signed_volume(&pts).abs();
        assert!((v - 1.0 / 6.0).abs() < 1e-5);
    }

    #[test]
    fn particle_falls() {
        let mut s = new_xpbd_volume();
        s.add_particle([0.0, 10.0, 0.0], 1.0);
        s.step(0.5, 5);
        assert!(s.positions[0][1] < 10.0);
    }

    #[test]
    fn static_particle_fixed() {
        let mut s = new_xpbd_volume();
        s.add_particle([0.0, 5.0, 0.0], 0.0);
        s.step(1.0, 5);
        assert!((s.positions[0][1] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn step_count() {
        let mut s = new_xpbd_volume();
        s.add_particle([0.0; 3], 1.0);
        s.step(0.1, 2);
        assert_eq!(s.steps, 1);
    }

    #[test]
    fn time_advances() {
        let mut s = new_xpbd_volume();
        s.step(0.2, 1);
        assert!((s.time - 0.2).abs() < 1e-5);
    }

    #[test]
    fn particle_count() {
        let mut s = new_xpbd_volume();
        s.add_particle([0.0; 3], 1.0);
        s.add_particle([1.0, 0.0, 0.0], 1.0);
        assert_eq!(s.particle_count(), 2);
    }

    #[test]
    fn constraint_registered() {
        let mut s = new_xpbd_volume();
        s.add_particle([0.0, 0.0, 0.0], 1.0);
        s.add_particle([1.0, 0.0, 0.0], 1.0);
        s.add_particle([0.0, 1.0, 0.0], 1.0);
        s.add_particle([0.0, 0.0, 1.0], 1.0);
        s.add_volume_constraint([0, 1, 2, 3], 0.0);
        assert_eq!(s.constraint_count(), 1);
    }

    #[test]
    fn clear_resets() {
        let mut s = new_xpbd_volume();
        s.add_particle([0.0; 3], 1.0);
        s.step(0.1, 1);
        s.clear();
        assert_eq!(s.particle_count(), 0);
    }

    #[test]
    fn default_valid() {
        let s = XpbdVolume::default();
        assert_eq!(s.particle_count(), 0);
    }

    #[test]
    fn gravity_direction() {
        let mut s = new_xpbd_volume();
        s.add_particle([0.0, 0.0, 0.0], 1.0);
        s.step(0.5, 5);
        assert!(s.velocities[0][1] < 0.0);
    }

    // Helper: unit-tet positions used by the new tests.
    fn unit_tet_positions() -> [[f32; 3]; 4] {
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    /// Real XPBD gradient should drive the tet's volume back toward rest_volume.
    /// We perturb one vertex, run substeps, and assert the volume deviation shrinks.
    #[test]
    fn test_volume_constraint_restores_volume() {
        let mut s = new_xpbd_volume();
        // Disable gravity so volume change is solely from the constraint.
        s.gravity = [0.0, 0.0, 0.0];

        let pts = unit_tet_positions();
        for &p in &pts {
            s.add_particle(p, 1.0);
        }
        // Constraint is added BEFORE perturbation so rest_volume matches original tet.
        s.add_volume_constraint([0, 1, 2, 3], 0.0);

        // Perturb vertex 3 outward — this increases the volume.
        s.positions[3] = [0.0, 0.0, 2.0];

        // Compute initial deviation |V - V_rest|.
        let rest_volume = s.constraints[0].rest_volume;
        let initial_pts: [[f32; 3]; 4] = [
            s.positions[0],
            s.positions[1],
            s.positions[2],
            s.positions[3],
        ];
        let initial_err = (tet_signed_volume(&initial_pts).abs() - rest_volume).abs();

        // Run 10 substeps with a small dt.
        for _ in 0..10 {
            s.step(0.016, 1);
        }

        let final_pts: [[f32; 3]; 4] = [
            s.positions[0],
            s.positions[1],
            s.positions[2],
            s.positions[3],
        ];
        let final_err = (tet_signed_volume(&final_pts).abs() - rest_volume).abs();

        assert!(
            final_err < initial_err,
            "volume deviation did not decrease: initial={initial_err}, final={final_err}"
        );
    }

    /// A tet exactly at rest volume with no gravity → no positional update (C=0 → Δλ=0).
    #[test]
    fn test_volume_constraint_rest_state_stillness() {
        let mut s = new_xpbd_volume();
        s.gravity = [0.0, 0.0, 0.0];

        let pts = unit_tet_positions();
        for &p in &pts {
            s.add_particle(p, 1.0);
        }
        s.add_volume_constraint([0, 1, 2, 3], 0.0);

        let before: Vec<[f32; 3]> = s.positions.clone();
        s.step(0.016, 4);
        let after: Vec<[f32; 3]> = s.positions.clone();

        for i in 0..4 {
            let d2 = (before[i][0] - after[i][0]).powi(2)
                + (before[i][1] - after[i][1]).powi(2)
                + (before[i][2] - after[i][2]).powi(2);
            assert!(d2 < 1e-10, "particle {i} moved at rest state: d²={d2}");
        }
    }

    /// The analytic gradients sum to zero (∇₀+∇₁+∇₂+∇₃=0), so a single projection
    /// step must leave the centre of mass unchanged (linear momentum conservation).
    #[test]
    fn test_volume_constraint_momentum_conservation() {
        let mut s = new_xpbd_volume();
        s.gravity = [0.0, 0.0, 0.0];

        let pts = unit_tet_positions();
        for &p in &pts {
            s.add_particle(p, 1.0);
        }
        s.add_volume_constraint([0, 1, 2, 3], 0.0);

        // Perturb to create a non-zero constraint value.
        s.positions[3] = [0.0, 0.0, 1.5];

        let pos_before: Vec<[f32; 3]> = s.positions.clone();
        // One substep only (one projection pass).
        s.step(0.016, 1);
        let pos_after: Vec<[f32; 3]> = s.positions.clone();

        // Sum of positional deltas must be ≈ 0 (all masses equal → sum Δpᵢ = 0).
        let mut delta_sum = [0.0f32; 3];
        for i in 0..4 {
            delta_sum[0] += pos_after[i][0] - pos_before[i][0];
            delta_sum[1] += pos_after[i][1] - pos_before[i][1];
            delta_sum[2] += pos_after[i][2] - pos_before[i][2];
        }
        let mag = (delta_sum[0].powi(2) + delta_sum[1].powi(2) + delta_sum[2].powi(2)).sqrt();
        assert!(
            mag < 1e-5,
            "centre-of-mass shifted by {mag}: momentum not conserved"
        );
    }
}
