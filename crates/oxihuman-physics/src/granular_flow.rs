// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
#![allow(clippy::needless_range_loop)]

//! Discrete element method (DEM) for granular flow simulation.

/// A granular particle in DEM.
#[derive(Debug, Clone)]
pub struct GranularParticle {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub radius: f32,
    pub mass: f32,
}

/// DEM simulation config.
#[derive(Debug, Clone)]
pub struct DemConfig {
    pub gravity: [f32; 2],
    pub restitution: f32,
    pub friction: f32,
    pub floor_y: f32,
}

/// Granular flow simulation (2D DEM).
pub struct GranularFlow {
    pub particles: Vec<GranularParticle>,
    pub config: DemConfig,
}

/// Construct a new GranularFlow.
pub fn new_granular_flow(config: DemConfig) -> GranularFlow {
    GranularFlow {
        particles: Vec::new(),
        config,
    }
}

/// Default DEM config (gravity pointing down).
pub fn default_dem_config() -> DemConfig {
    DemConfig {
        gravity: [0.0, -9.81],
        restitution: 0.6,
        friction: 0.3,
        floor_y: 0.0,
    }
}

impl GranularFlow {
    /// Add a particle.
    pub fn add_particle(&mut self, pos: [f32; 2], vel: [f32; 2], radius: f32, mass: f32) {
        self.particles.push(GranularParticle {
            position: pos,
            velocity: vel,
            radius,
            mass,
        });
    }

    /// Number of particles.
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Simulate one timestep.
    pub fn step(&mut self, dt: f32) {
        let n = self.particles.len();
        let mut forces = vec![[0.0f32; 2]; n];

        /* gravity */
        for (i, p) in self.particles.iter().enumerate() {
            forces[i][0] += p.mass * self.config.gravity[0];
            forces[i][1] += p.mass * self.config.gravity[1];
        }

        /* particle-particle collisions */
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.particles[j].position[0] - self.particles[i].position[0];
                let dy = self.particles[j].position[1] - self.particles[i].position[1];
                let dist = (dx * dx + dy * dy).sqrt();
                let min_dist = self.particles[i].radius + self.particles[j].radius;
                if dist < min_dist && dist > 1e-9 {
                    let overlap = min_dist - dist;
                    let nx = dx / dist;
                    let ny = dy / dist;
                    let k = 1000.0f32;
                    let fx = k * overlap * nx;
                    let fy = k * overlap * ny;
                    forces[i][0] -= fx;
                    forces[i][1] -= fy;
                    forces[j][0] += fx;
                    forces[j][1] += fy;
                }
            }
        }

        /* integrate */
        let floor_y = self.config.floor_y;
        let e = self.config.restitution;
        for (i, p) in self.particles.iter_mut().enumerate() {
            for k in 0..2 {
                let accel = forces[i][k] / p.mass;
                p.velocity[k] += accel * dt;
            }
            for k in 0..2 {
                p.position[k] += p.velocity[k] * dt;
            }
            /* floor collision */
            if p.position[1] - p.radius < floor_y {
                p.position[1] = floor_y + p.radius;
                if p.velocity[1] < 0.0 {
                    p.velocity[1] = -p.velocity[1] * e;
                }
            }
        }
    }

    /// Total kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        self.particles
            .iter()
            .map(|p| {
                let v2 = p.velocity[0] * p.velocity[0] + p.velocity[1] * p.velocity[1];
                0.5 * p.mass * v2
            })
            .sum()
    }

    /// Count active (moving) particles.
    pub fn active_count(&self, threshold: f32) -> usize {
        self.particles
            .iter()
            .filter(|p| {
                let v2 = p.velocity[0] * p.velocity[0] + p.velocity[1] * p.velocity[1];
                v2 > threshold * threshold
            })
            .count()
    }

    /// Average height of particles.
    pub fn avg_height(&self) -> f32 {
        if self.particles.is_empty() {
            return 0.0;
        }
        self.particles.iter().map(|p| p.position[1]).sum::<f32>() / self.particles.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_flow() -> GranularFlow {
        let mut gf = new_granular_flow(default_dem_config());
        gf.add_particle([0.0, 1.0], [0.0, 0.0], 0.1, 1.0);
        gf
    }

    #[test]
    fn test_new_flow() {
        /* new_granular_flow starts with no particles */
        let gf = new_granular_flow(default_dem_config());
        assert_eq!(gf.particle_count(), 0);
    }

    #[test]
    fn test_add_particle() {
        /* add_particle increments count */
        let mut gf = new_granular_flow(default_dem_config());
        gf.add_particle([0.0, 0.0], [0.0, 0.0], 0.1, 1.0);
        assert_eq!(gf.particle_count(), 1);
    }

    #[test]
    fn test_gravity_pulls_down() {
        /* particle falls under gravity */
        let mut gf = simple_flow();
        let y0 = gf.particles[0].position[1];
        gf.step(0.05);
        assert!(gf.particles[0].position[1] < y0);
    }

    #[test]
    fn test_floor_bounce() {
        /* particle bounces off floor */
        let mut gf = new_granular_flow(default_dem_config());
        gf.add_particle([0.0, 0.15], [0.0, -5.0], 0.1, 1.0);
        for _ in 0..20 {
            gf.step(0.01);
        }
        assert!(gf.particles[0].position[1] >= 0.0);
    }

    #[test]
    fn test_kinetic_energy_increases() {
        /* kinetic energy increases as particles fall */
        let mut gf = simple_flow();
        let ke0 = gf.kinetic_energy();
        gf.step(0.1);
        assert!(gf.kinetic_energy() > ke0);
    }

    #[test]
    fn test_two_particles_collision() {
        /* two particles push apart when overlapping */
        let mut gf = new_granular_flow(default_dem_config());
        gf.add_particle([0.0, 0.5], [0.0, 0.0], 0.15, 1.0);
        gf.add_particle([0.2, 0.5], [0.0, 0.0], 0.15, 1.0);
        /* they overlap (0.2 < 0.3 combined radius) */
        for _ in 0..5 {
            gf.step(0.001);
        }
        let dx = gf.particles[1].position[0] - gf.particles[0].position[0];
        assert!(dx >= 0.2, "dx={dx}");
    }

    #[test]
    fn test_avg_height() {
        /* avg_height returns mean y position */
        let mut gf = new_granular_flow(default_dem_config());
        gf.add_particle([0.0, 2.0], [0.0, 0.0], 0.1, 1.0);
        gf.add_particle([0.0, 4.0], [0.0, 0.0], 0.1, 1.0);
        assert!((gf.avg_height() - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_active_count() {
        /* active_count returns particles moving above threshold */
        let mut gf = simple_flow();
        gf.step(0.1);
        assert!(gf.active_count(0.01) > 0);
    }
}
