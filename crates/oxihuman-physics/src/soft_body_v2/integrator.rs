// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Implicit Euler integration with conjugate gradient solver.
//!
//! The implicit Euler scheme solves:
//!   v_{n+1} = v_n + dt * M^{-1} * f(x_n + dt * v_{n+1})
//!   x_{n+1} = x_n + dt * v_{n+1}
//!
//! For the linearized system we solve:
//!   (M - dt^2 * K) * dv = dt * f(x_n) + dt^2 * K * v_n
//!
//! using the conjugate gradient method.

use anyhow::Result;

use super::Vec3;

/// Result of an integration step.
#[derive(Debug, Clone)]
pub struct IntegrationResult {
    /// Number of CG iterations used.
    pub cg_iterations: usize,
    /// Final residual norm of the CG solver.
    pub residual_norm: f64,
    /// Whether the CG solver converged.
    pub converged: bool,
}

/// Implicit Euler integrator using conjugate gradient.
#[derive(Debug)]
pub struct ImplicitEulerIntegrator {
    /// Time step.
    dt: f64,
    /// Maximum CG iterations.
    max_iterations: usize,
    /// CG convergence tolerance.
    cg_tolerance: f64,
}

impl ImplicitEulerIntegrator {
    /// Create a new integrator.
    pub fn new(dt: f64, max_iterations: usize) -> Self {
        Self {
            dt,
            max_iterations,
            cg_tolerance: 1e-6,
        }
    }

    /// Set the CG convergence tolerance.
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.cg_tolerance = tol;
        self
    }

    /// Perform a semi-implicit Euler integration step.
    ///
    /// This is a simplified implicit Euler that uses the elastic forces
    /// computed at the current configuration and applies damping.
    ///
    /// For full implicit Euler, `integrate_implicit` should be used with
    /// stiffness matrix information.
    pub fn integrate(
        &mut self,
        positions: &mut [Vec3],
        velocities: &mut [Vec3],
        inv_masses: &[f64],
        forces: &[Vec3],
        gravity: &Vec3,
        damping: f64,
    ) -> Result<IntegrationResult> {
        let n = positions.len();

        for i in 0..n {
            let inv_m = inv_masses[i];
            if inv_m <= 0.0 {
                // Fixed node
                velocities[i] = [0.0; 3];
                continue;
            }

            // Apply gravity and elastic forces
            for d in 0..3 {
                let accel = (forces[i][d] + gravity[d] / inv_m) * inv_m;
                velocities[i][d] += accel * self.dt;
            }

            // Apply damping
            for vel_comp in velocities[i].iter_mut() {
                *vel_comp *= damping;
            }

            // Update positions
            for d in 0..3 {
                positions[i][d] += velocities[i][d] * self.dt;
            }
        }

        Ok(IntegrationResult {
            cg_iterations: 0,
            residual_norm: 0.0,
            converged: true,
        })
    }

    /// Perform fully implicit Euler integration using conjugate gradient.
    ///
    /// Solves (M - dt^2 * K) * dv = dt * f + dt^2 * K * v
    /// where K is given in sparse triplet format (rows, cols, values).
    ///
    /// Parameters:
    /// - `positions`: current positions (updated in place)
    /// - `velocities`: current velocities (updated in place)
    /// - `inv_masses`: inverse masses per node
    /// - `forces`: elastic + external forces at current configuration
    /// - `gravity`: gravity vector
    /// - `damping`: velocity damping factor
    /// - `stiffness_rows`, `stiffness_cols`, `stiffness_values`: sparse K matrix
    #[allow(clippy::too_many_arguments)]
    pub fn integrate_implicit(
        &mut self,
        positions: &mut [Vec3],
        velocities: &mut [Vec3],
        inv_masses: &[f64],
        forces: &[Vec3],
        gravity: &Vec3,
        damping: f64,
        stiffness_rows: &[usize],
        stiffness_cols: &[usize],
        stiffness_values: &[f64],
    ) -> Result<IntegrationResult> {
        let n = positions.len();
        let dof = n * 3;
        let dt = self.dt;
        let dt2 = dt * dt;

        // Build the right-hand side: b = dt * f + dt^2 * K * v
        let mut rhs = vec![0.0_f64; dof];
        let mut v_flat = vec![0.0_f64; dof];

        for i in 0..n {
            let inv_m = inv_masses[i];
            for d in 0..3 {
                let idx = i * 3 + d;
                v_flat[idx] = velocities[i][d];

                // Total force = elastic + gravity (gravity = g / inv_m * inv_m = g for mass=1/inv_m)
                let grav_force = if inv_m > 0.0 { gravity[d] / inv_m } else { 0.0 };
                rhs[idx] = dt * (forces[i][d] + grav_force);
            }
        }

        // Add dt^2 * K * v to rhs
        let kv = sparse_matvec_flat(
            stiffness_rows,
            stiffness_cols,
            stiffness_values,
            &v_flat,
            dof,
        );
        for i in 0..dof {
            rhs[i] += dt2 * kv[i];
        }

        // Build the effective mass matrix diagonal: M_ii = 1/inv_mass_i
        // The system matrix is A = M - dt^2 * K
        // We solve A * dv = rhs using CG

        // For the CG solver, we need A*x operation:
        // A*x = M*x - dt^2 * K * x
        let masses: Vec<f64> = inv_masses
            .iter()
            .flat_map(|&inv_m| {
                let m = if inv_m > 0.0 { 1.0 / inv_m } else { 1e10 };
                [m, m, m]
            })
            .collect();

        // Initial guess: dv = 0
        let mut dv = vec![0.0_f64; dof];

        let result = conjugate_gradient(
            |x| {
                // A*x = M*x - dt^2 * K*x
                let kx =
                    sparse_matvec_flat(stiffness_rows, stiffness_cols, stiffness_values, x, dof);
                let mut ax = vec![0.0; dof];
                for i in 0..dof {
                    ax[i] = masses[i] * x[i] - dt2 * kx[i];
                }
                ax
            },
            &rhs,
            &mut dv,
            self.max_iterations,
            self.cg_tolerance,
            &masses,
        );

        // Update velocities and positions
        for i in 0..n {
            let inv_m = inv_masses[i];
            if inv_m <= 0.0 {
                velocities[i] = [0.0; 3];
                continue;
            }

            for d in 0..3 {
                velocities[i][d] += dv[i * 3 + d];
                velocities[i][d] *= damping;
                positions[i][d] += velocities[i][d] * dt;
            }
        }

        Ok(result)
    }
}

