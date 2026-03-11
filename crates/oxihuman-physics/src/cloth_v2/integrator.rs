// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Symplectic Euler time integration for cloth simulation.
//!
//! Implements the standard PBD integration scheme:
//! 1. Apply external forces to velocities (symplectic Euler velocity update)
//! 2. Predict new positions from updated velocities
//! 3. After constraint projection, derive final velocities from position deltas

/// Apply external forces (gravity, wind, etc.) to velocities using symplectic Euler.
///
/// For each vertex with `inv_mass > 0`, updates:
///   v += dt * inv_mass * force_per_unit_mass
///
/// Since gravity is mass-independent (F = mg, a = g), `gravity` is applied
/// directly as acceleration. Additional per-vertex external forces can be
/// provided via the `external_forces` slice (force, not acceleration).
pub fn integrate_velocities(
    velocities: &mut [[f64; 3]],
    inv_masses: &[f64],
    gravity: &[f64; 3],
    external_forces: Option<&[[f64; 3]]>,
    dt: f64,
) {
    let n = velocities.len().min(inv_masses.len());

    for i in 0..n {
        let w = inv_masses[i];
        if w <= 0.0 {
            // Fixed/pinned vertex
            continue;
        }

        // Gravity (acceleration, mass-independent)
        velocities[i][0] += gravity[0] * dt;
        velocities[i][1] += gravity[1] * dt;
        velocities[i][2] += gravity[2] * dt;

        // External forces (F -> a = F * inv_mass)
        if let Some(forces) = external_forces {
            if let Some(f) = forces.get(i) {
                velocities[i][0] += f[0] * w * dt;
                velocities[i][1] += f[1] * w * dt;
                velocities[i][2] += f[2] * w * dt;
            }
        }
    }
}

/// Predict new positions from current positions and velocities (symplectic Euler).
///
/// Stores old positions into `prev_positions` and computes:
///   p_new = p_old + dt * v
///
/// The `prev_positions` buffer is used later to derive corrected velocities
/// after constraint projection.
pub fn integrate_positions(
    positions: &mut [[f64; 3]],
    velocities: &[[f64; 3]],
    prev_positions: &mut Vec<[f64; 3]>,
    inv_masses: &[f64],
    dt: f64,
) {
    let n = positions.len().min(velocities.len()).min(inv_masses.len());

    // Save current positions
    prev_positions.clear();
    prev_positions.reserve(n);
    for i in 0..n {
        prev_positions.push(positions[i]);
    }

    // Predict new positions
    for i in 0..n {
        if inv_masses[i] <= 0.0 {
            continue;
        }

        positions[i][0] += velocities[i][0] * dt;
        positions[i][1] += velocities[i][1] * dt;
        positions[i][2] += velocities[i][2] * dt;
    }
}

/// Update velocities from position changes after constraint projection.
///
/// Computes: v = (p_new - p_old) / dt
///
/// This is the standard PBD velocity derivation step, executed after all
/// constraints have been projected.
pub fn update_velocities_from_positions(
    positions: &[[f64; 3]],
    prev_positions: &[[f64; 3]],
    velocities: &mut [[f64; 3]],
    inv_masses: &[f64],
    dt: f64,
) {
    if dt < 1e-30 {
        return;
    }
    let inv_dt = 1.0 / dt;

    let n = positions
        .len()
        .min(prev_positions.len())
        .min(velocities.len())
        .min(inv_masses.len());

    for i in 0..n {
        if inv_masses[i] <= 0.0 {
            velocities[i] = [0.0, 0.0, 0.0];
            continue;
        }

        velocities[i][0] = (positions[i][0] - prev_positions[i][0]) * inv_dt;
        velocities[i][1] = (positions[i][1] - prev_positions[i][1]) * inv_dt;
        velocities[i][2] = (positions[i][2] - prev_positions[i][2]) * inv_dt;
    }
}

/// Apply velocity damping to slow down the cloth.
///
/// Simple per-vertex exponential damping: v *= (1 - damping).
/// `damping` should be in [0, 1] where 0 = no damping, 1 = full stop.
pub fn apply_damping(velocities: &mut [[f64; 3]], inv_masses: &[f64], damping: f64) {
    let factor = (1.0 - damping).clamp(0.0, 1.0);
    let n = velocities.len().min(inv_masses.len());

    for i in 0..n {
        if inv_masses[i] <= 0.0 {
            continue;
        }
        velocities[i][0] *= factor;
        velocities[i][1] *= factor;
        velocities[i][2] *= factor;
    }
}

