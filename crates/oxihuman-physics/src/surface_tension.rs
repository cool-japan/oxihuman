// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Surface tension force model for fluid interfaces.

/// Configuration for surface tension.
#[derive(Debug, Clone)]
pub struct SurfaceTensionConfig {
    /// Surface tension coefficient gamma (N/m).
    pub gamma: f32,
    /// Smoothing radius for interface detection.
    pub smoothing_radius: f32,
}

impl Default for SurfaceTensionConfig {
    fn default() -> Self {
        SurfaceTensionConfig {
            gamma: 0.072,
            smoothing_radius: 0.05,
        }
    }
}

/// A fluid particle with surface tension properties.
#[derive(Debug, Clone)]
pub struct SurfaceParticle {
    pub position: [f32; 3],
    pub density: f32,
    pub is_surface: bool,
}

/// Surface tension model.
pub struct SurfaceTensionModel {
    pub config: SurfaceTensionConfig,
    pub particles: Vec<SurfaceParticle>,
}

/// Construct a new SurfaceTensionModel.
pub fn new_surface_tension_model(config: SurfaceTensionConfig) -> SurfaceTensionModel {
    SurfaceTensionModel {
        config,
        particles: Vec::new(),
    }
}

impl SurfaceTensionModel {
    /// Add a particle.
    pub fn add_particle(&mut self, pos: [f32; 3], density: f32) {
        self.particles.push(SurfaceParticle {
            position: pos,
            density,
            is_surface: false,
        });
    }

    /// Detect which particles are on the surface (low neighbor count).
    pub fn detect_surface(&mut self) {
        let n = self.particles.len();
        let r = self.config.smoothing_radius;
        let r2 = r * r;
        for i in 0..n {
            let mut neighbors = 0usize;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let dx = self.particles[j].position[0] - self.particles[i].position[0];
                let dy = self.particles[j].position[1] - self.particles[i].position[1];
                let dz = self.particles[j].position[2] - self.particles[i].position[2];
                if dx * dx + dy * dy + dz * dz < r2 {
                    neighbors += 1;
                }
            }
            self.particles[i].is_surface = neighbors < 4;
        }
    }

    /// Compute the surface tension force on particle `i` (Young-Laplace approximation).
    pub fn force_on(&self, i: usize) -> [f32; 3] {
        if i >= self.particles.len() || !self.particles[i].is_surface {
            return [0.0; 3];
        }
        let r = self.config.smoothing_radius;
        let curvature = 2.0 / r;
        let pressure = self.config.gamma * curvature;
        let pi = self.particles[i].position;
        let mut cx = 0.0f32;
        let mut cy = 0.0f32;
        let mut cz = 0.0f32;
        let mut count = 0;
        let r2 = r * r;
        for (j, pj) in self.particles.iter().enumerate() {
            if j == i {
                continue;
            }
            let dx = pj.position[0] - pi[0];
            let dy = pj.position[1] - pi[1];
            let dz = pj.position[2] - pi[2];
            if dx * dx + dy * dy + dz * dz < r2 {
                cx += dx;
                cy += dy;
                cz += dz;
                count += 1;
            }
        }
        if count == 0 {
            return [0.0; 3];
        }
        let len = (cx * cx + cy * cy + cz * cz).sqrt().max(1e-9);
        [
            pressure * cx / len,
            pressure * cy / len,
            pressure * cz / len,
        ]
    }

    /// Number of surface particles.
    pub fn surface_count(&self) -> usize {
        self.particles.iter().filter(|p| p.is_surface).count()
    }

    /// Total particle count.
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

/// Compute the capillary pressure for a spherical droplet (Young-Laplace).
pub fn laplace_pressure(gamma: f32, radius: f32) -> f32 {
    2.0 * gamma / radius.max(1e-9)
}

/// Compute the capillary length.
pub fn capillary_length(gamma: f32, density: f32, g: f32) -> f32 {
    (gamma / (density * g)).sqrt()
}

/// Weber number: inertial vs. surface tension forces.
pub fn weber_number(density: f32, velocity: f32, length: f32, gamma: f32) -> f32 {
    density * velocity * velocity * length / gamma.max(1e-15)
}

/// Bond number: gravitational vs. surface tension forces.
pub fn bond_number(density: f32, g: f32, length: f32, gamma: f32) -> f32 {
    density * g * length * length / gamma.max(1e-15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_model() {
        /* new model has zero particles */
        let m = new_surface_tension_model(SurfaceTensionConfig::default());
        assert_eq!(m.particle_count(), 0);
    }

    #[test]
    fn test_add_particle() {
        /* add_particle increments count */
        let mut m = new_surface_tension_model(SurfaceTensionConfig::default());
        m.add_particle([0.0, 0.0, 0.0], 1000.0);
        assert_eq!(m.particle_count(), 1);
    }

    #[test]
    fn test_laplace_pressure() {
        /* laplace_pressure for droplet is 2*gamma/r */
        let p = laplace_pressure(0.072, 0.01);
        assert!((p - 14.4).abs() < 0.01, "p={p}");
    }

    #[test]
    fn test_capillary_length() {
        /* capillary length for water is roughly 2-3 mm */
        let cl = capillary_length(0.072, 1000.0, 9.81);
        assert!(cl > 0.001 && cl < 0.01, "cl={cl}");
    }

    #[test]
    fn test_weber_number() {
        /* weber number positive for positive inputs */
        let we = weber_number(1000.0, 1.0, 0.01, 0.072);
        assert!(we > 0.0);
    }

    #[test]
    fn test_bond_number() {
        /* bond number positive for positive inputs */
        let bo = bond_number(1000.0, 9.81, 0.01, 0.072);
        assert!(bo > 0.0);
    }

    #[test]
    fn test_detect_surface_marks_isolated() {
        /* isolated particle is flagged as surface */
        let mut m = new_surface_tension_model(SurfaceTensionConfig::default());
        m.add_particle([0.0, 0.0, 0.0], 1000.0);
        m.detect_surface();
        assert!(m.particles[0].is_surface);
    }

    #[test]
    fn test_surface_count() {
        /* surface_count returns number of flagged particles */
        let mut m = new_surface_tension_model(SurfaceTensionConfig::default());
        m.add_particle([0.0, 0.0, 0.0], 1000.0);
        m.add_particle([10.0, 0.0, 0.0], 1000.0);
        m.detect_surface();
        assert!(m.surface_count() >= 1);
    }

    #[test]
    fn test_force_on_oob_zero() {
        /* force_on out-of-bounds index returns zero */
        let m = new_surface_tension_model(SurfaceTensionConfig::default());
        assert_eq!(m.force_on(99), [0.0; 3]);
    }
}