/// Conjugate gradient solver for Ax = b.
///
/// Uses diagonal preconditioning with the mass matrix diagonal.
///
/// Parameters:
/// - `matvec`: function computing A*x
/// - `b`: right-hand side
/// - `x`: initial guess (modified in place with the solution)
/// - `max_iter`: maximum iterations
/// - `tol`: convergence tolerance (relative to |b|)
/// - `precond_diag`: diagonal preconditioner values
fn conjugate_gradient<F>(
    matvec: F,
    b: &[f64],
    x: &mut [f64],
    max_iter: usize,
    tol: f64,
    precond_diag: &[f64],
) -> IntegrationResult
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    let n = b.len();

    // r = b - A*x
    let ax = matvec(x);
    let mut r: Vec<f64> = b
        .iter()
        .zip(ax.iter())
        .map(|(&bi, &axi)| bi - axi)
        .collect();

    // Preconditioned residual: z = M^{-1} * r
    let mut z = precondition(&r, precond_diag);

    // p = z
    let mut p = z.clone();

    let mut rz = dot_flat(&r, &z);
    let b_norm = dot_flat(b, b).sqrt().max(1e-30);

    let mut iterations = 0;
    let mut residual_norm = dot_flat(&r, &r).sqrt() / b_norm;

    for iter in 0..max_iter {
        iterations = iter + 1;

        let ap = matvec(&p);
        let pap = dot_flat(&p, &ap);

        if pap.abs() < 1e-30 {
            break;
        }

        let alpha = rz / pap;

        // x = x + alpha * p
        for i in 0..n {
            x[i] += alpha * p[i];
        }

        // r = r - alpha * A*p
        for i in 0..n {
            r[i] -= alpha * ap[i];
        }

        residual_norm = dot_flat(&r, &r).sqrt() / b_norm;
        if residual_norm < tol {
            break;
        }

        // z = M^{-1} * r
        z = precondition(&r, precond_diag);

        let rz_new = dot_flat(&r, &z);
        if rz.abs() < 1e-30 {
            break;
        }
        let beta = rz_new / rz;

        // p = z + beta * p
        for i in 0..n {
            p[i] = z[i] + beta * p[i];
        }

        rz = rz_new;
    }

    IntegrationResult {
        cg_iterations: iterations,
        residual_norm,
        converged: residual_norm < tol,
    }
}

/// Apply diagonal preconditioning: z_i = r_i / M_ii.
fn precondition(r: &[f64], diag: &[f64]) -> Vec<f64> {
    r.iter()
        .zip(diag.iter())
        .map(|(&ri, &di)| if di.abs() > 1e-30 { ri / di } else { ri })
        .collect()
}

