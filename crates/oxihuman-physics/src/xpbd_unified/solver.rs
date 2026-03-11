// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! XPBD unified solver: predict → project constraints → update velocities.

use anyhow::Result;

use super::constraint::{ConstraintEntry, ConstraintId, XpbdConstraint};

// ─── Configuration ──────────────────────────────────────────────────────────

/// Solver configuration parameters.
#[derive(Debug, Clone)]
pub struct XpbdConfig {
    /// Timestep per simulation step (seconds).
    pub dt: f64,
    /// Gravitational acceleration (m/s²).
    pub gravity: [f64; 3],
    /// Number of Gauss-Seidel constraint projection iterations per step.
    pub iterations: usize,
    /// Velocity damping factor (0 = no damping, 1 = full damping).
    pub damping: f64,
    /// Number of substeps (the timestep is divided by this value).
    pub substeps: usize,
}

impl Default for XpbdConfig {
    fn default() -> Self {
        Self {
            dt: 1.0 / 60.0,
            gravity: [0.0, -9.81, 0.0],
            iterations: 10,
            damping: 0.01,
            substeps: 1,
        }
    }
}

// ─── Solver ─────────────────────────────────────────────────────────────────

/// XPBD unified solver managing particles and constraints.
///
/// The solver owns the particle state (positions, velocities, masses) and a
/// collection of trait-object constraints.  Each call to [`step`] advances the
/// simulation by the configured timestep using the XPBD algorithm.
pub struct XpbdSolver {
    /// Current particle positions.
    positions: Vec<[f64; 3]>,
    /// Positions at the start of the current step (for velocity update).
    prev_positions: Vec<[f64; 3]>,
    /// Particle velocities.
    velocities: Vec<[f64; 3]>,
    /// Inverse masses (0 ⇒ fixed / infinite mass).
    inv_masses: Vec<f64>,
    /// Constraint entries (with accumulated Lagrange multipliers).
    constraints: Vec<ConstraintEntry>,
    /// Next constraint id counter.
    next_constraint_id: u64,
    /// Solver configuration.
    config: XpbdConfig,
}

impl XpbdSolver {
    /// Create a new solver with the given configuration.
    pub fn new(config: XpbdConfig) -> Self {
        Self {
            positions: Vec::new(),
            prev_positions: Vec::new(),
            velocities: Vec::new(),
            inv_masses: Vec::new(),
            constraints: Vec::new(),
            next_constraint_id: 0,
            config,
        }
    }

    // ── Particle management ─────────────────────────────────────────────

    /// Add a particle and return its index.
    ///
    /// `mass` ≤ 0 is treated as infinite mass (fixed particle).
    pub fn add_particle(&mut self, pos: [f64; 3], mass: f64) -> usize {
        let idx = self.positions.len();
        self.positions.push(pos);
        self.prev_positions.push(pos);
        self.velocities.push([0.0; 3]);
        let inv_m = if mass > f64::EPSILON { 1.0 / mass } else { 0.0 };
        self.inv_masses.push(inv_m);
        idx
    }

    /// Number of particles.
    pub fn particle_count(&self) -> usize {
        self.positions.len()
    }

    /// Set the position of an existing particle.
    pub fn set_position(&mut self, index: usize, pos: [f64; 3]) -> Result<()> {
        let p = self
            .positions
            .get_mut(index)
            .ok_or_else(|| anyhow::anyhow!("particle index {} out of range", index))?;
        *p = pos;
        Ok(())
    }

    /// Set the velocity of an existing particle.
    pub fn set_velocity(&mut self, index: usize, vel: [f64; 3]) -> Result<()> {
        let v = self
            .velocities
            .get_mut(index)
            .ok_or_else(|| anyhow::anyhow!("particle index {} out of range", index))?;
        *v = vel;
        Ok(())
    }

    /// Set the mass of an existing particle.
    pub fn set_mass(&mut self, index: usize, mass: f64) -> Result<()> {
        let w = self
            .inv_masses
            .get_mut(index)
            .ok_or_else(|| anyhow::anyhow!("particle index {} out of range", index))?;
        *w = if mass > f64::EPSILON { 1.0 / mass } else { 0.0 };
        Ok(())
    }

    // ── Constraint management ───────────────────────────────────────────

    /// Add a constraint and return its id.
    pub fn add_constraint(&mut self, c: Box<dyn XpbdConstraint>) -> ConstraintId {
        let id = ConstraintId(self.next_constraint_id);
        self.next_constraint_id += 1;
        self.constraints.push(ConstraintEntry::new(id, c));
        id
    }

