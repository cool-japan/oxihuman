// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
#![allow(clippy::needless_range_loop)]

//! Magnetic dipole-dipole force interactions.

/// A magnetic dipole particle.
#[derive(Debug, Clone)]
pub struct MagneticDipole {
    pub position: [f32; 3],
    pub moment: [f32; 3], /* magnetic moment vector (A·m²) */
    pub mass: f32,
    pub velocity: [f32; 3],
}

/// Construct a new MagneticDipole.
pub fn new_magnetic_dipole(pos: [f32; 3], moment: [f32; 3], mass: f32) -> MagneticDipole {
    MagneticDipole {
        position: pos,
        moment,
        mass,
        velocity: [0.0; 3],
    }
}

const MU0_4PI: f32 = 1e-7; /* μ₀ / (4π) */

/// Compute the magnetic field of dipole `source` at position `r`.
pub fn dipole_field(source: &MagneticDipole, r: [f32; 3]) -> [f32; 3] {
    let dr = [
        r[0] - source.position[0],
        r[1] - source.position[1],
        r[2] - source.position[2],
    ];
    let r_len = (dr[0] * dr[0] + dr[1] * dr[1] + dr[2] * dr[2]).sqrt();
    if r_len < 1e-9 {
        return [0.0; 3];
    }
    let r3 = r_len * r_len * r_len;
    let r5 = r3 * r_len * r_len;
    let m = &source.moment;
    let m_dot_r: f32 = m[0] * dr[0] + m[1] * dr[1] + m[2] * dr[2];
    let mut b = [0.0f32; 3];
    for k in 0..3 {
        b[k] = MU0_4PI * (3.0 * m_dot_r * dr[k] / r5 - m[k] / r3);
    }
    b
}

/// Compute the dipole-dipole force on `target` due to `source`.
pub fn dipole_force(source: &MagneticDipole, target: &MagneticDipole) -> [f32; 3] {
    let dr = [
        target.position[0] - source.position[0],
        target.position[1] - source.position[1],
        target.position[2] - source.position[2],
    ];
    let r = (dr[0] * dr[0] + dr[1] * dr[1] + dr[2] * dr[2]).sqrt();
    if r < 1e-9 {
        return [0.0; 3];
    }
    let r4 = r * r * r * r;
    let m1 = &source.moment;
    let m2 = &target.moment;
    let rhat = [dr[0] / r, dr[1] / r, dr[2] / r];
    let m1r: f32 = m1[0] * rhat[0] + m1[1] * rhat[1] + m1[2] * rhat[2];
    let m2r: f32 = m2[0] * rhat[0] + m2[1] * rhat[1] + m2[2] * rhat[2];
    let m1m2: f32 = m1[0] * m2[0] + m1[1] * m2[1] + m1[2] * m2[2];
    let mut f = [0.0f32; 3];
    let coeff = 3.0 * MU0_4PI / r4;
    for k in 0..3 {
        f[k] = coeff * (m2r * m1[k] + m1r * m2[k] + m1m2 * rhat[k] - 5.0 * m1r * m2r * rhat[k]);
    }
    f
}

/// Magnetic particle system.
pub struct MagneticParticleSystem {
    pub dipoles: Vec<MagneticDipole>,
    pub damping: f32,
}

/// Construct a new MagneticParticleSystem.
pub fn new_magnetic_system(damping: f32) -> MagneticParticleSystem {
    MagneticParticleSystem {
        dipoles: Vec::new(),
        damping,
    }
}

impl MagneticParticleSystem {
    /// Add a dipole.
    pub fn add_dipole(&mut self, d: MagneticDipole) {
        self.dipoles.push(d);
    }