/// Dot product of two flat vectors.
fn dot_flat(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(&ai, &bi)| ai * bi).sum()
}

/// Sparse matrix-vector multiply (flat vector version).
fn sparse_matvec_flat(
    rows: &[usize],
    cols: &[usize],
    values: &[f64],
    x: &[f64],
    n: usize,
) -> Vec<f64> {
    let mut y = vec![0.0; n];
    for ((&r, &c), &v) in rows.iter().zip(cols.iter()).zip(values.iter()) {
        if r < n && c < x.len() {
            y[r] += v * x[c];
        }
    }
    y
}

/// Simple explicit Euler integrator for comparison/testing.
pub fn explicit_euler_step(
    positions: &mut [Vec3],
    velocities: &mut [Vec3],
    inv_masses: &[f64],
    forces: &[Vec3],
    gravity: &Vec3,
    dt: f64,
    damping: f64,
) {
    for i in 0..positions.len() {
        let inv_m = inv_masses[i];
        if inv_m <= 0.0 {
            continue;
        }

        for d in 0..3 {
            let grav_force = gravity[d] / inv_m;
            let accel = (forces[i][d] + grav_force) * inv_m;
            velocities[i][d] += accel * dt;
            velocities[i][d] *= damping;
            positions[i][d] += velocities[i][d] * dt;
        }
    }
}

/// Compute kinetic energy of the system.
pub fn kinetic_energy(velocities: &[Vec3], inv_masses: &[f64]) -> f64 {
    let mut ke = 0.0;
    for (v, &inv_m) in velocities.iter().zip(inv_masses.iter()) {
        if inv_m > 0.0 {
            let m = 1.0 / inv_m;
            let v_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
            ke += 0.5 * m * v_sq;
        }
    }
    ke
}

