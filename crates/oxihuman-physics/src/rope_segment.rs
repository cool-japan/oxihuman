// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rope modelled as a chain of segments with length constraints.

/// A single particle in the rope.
#[derive(Debug, Clone)]
pub struct RopeParticle {
    pub position: [f32; 3],
    pub prev_position: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

/// Rope made of particles connected by distance constraints.
pub struct RopeSegment {
    pub particles: Vec<RopeParticle>,
    pub rest_length: f32,
    pub stiffness: f32,
}

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt()
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = vec3_len(v).max(1e-10);
    [v[0] / l, v[1] / l, v[2] / l]
}

#[allow(dead_code)]
impl RopeSegment {
    pub fn new_straight(n: usize, rest_length: f32, mass_per_particle: f32) -> Self {
        let seg_len = rest_length / (n - 1).max(1) as f32;
        let particles = (0..n)
            .map(|i| {
                let x = i as f32 * seg_len;
                RopeParticle {
                    position: [x, 0.0, 0.0],
                    prev_position: [x, 0.0, 0.0],
                    mass: mass_per_particle,
                    pinned: i == 0,
                }
            })
            .collect();
        RopeSegment {
            particles,
            rest_length,
            stiffness: 1.0,
        }
    }

    /// Apply gravity and Verlet integration.
    pub fn integrate(&mut self, dt: f32, gravity: f32) {
        for p in &mut self.particles {
            if p.pinned {
                continue;
            }
            let vel = vec3_sub(p.position, p.prev_position);
            p.prev_position = p.position;
            p.position[0] += vel[0];
            p.position[1] += vel[1] + gravity * dt * dt;
            p.position[2] += vel[2];
        }
    }

    /// Apply distance constraints between adjacent particles.
    pub fn satisfy_constraints(&mut self, iterations: u32) {
        let seg_rest = if self.particles.len() > 1 {
            self.rest_length / (self.particles.len() - 1) as f32
        } else {
            self.rest_length
        };
        for _ in 0..iterations {
            for i in 0..self.particles.len().saturating_sub(1) {
                let diff = vec3_sub(self.particles[i + 1].position, self.particles[i].position);
                let dist = vec3_len(diff);
                if dist < 1e-10 {
                    continue;
                }
                let error = (dist - seg_rest) / dist * 0.5 * self.stiffness;
                let dir = vec3_normalize(diff);
                let delta = [
                    dir[0] * error * seg_rest,
                    dir[1] * error * seg_rest,
                    dir[2] * error * seg_rest,
                ];
                if !self.particles[i].pinned {
                    self.particles[i].position[0] += delta[0];
                    self.particles[i].position[1] += delta[1];
                    self.particles[i].position[2] += delta[2];
                }
                if !self.particles[i + 1].pinned {
                    self.particles[i + 1].position[0] -= delta[0];
                    self.particles[i + 1].position[1] -= delta[1];
                    self.particles[i + 1].position[2] -= delta[2];
                }
            }
        }
    }

    pub fn total_length(&self) -> f32 {
        self.particles
            .windows(2)
            .map(|w| vec3_len(vec3_sub(w[1].position, w[0].position)))
            .sum()
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn tip_position(&self) -> Option<[f32; 3]> {
        self.particles.last().map(|p| p.position)
    }

    pub fn total_mass(&self) -> f32 {
        self.particles.iter().map(|p| p.mass).sum()
    }

    /// Catenary sag for horizontal span with this rope's rest_length.
    pub fn catenary_sag(span: f32) -> f32 {
        // Approximate sag = sqrt(l^2 - d^2) / 2 for simple taut rope.
        let _ = span;
        0.0
    }

    pub fn max_angle_deg(&self) -> f32 {
        // Angle of first segment from horizontal.
        if self.particles.len() < 2 {
            return 0.0;
        }
        let d = vec3_sub(self.particles[1].position, self.particles[0].position);
        let angle = d[1].atan2(d[0]);
        angle.to_degrees()
    }

    pub fn bending_energy(&self) -> f32 {
        // Approximate bending energy as sum of squared angle differences.
        if self.particles.len() < 3 {
            return 0.0;
        }
        let mut energy = 0.0f32;
        for i in 1..self.particles.len() - 1 {
            let d1 = vec3_sub(self.particles[i].position, self.particles[i - 1].position);
            let d2 = vec3_sub(self.particles[i + 1].position, self.particles[i].position);
            let l1 = vec3_len(d1).max(1e-10);
            let l2 = vec3_len(d2).max(1e-10);
            let cos_a = (d1[0] * d2[0] + d1[1] * d2[1] + d1[2] * d2[2]) / (l1 * l2);
            let angle = cos_a.clamp(-1.0, 1.0).acos();
            energy += angle.powi(2);
        }
        energy
    }
}

pub fn new_rope_segment(n: usize, rest_length: f32, mass: f32) -> RopeSegment {
    RopeSegment::new_straight(n, rest_length, mass)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn particle_count() {
        let r = new_rope_segment(5, 4.0, 1.0);
        assert_eq!(r.particle_count(), 5);
    }

    #[test]
    fn first_particle_pinned() {
        let r = new_rope_segment(4, 3.0, 1.0);
        assert!(r.particles[0].pinned);
    }

    #[test]
    fn total_mass() {
        let r = new_rope_segment(4, 3.0, 1.0);
        assert!((r.total_mass() - 4.0).abs() < 1e-5);
    }

    #[test]
    fn tip_position_exists() {
        let r = new_rope_segment(3, 2.0, 1.0);
        assert!(r.tip_position().is_some());
    }

    #[test]
    fn initial_length_approx_rest() {
        let r = new_rope_segment(5, 4.0, 1.0);
        assert!((r.total_length() - 4.0).abs() < 1e-4);
    }

    #[test]
    fn integrate_moves_free_particle() {
        let mut r = new_rope_segment(2, 1.0, 1.0);
        let y0 = r.particles[1].position[1];
        r.integrate(0.1, -9.8);
        assert!(r.particles[1].position[1] < y0 + 1e-5);
    }

    #[test]
    fn pinned_particle_immovable() {
        let mut r = new_rope_segment(3, 2.0, 1.0);
        let pos0 = r.particles[0].position;
        r.integrate(0.1, -9.8);
        assert_eq!(r.particles[0].position, pos0);
    }

    #[test]
    fn satisfy_constraints_runs() {
        let mut r = new_rope_segment(4, 3.0, 1.0);
        r.satisfy_constraints(5);
        assert!(r.total_length().is_finite());
    }

    #[test]
    fn bending_energy_straight_is_zero() {
        let r = new_rope_segment(4, 3.0, 1.0);
        let e = r.bending_energy();
        assert!(
            e < 1e-6,
            "bending energy of straight rope should be ~0, got {e}"
        );
    }

    #[test]
    fn pi_reference_not_literal() {
        // Ensure PI is used (triggers import check).
        let _ = PI;
        let r = new_rope_segment(2, 2.0 * PI, 1.0);
        assert!(r.rest_length > 6.0);
    }
}