    /// Remove a constraint by id.  Returns `true` if found and removed.
    pub fn remove_constraint(&mut self, id: ConstraintId) -> bool {
        let before = self.constraints.len();
        self.constraints.retain(|e| e.id != id);
        self.constraints.len() < before
    }

    /// Number of active constraints.
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    // ── Accessors ───────────────────────────────────────────────────────

    /// Immutable view of positions.
    pub fn positions(&self) -> &[[f64; 3]] {
        &self.positions
    }

    /// Immutable view of velocities.
    pub fn velocities(&self) -> &[[f64; 3]] {
        &self.velocities
    }

    /// Immutable view of inverse masses.
    pub fn inv_masses(&self) -> &[f64] {
        &self.inv_masses
    }

    /// Mutable access to the config.
    pub fn config_mut(&mut self) -> &mut XpbdConfig {
        &mut self.config
    }

    /// Immutable view of the config.
    pub fn config(&self) -> &XpbdConfig {
        &self.config
    }

    // ── Simulation step ─────────────────────────────────────────────────

    /// Advance the simulation by one full timestep.
    ///
    /// The timestep is divided into `config.substeps` sub-steps.  Each
    /// sub-step performs:
    ///
    /// 1. Apply external forces (gravity) to velocities
    /// 2. Predict positions: x_pred = x + v * sub_dt
    /// 3. For each iteration: Gauss-Seidel project all constraints
    /// 4. Update velocities: v = (x_new - x_old) / sub_dt
    /// 5. Apply damping
    pub fn step(&mut self) -> Result<()> {
        let n = self.positions.len();
        if n == 0 {
            return Ok(());
        }

        let substeps = self.config.substeps.max(1);
        let sub_dt = self.config.dt / substeps as f64;

        for _ in 0..substeps {
            self.substep(n, sub_dt)?;
        }

        Ok(())
    }

    /// Execute one substep of the XPBD algorithm.
    fn substep(&mut self, n: usize, sub_dt: f64) -> Result<()> {
        // 1. Apply external forces → update velocities
        let gravity = self.config.gravity;
        for i in 0..n {
            if self.inv_masses[i] < f64::EPSILON {
                continue; // fixed particle
            }
            for d in 0..3 {
                self.velocities[i][d] += gravity[d] * sub_dt;
            }
        }

        // 2. Predict positions & save previous
        for i in 0..n {
            self.prev_positions[i] = self.positions[i];
            if self.inv_masses[i] < f64::EPSILON {
                continue;
            }
            for d in 0..3 {
                self.positions[i][d] += self.velocities[i][d] * sub_dt;
            }
        }

        // Reset Lagrange multipliers for this substep
        for entry in &mut self.constraints {
            entry.reset_lambda();
        }

        // 3. Gauss-Seidel constraint projection
        let iterations = self.config.iterations;
        for _ in 0..iterations {
            for entry in &mut self.constraints {
                let _violation = entry.constraint.project(
                    &mut self.positions,
                    &self.inv_masses,
                    sub_dt,
                );
            }
        }

        // 4. Update velocities from position change
        let inv_dt = if sub_dt.abs() > f64::EPSILON {
            1.0 / sub_dt
        } else {
            0.0
        };
        for i in 0..n {
            if self.inv_masses[i] < f64::EPSILON {
                self.velocities[i] = [0.0; 3];
                continue;
            }
            for d in 0..3 {
                self.velocities[i][d] =
                    (self.positions[i][d] - self.prev_positions[i][d]) * inv_dt;
            }
        }

        // 5. Apply damping
        let damping = self.config.damping.clamp(0.0, 1.0);
        if damping > f64::EPSILON {
            let factor = 1.0 - damping;
            for i in 0..n {
                for d in 0..3 {
                    self.velocities[i][d] *= factor;
                }
            }
        }

        Ok(())
    }

    /// Run multiple steps.
    pub fn step_n(&mut self, count: usize) -> Result<()> {
        for _ in 0..count {
            self.step()?;
        }
        Ok(())
    }