/// Compute gravitational potential energy.
pub fn gravitational_potential_energy(
    positions: &[Vec3],
    inv_masses: &[f64],
    gravity: &Vec3,
) -> f64 {
    let mut pe = 0.0;
    for (p, &inv_m) in positions.iter().zip(inv_masses.iter()) {
        if inv_m > 0.0 {
            let m = 1.0 / inv_m;
            // PE = -m * g . x
            pe -= m * (gravity[0] * p[0] + gravity[1] * p[1] + gravity[2] * p[2]);
        }
    }
    pe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conjugate_gradient_identity() {
        // Solve I * x = b
        let b = vec![1.0, 2.0, 3.0];
        let mut x = vec![0.0; 3];
        let diag = vec![1.0; 3];

        let result = conjugate_gradient(|v| v.to_vec(), &b, &mut x, 100, 1e-10, &diag);

        assert!(result.converged);
        for i in 0..3 {
            assert!(
                (x[i] - b[i]).abs() < 1e-8,
                "x[{i}] = {} expected {}",
                x[i],
                b[i]
            );
        }
    }

    #[test]
    fn test_conjugate_gradient_diagonal() {
        // Solve diag(2,3,4) * x = (6, 12, 20)
        let b = vec![6.0, 12.0, 20.0];
        let diag_vals = [2.0, 3.0, 4.0];
        let mut x = vec![0.0; 3];
        let precond = vec![1.0; 3];

        let result = conjugate_gradient(
            |v| {
                vec![
                    diag_vals[0] * v[0],
                    diag_vals[1] * v[1],
                    diag_vals[2] * v[2],
                ]
            },
            &b,
            &mut x,
            100,
            1e-10,
            &precond,
        );

        assert!(result.converged);
        assert!((x[0] - 3.0).abs() < 1e-8);
        assert!((x[1] - 4.0).abs() < 1e-8);
        assert!((x[2] - 5.0).abs() < 1e-8);
    }

    #[test]
    fn test_conjugate_gradient_spd_matrix() {
        // Solve a 3x3 SPD system
        // A = [[4, 1, 0], [1, 3, 1], [0, 1, 2]]
        let matvec = |v: &[f64]| -> Vec<f64> {
            vec![
                4.0 * v[0] + 1.0 * v[1],
                1.0 * v[0] + 3.0 * v[1] + 1.0 * v[2],
                1.0 * v[1] + 2.0 * v[2],
            ]
        };
        let b = vec![5.0, 5.0, 3.0];
        let mut x = vec![0.0; 3];
        let precond = vec![4.0, 3.0, 2.0]; // diagonal of A

        let result = conjugate_gradient(matvec, &b, &mut x, 100, 1e-10, &precond);

        assert!(
            result.converged,
            "CG did not converge, residual={}",
            result.residual_norm
        );

        // Verify A*x = b
        let ax = [
            4.0 * x[0] + 1.0 * x[1],
            1.0 * x[0] + 3.0 * x[1] + 1.0 * x[2],
            1.0 * x[1] + 2.0 * x[2],
        ];
        for i in 0..3 {
            assert!(
                (ax[i] - b[i]).abs() < 1e-6,
                "Ax[{i}]={} vs b[{i}]={}",
                ax[i],
                b[i]
            );
        }
    }

    #[test]
    fn test_integrate_fixed_node() {
        let mut positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mut velocities = vec![[0.0; 3]; 2];
        let inv_masses = vec![0.0, 1.0]; // node 0 is fixed
        let forces = vec![[0.0; 3]; 2];
        let gravity = [0.0, -9.81, 0.0];

        let mut integrator = ImplicitEulerIntegrator::new(1.0 / 60.0, 10);
        integrator
            .integrate(
                &mut positions,
                &mut velocities,
                &inv_masses,
                &forces,
                &gravity,
                0.99,
            )
            .unwrap();

        // Node 0 should not move
        assert!((positions[0][0]).abs() < 1e-15);
        assert!((positions[0][1]).abs() < 1e-15);
        assert!((positions[0][2]).abs() < 1e-15);

        // Node 1 should have moved down due to gravity
        assert!(positions[1][1] < 0.0, "Node 1 should move down");
    }

    #[test]
    fn test_kinetic_energy() {
        let velocities = vec![[1.0, 0.0, 0.0], [0.0, 2.0, 0.0]];
        let inv_masses = vec![1.0, 0.5]; // masses 1 and 2

        let ke = kinetic_energy(&velocities, &inv_masses);
        // KE = 0.5 * 1 * 1 + 0.5 * 2 * 4 = 0.5 + 4.0 = 4.5
        assert!((ke - 4.5).abs() < 1e-12, "KE = {ke}");
    }

    #[test]
    fn test_explicit_euler_step() {
        let mut positions = vec![[0.0, 10.0, 0.0]];
        let mut velocities = vec![[0.0; 3]];
        let inv_masses = vec![1.0];
        let forces = vec![[0.0; 3]];
        let gravity = [0.0, -9.81, 0.0];

        explicit_euler_step(
            &mut positions,
            &mut velocities,
            &inv_masses,
            &forces,
            &gravity,
            1.0 / 60.0,
            1.0,
        );

        // Should have fallen slightly
        assert!(positions[0][1] < 10.0);
        assert!(velocities[0][1] < 0.0);
    }

    #[test]
    fn test_gravitational_potential_energy() {
        let positions = vec![[0.0, 10.0, 0.0]];
        let inv_masses = vec![1.0]; // mass = 1
        let gravity = [0.0, -9.81, 0.0];

        let pe = gravitational_potential_energy(&positions, &inv_masses, &gravity);
        // PE = -m * g . x = -1 * (-9.81 * 10) = 98.1
        assert!((pe - 98.1).abs() < 1e-10, "PE = {pe}");
    }

    #[test]
    fn test_sparse_matvec_flat() {
        let rows = vec![0, 0, 1, 1];
        let cols = vec![0, 1, 0, 1];
        let values = vec![2.0, 1.0, 1.0, 3.0];
        let x = vec![1.0, 2.0];
        let y = sparse_matvec_flat(&rows, &cols, &values, &x, 2);
        assert!((y[0] - 4.0).abs() < 1e-12); // 2*1 + 1*2
        assert!((y[1] - 7.0).abs() < 1e-12); // 1*1 + 3*2
    }

    #[test]
    fn test_dot_flat() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert!((dot_flat(&a, &b) - 32.0).abs() < 1e-12);
    }

    #[test]
    fn test_integrate_implicit_basic() {
        let mut positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mut velocities = vec![[0.0; 3]; 2];
        let inv_masses = vec![0.0, 1.0]; // node 0 fixed
        let forces = vec![[0.0; 3]; 2];
        let gravity = [0.0, -9.81, 0.0];

        // Empty stiffness matrix (no elastic coupling)
        let rows: Vec<usize> = vec![];
        let cols: Vec<usize> = vec![];
        let values: Vec<f64> = vec![];

        let mut integrator = ImplicitEulerIntegrator::new(1.0 / 60.0, 10);
        let result = integrator
            .integrate_implicit(
                &mut positions,
                &mut velocities,
                &inv_masses,
                &forces,
                &gravity,
                0.99,
                &rows,
                &cols,
                &values,
            )
            .unwrap();

        // Should converge immediately with empty stiffness
        assert!(result.converged || result.cg_iterations <= 1);

        // Node 1 should fall
        assert!(positions[1][1] < 0.0);
    }
}