/// Apply a velocity floor to prevent numerical drift.
///
/// Any velocity component with magnitude below `threshold` is set to zero.
pub fn apply_velocity_floor(velocities: &mut [[f64; 3]], threshold: f64) {
    for v in velocities.iter_mut() {
        for component in v.iter_mut() {
            if component.abs() < threshold {
                *component = 0.0;
            }
        }
    }
}

/// Compute the kinetic energy of the system.
///
/// Returns: sum of 0.5 * m * |v|^2 for all vertices with inv_mass > 0.
pub fn kinetic_energy(velocities: &[[f64; 3]], inv_masses: &[f64]) -> f64 {
    let n = velocities.len().min(inv_masses.len());
    let mut energy = 0.0;

    for i in 0..n {
        let w = inv_masses[i];
        if w <= 0.0 {
            continue;
        }
        let mass = 1.0 / w;
        let v_sq = velocities[i][0] * velocities[i][0]
            + velocities[i][1] * velocities[i][1]
            + velocities[i][2] * velocities[i][2];
        energy += 0.5 * mass * v_sq;
    }

    energy
}

/// Compute the maximum velocity magnitude in the system.
pub fn max_velocity(velocities: &[[f64; 3]]) -> f64 {
    let mut max_sq = 0.0_f64;

    for v in velocities {
        let sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
        if sq > max_sq {
            max_sq = sq;
        }
    }

    max_sq.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gravity_integration() {
        let mut velocities = vec![[0.0, 0.0, 0.0]; 3];
        let inv_masses = vec![1.0, 1.0, 0.0]; // vertex 2 is pinned
        let gravity = [0.0, -9.81, 0.0];
        let dt = 0.01;

        integrate_velocities(&mut velocities, &inv_masses, &gravity, None, dt);

        assert!((velocities[0][1] - (-0.0981)).abs() < 1e-10);
        assert!((velocities[1][1] - (-0.0981)).abs() < 1e-10);
        assert_eq!(velocities[2], [0.0, 0.0, 0.0]); // pinned
    }

    #[test]
    fn test_position_integration() {
        let mut positions = vec![[0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let velocities = vec![[0.0, -1.0, 0.0], [1.0, -1.0, 0.0]];
        let inv_masses = vec![1.0, 1.0];
        let mut prev = Vec::new();
        let dt = 0.5;

        integrate_positions(&mut positions, &velocities, &mut prev, &inv_masses, dt);

        assert!((positions[0][1] - 0.5).abs() < 1e-10);
        assert!((positions[1][0] - 1.5).abs() < 1e-10);
        assert_eq!(prev[0], [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_velocity_update() {
        let positions = vec![[0.0, 0.5, 0.0], [1.5, 0.5, 0.0]];
        let prev_positions = vec![[0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
        let mut velocities = vec![[0.0; 3]; 2];
        let inv_masses = vec![1.0, 1.0];
        let dt = 0.5;

        update_velocities_from_positions(&positions, &prev_positions, &mut velocities, &inv_masses, dt);

        assert!((velocities[0][1] - (-1.0)).abs() < 1e-10);
        assert!((velocities[1][0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_damping() {
        let mut velocities = vec![[10.0, -5.0, 3.0]];
        let inv_masses = vec![1.0];

        apply_damping(&mut velocities, &inv_masses, 0.1);

        assert!((velocities[0][0] - 9.0).abs() < 1e-10);
        assert!((velocities[0][1] - (-4.5)).abs() < 1e-10);
    }

    #[test]
    fn test_kinetic_energy() {
        let velocities = vec![[3.0, 4.0, 0.0]];
        let inv_masses = vec![0.5]; // mass = 2
        let ke = kinetic_energy(&velocities, &inv_masses);
        // 0.5 * 2 * 25 = 25
        assert!((ke - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_pinned_vertex_zero_velocity() {
        let positions = vec![[1.0, 2.0, 3.0]];
        let prev_positions = vec![[0.0, 0.0, 0.0]];
        let mut velocities = vec![[999.0, 999.0, 999.0]];
        let inv_masses = vec![0.0]; // pinned

        update_velocities_from_positions(&positions, &prev_positions, &mut velocities, &inv_masses, 1.0);
        assert_eq!(velocities[0], [0.0, 0.0, 0.0]);
    }
}