    /// Compute total kinetic energy (0.5 * m * v²) of the system.
    pub fn kinetic_energy(&self) -> f64 {
        let mut ke = 0.0;
        for i in 0..self.positions.len() {
            if self.inv_masses[i] < f64::EPSILON {
                continue;
            }
            let m = 1.0 / self.inv_masses[i];
            let v2 = self.velocities[i][0] * self.velocities[i][0]
                + self.velocities[i][1] * self.velocities[i][1]
                + self.velocities[i][2] * self.velocities[i][2];
            ke += 0.5 * m * v2;
        }
        ke
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xpbd_unified::builtin_constraints::DistanceConstraint;

    #[test]
    fn test_empty_solver_step() {
        let mut solver = XpbdSolver::new(XpbdConfig::default());
        assert!(solver.step().is_ok());
    }

    #[test]
    fn test_particle_falls_under_gravity() {
        let mut solver = XpbdSolver::new(XpbdConfig {
            dt: 0.01,
            gravity: [0.0, -9.81, 0.0],
            iterations: 5,
            damping: 0.0,
            substeps: 1,
        });
        solver.add_particle([0.0, 10.0, 0.0], 1.0);
        solver.step().unwrap();
        // Should have moved downward
        assert!(solver.positions()[0][1] < 10.0);
    }

    #[test]
    fn test_fixed_particle_does_not_move() {
        let mut solver = XpbdSolver::new(XpbdConfig::default());
        solver.add_particle([0.0, 5.0, 0.0], 0.0); // fixed
        solver.step().unwrap();
        assert!((solver.positions()[0][1] - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distance_constraint_integration() {
        let mut solver = XpbdSolver::new(XpbdConfig {
            dt: 0.01,
            gravity: [0.0, 0.0, 0.0],
            iterations: 20,
            damping: 0.0,
            substeps: 1,
        });
        solver.add_particle([0.0, 0.0, 0.0], 1.0);
        solver.add_particle([3.0, 0.0, 0.0], 1.0);
        let dc = DistanceConstraint::new(0, 1, 1.0, 0.0);
        solver.add_constraint(Box::new(dc));
        solver.step().unwrap();
        let dx = solver.positions()[1][0] - solver.positions()[0][0];
        let dy = solver.positions()[1][1] - solver.positions()[0][1];
        let dz = solver.positions()[1][2] - solver.positions()[0][2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        // Should be closer to rest length 1.0 than original 3.0
        assert!(dist < 2.5);
    }

    #[test]
    fn test_add_remove_constraint() {
        let mut solver = XpbdSolver::new(XpbdConfig::default());
        solver.add_particle([0.0, 0.0, 0.0], 1.0);
        solver.add_particle([1.0, 0.0, 0.0], 1.0);
        let dc = DistanceConstraint::new(0, 1, 1.0, 0.0);
        let id = solver.add_constraint(Box::new(dc));
        assert_eq!(solver.constraint_count(), 1);
        assert!(solver.remove_constraint(id));
        assert_eq!(solver.constraint_count(), 0);
        // Removing again should return false
        assert!(!solver.remove_constraint(id));
    }

    #[test]
    fn test_kinetic_energy() {
        let mut solver = XpbdSolver::new(XpbdConfig::default());
        let idx = solver.add_particle([0.0, 0.0, 0.0], 2.0);
        solver.set_velocity(idx, [1.0, 0.0, 0.0]).unwrap();
        // KE = 0.5 * 2.0 * 1.0^2 = 1.0
        assert!((solver.kinetic_energy() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_substeps() {
        let mut solver = XpbdSolver::new(XpbdConfig {
            dt: 0.01,
            gravity: [0.0, -9.81, 0.0],
            iterations: 5,
            damping: 0.0,
            substeps: 4,
        });
        solver.add_particle([0.0, 10.0, 0.0], 1.0);
        solver.step().unwrap();
        assert!(solver.positions()[0][1] < 10.0);
    }

    #[test]
    fn test_step_n() {
        let mut solver = XpbdSolver::new(XpbdConfig {
            dt: 0.01,
            gravity: [0.0, -9.81, 0.0],
            iterations: 5,
            damping: 0.0,
            substeps: 1,
        });
        solver.add_particle([0.0, 100.0, 0.0], 1.0);
        solver.step_n(100).unwrap();
        // Should have fallen significantly
        assert!(solver.positions()[0][1] < 100.0);
    }

    #[test]
    fn test_set_position() {
        let mut solver = XpbdSolver::new(XpbdConfig::default());
        solver.add_particle([0.0, 0.0, 0.0], 1.0);
        assert!(solver.set_position(0, [5.0, 5.0, 5.0]).is_ok());
        assert!((solver.positions()[0][0] - 5.0).abs() < f64::EPSILON);
        assert!(solver.set_position(99, [0.0, 0.0, 0.0]).is_err());
    }

    #[test]
    fn test_damping_reduces_velocity() {
        let mut solver = XpbdSolver::new(XpbdConfig {
            dt: 0.01,
            gravity: [0.0, 0.0, 0.0],
            iterations: 0,
            damping: 0.5,
            substeps: 1,
        });
        let idx = solver.add_particle([0.0, 0.0, 0.0], 1.0);
        solver.set_velocity(idx, [10.0, 0.0, 0.0]).unwrap();
        solver.step().unwrap();
        // Velocity should be damped
        assert!(solver.velocities()[0][0].abs() < 10.0);
    }
}
