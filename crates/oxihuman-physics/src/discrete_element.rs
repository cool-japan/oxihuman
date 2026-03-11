// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! DEM particle simulation stub — models granular media as collections of
//! rigid spheres with Hertzian contact and simple friction.

/// A DEM sphere particle.
#[derive(Debug, Clone)]
pub struct DemParticle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub radius: f64,
    pub mass: f64,
    pub force_x: f64,
    pub force_y: f64,
}

impl DemParticle {
    pub fn new(x: f64, y: f64, radius: f64, mass: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            radius,
            mass,
            force_x: 0.0,
            force_y: 0.0,
        }
    }
}

/// DEM simulation.
pub struct DiscreteElement {
    pub particles: Vec<DemParticle>,
    pub k_normal: f64, /* normal contact stiffness */
    pub gravity: f64,
}

impl DiscreteElement {
    /// Create a new DEM simulation.
    pub fn new(k_normal: f64, gravity: f64) -> Self {
        Self {
            particles: Vec::new(),
            k_normal,
            gravity,
        }
    }

    /// Add a particle.
    pub fn add_particle(&mut self, x: f64, y: f64, radius: f64, mass: f64) -> usize {
        let idx = self.particles.len();
        self.particles.push(DemParticle::new(x, y, radius, mass));
        idx
    }

    /// Reset forces to gravity.
    pub fn reset_forces(&mut self) {
        for p in &mut self.particles {
            p.force_x = 0.0;
            p.force_y = -p.mass * self.gravity;
        }
    }

    /// Compute contact forces between overlapping spheres.
    pub fn compute_contact_forces(&mut self) {
        let n = self.particles.len();
        let mut fx = vec![0.0f64; n];
        let mut fy = vec![0.0f64; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.particles[j].x - self.particles[i].x;
                let dy = self.particles[j].y - self.particles[i].y;
                let dist = (dx * dx + dy * dy).sqrt();
                let overlap = self.particles[i].radius + self.particles[j].radius - dist;
                if overlap > 0.0 && dist > 1e-14 {
                    let fn_mag = self.k_normal * overlap;
                    let nx = dx / dist;
                    let ny = dy / dist;
                    fx[i] -= fn_mag * nx;
                    fy[i] -= fn_mag * ny;
                    fx[j] += fn_mag * nx;
                    fy[j] += fn_mag * ny;
                }
            }
        }
        for (i, p) in self.particles.iter_mut().enumerate() {
            p.force_x += fx[i];
            p.force_y += fy[i];
        }
    }

    /// Integrate velocities and positions.
    pub fn integrate(&mut self, dt: f64) {
        for p in &mut self.particles {
            if p.mass > 0.0 {
                p.vx += p.force_x / p.mass * dt;
                p.vy += p.force_y / p.mass * dt;
            }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
    }

    /// Count overlapping pairs.
    pub fn overlap_count(&self) -> usize {
        let n = self.particles.len();
        let mut count = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.particles[j].x - self.particles[i].x;
                let dy = self.particles[j].y - self.particles[i].y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < self.particles[i].radius + self.particles[j].radius {
                    count += 1;
                }
            }
        }
        count
    }

    /// Number of particles.
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

/// Create a new DEM simulation.
pub fn new_discrete_element(k_normal: f64, gravity: f64) -> DiscreteElement {
    DiscreteElement::new(k_normal, gravity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_particle() {
        let mut dem = DiscreteElement::new(1e3, 9.81);
        let idx = dem.add_particle(0.0, 0.0, 0.5, 1.0);
        assert_eq!(idx, 0); /* first particle index */
    }

    #[test]
    fn test_particle_count() {
        let mut dem = DiscreteElement::new(1e3, 9.81);
        dem.add_particle(0.0, 0.0, 0.5, 1.0);
        dem.add_particle(2.0, 0.0, 0.5, 1.0);
        assert_eq!(dem.particle_count(), 2); /* two particles */
    }

    #[test]
    fn test_overlap_count_no_overlap() {
        let mut dem = DiscreteElement::new(1e3, 9.81);
        dem.add_particle(0.0, 0.0, 0.4, 1.0);
        dem.add_particle(2.0, 0.0, 0.4, 1.0); /* far apart */
        assert_eq!(dem.overlap_count(), 0); /* no overlap */
    }

    #[test]
    fn test_overlap_count_with_overlap() {
        let mut dem = DiscreteElement::new(1e3, 9.81);
        dem.add_particle(0.0, 0.0, 0.5, 1.0);
        dem.add_particle(0.5, 0.0, 0.5, 1.0); /* overlapping */
        assert_eq!(dem.overlap_count(), 1); /* one overlap */
    }

    #[test]
    fn test_contact_force_separates() {
        let mut dem = DiscreteElement::new(1e3, 0.0);
        dem.add_particle(0.0, 0.0, 0.5, 1.0);
        dem.add_particle(0.5, 0.0, 0.5, 1.0); /* overlapping by 0.5 */
        dem.reset_forces();
        dem.compute_contact_forces();
        /* particle 0 should be pushed left (negative x) */
        assert!(dem.particles[0].force_x < 0.0); /* repelled left */
        assert!(dem.particles[1].force_x > 0.0); /* repelled right */
    }

    #[test]
    fn test_gravity_sets_force() {
        let mut dem = DiscreteElement::new(1e3, 9.81);
        dem.add_particle(0.0, 10.0, 0.5, 2.0);
        dem.reset_forces();
        assert!((dem.particles[0].force_y + 2.0 * 9.81).abs() < 1e-10); /* gravity force */
    }

    #[test]
    fn test_integrate_moves_particle() {
        let mut dem = DiscreteElement::new(0.0, 9.81);
        dem.add_particle(0.0, 10.0, 0.5, 1.0);
        dem.reset_forces();
        dem.integrate(0.1);
        assert!(dem.particles[0].vy < 0.0); /* falling */
    }

    #[test]
    fn test_new_helper() {
        let dem = new_discrete_element(500.0, 9.81);
        assert_eq!(dem.particle_count(), 0); /* empty on creation */
    }

    #[test]
    fn test_three_particles_overlap() {
        let mut dem = DiscreteElement::new(1e3, 0.0);
        dem.add_particle(0.0, 0.0, 0.5, 1.0);
        dem.add_particle(0.6, 0.0, 0.5, 1.0);
        dem.add_particle(1.2, 0.0, 0.5, 1.0);
        /* (0,1) and (1,2) overlap */
        assert_eq!(dem.overlap_count(), 2); /* two overlapping pairs */
    }
}