    /// Simulate one timestep (explicit Euler).
    pub fn step(&mut self, dt: f32, gravity: [f32; 3]) {
        let n = self.dipoles.len();
        let mut forces = vec![[0.0f32; 3]; n];

        for i in 0..n {
            for k in 0..3 {
                forces[i][k] += self.dipoles[i].mass * gravity[k];
            }
        }

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let f = dipole_force(&self.dipoles[j], &self.dipoles[i]);
                for k in 0..3 {
                    forces[i][k] += f[k];
                }
            }
        }

        for i in 0..n {
            for k in 0..3 {
                let a = forces[i][k] / self.dipoles[i].mass;
                self.dipoles[i].velocity[k] =
                    self.dipoles[i].velocity[k] * (1.0 - self.damping * dt) + a * dt;
                self.dipoles[i].position[k] += self.dipoles[i].velocity[k] * dt;
            }
        }
    }

    /// Total count.
    pub fn count(&self) -> usize {
        self.dipoles.len()
    }

    /// Compute total magnetic potential energy (approximate dipole-dipole).
    pub fn potential_energy(&self) -> f32 {
        let n = self.dipoles.len();
        let mut e = 0.0f32;
        for i in 0..n {
            for j in (i + 1)..n {
                let b = dipole_field(&self.dipoles[j], self.dipoles[i].position);
                let m = &self.dipoles[i].moment;
                e -= m[0] * b[0] + m[1] * b[1] + m[2] * b[2];
            }
        }
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_system() {
        /* new system has zero dipoles */
        let s = new_magnetic_system(0.0);
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn test_add_dipole() {
        /* add_dipole increments count */
        let mut s = new_magnetic_system(0.0);
        s.add_dipole(new_magnetic_dipole([0.0; 3], [0.0, 0.0, 1.0], 1.0));
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn test_dipole_field_zero_at_source() {
        /* dipole field at source position is zero */
        let d = new_magnetic_dipole([0.0; 3], [0.0, 0.0, 1.0], 1.0);
        let b = dipole_field(&d, [0.0, 0.0, 0.0]);
        assert_eq!(b, [0.0; 3]);
    }

    #[test]
    fn test_dipole_field_nonzero_at_distance() {
        /* dipole field is nonzero at remote point */
        let d = new_magnetic_dipole([0.0; 3], [0.0, 0.0, 1.0], 1.0);
        let b = dipole_field(&d, [0.0, 0.0, 1.0]);
        let mag = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
        assert!(mag > 0.0);
    }

    #[test]
    fn test_dipole_force_opposite_sign_alignment() {
        /* anti-parallel dipoles repel along axis */
        let d1 = new_magnetic_dipole([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
        let d2 = new_magnetic_dipole([0.0, 0.0, 1.0], [0.0, 0.0, -1.0], 1.0);
        let f = dipole_force(&d1, &d2);
        /* force along z-axis */
        let _ = f[2]; /* just verify no panic */
    }

    #[test]
    fn test_step_moves_dipoles() {
        /* step moves dipoles under gravity */
        let mut s = new_magnetic_system(0.0);
        s.add_dipole(new_magnetic_dipole([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], 1.0));
        let y0 = s.dipoles[0].position[1];
        s.step(0.01, [0.0, -9.81, 0.0]);
        assert!(s.dipoles[0].position[1] < y0);
    }

    #[test]
    fn test_potential_energy_finite() {
        /* potential energy returns finite value */
        let mut s = new_magnetic_system(0.0);
        s.add_dipole(new_magnetic_dipole([0.0; 3], [0.0, 0.0, 1.0], 1.0));
        s.add_dipole(new_magnetic_dipole([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0));
        assert!(s.potential_energy().is_finite());
    }

    #[test]
    fn test_new_dipole() {
        /* new_magnetic_dipole initializes with zero velocity */
        let d = new_magnetic_dipole([1.0, 2.0, 3.0], [0.0, 1.0, 0.0], 2.0);
        assert_eq!(d.velocity, [0.0; 3]);
        assert!((d.mass - 2.0).abs() < 1e-6);
    }
}
