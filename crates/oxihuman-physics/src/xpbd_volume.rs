// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
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
}

#[allow(dead_code)]
impl VolumeConstraint {
    pub fn new(indices: [usize; 4], rest_volume: f32, compliance: f32) -> Self {
        Self {
            indices,
            rest_volume,
            compliance,
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
        // Volume constraints (simplified gradient-based).
        let constraints = self.constraints.clone();
        for c in &constraints {
            let pts: [[f32; 3]; 4] = [
                self.positions[c.indices[0]],
                self.positions[c.indices[1]],
                self.positions[c.indices[2]],
                self.positions[c.indices[3]],
            ];
            let vol = tet_signed_volume(&pts);
            let err = vol.abs() - c.rest_volume;
            // Simple correction: scale positions towards rest volume.
            if err.abs() > 1e-6 {
                let alpha = c.compliance / (dt * dt);
                let w_sum: f32 = c.indices.iter().map(|&i| self.inv_masses[i]).sum();
                if w_sum < 1e-9 {
                    continue;
                }
                let lagrange = -err / (w_sum + alpha);
                // Approximate gradient as centroid direction.
                let cm = [
                    (pts[0][0] + pts[1][0] + pts[2][0] + pts[3][0]) / 4.0,
                    (pts[0][1] + pts[1][1] + pts[2][1] + pts[3][1]) / 4.0,
                    (pts[0][2] + pts[1][2] + pts[2][2] + pts[3][2]) / 4.0,
                ];
                for &idx in &c.indices {
                    let w = self.inv_masses[idx];
                    let d = [
                        self.positions[idx][0] - cm[0],
                        self.positions[idx][1] - cm[1],
                        self.positions[idx][2] - cm[2],
                    ];
                    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
                    self.positions[idx][0] += w * lagrange * d[0] / len;
                    self.positions[idx][1] += w * lagrange * d[1] / len;
                    self.positions[idx][2] += w * lagrange * d[2] / len;
                }
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
}
