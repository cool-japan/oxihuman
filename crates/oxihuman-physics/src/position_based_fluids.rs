// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Position-Based Fluids (PBF) simulation.

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for the PBF solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbfConfig {
    pub rest_density: f32,
    pub solver_iterations: u32,
    pub particle_radius: f32,
    pub gravity: [f32; 3],
    pub epsilon_relaxation: f32,
}

/// A single PBF fluid particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbfParticle {
    pub position: [f32; 3],
    pub predicted: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub lambda: f32,
}

/// Container for all PBF particles and their configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbfSystem {
    pub particles: Vec<PbfParticle>,
    pub config: PbfConfig,
    pub time: f32,
}

/// Per-step diagnostic result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PbfResult {
    pub avg_density: f32,
    pub max_density_error: f32,
    pub particle_count: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a sensible default `PbfConfig`.
#[allow(dead_code)]
pub fn default_pbf_config() -> PbfConfig {
    PbfConfig {
        rest_density: 1000.0,
        solver_iterations: 3,
        particle_radius: 0.05,
        gravity: [0.0, -9.81, 0.0],
        epsilon_relaxation: 100.0,
    }
}

/// Construct a `PbfParticle` at `pos` with the given `mass`.
#[allow(dead_code)]
pub fn new_pbf_particle(pos: [f32; 3], mass: f32) -> PbfParticle {
    PbfParticle {
        position: pos,
        predicted: pos,
        velocity: [0.0; 3],
        mass,
        lambda: 0.0,
    }
}

/// Construct a new, empty `PbfSystem` with the given configuration.
#[allow(dead_code)]
pub fn new_pbf_system(cfg: PbfConfig) -> PbfSystem {
    PbfSystem {
        particles: Vec::new(),
        config: cfg,
        time: 0.0,
    }
}

/// Append a particle to the system.
#[allow(dead_code)]
pub fn add_pbf_particle(sys: &mut PbfSystem, p: PbfParticle) {
    sys.particles.push(p);
}

/// Advance the simulation by one timestep `dt`.
///
/// This is a simplified PBF implementation:
/// 1. Apply gravity to velocities and predict positions.
/// 2. Run `solver_iterations` density-correction passes.
/// 3. Update velocities from position deltas.
#[allow(dead_code)]
pub fn step_pbf(sys: &mut PbfSystem, dt: f32) -> PbfResult {
    let dt = dt.max(1e-6);
    let g = sys.config.gravity;
    let h = sys.config.particle_radius * 4.0; // smoothing length
    let rho0 = sys.config.rest_density;
    let eps = sys.config.epsilon_relaxation;
    let iters = sys.config.solver_iterations;

    // 1. Apply gravity and predict positions.
    for p in &mut sys.particles {
        p.velocity[0] += g[0] * dt;
        p.velocity[1] += g[1] * dt;
        p.velocity[2] += g[2] * dt;
        p.predicted[0] = p.position[0] + p.velocity[0] * dt;
        p.predicted[1] = p.position[1] + p.velocity[1] * dt;
        p.predicted[2] = p.position[2] + p.velocity[2] * dt;
    }

    // 2. Solver iterations: compute lambda and apply position correction.
    for _ in 0..iters {
        // Compute density and lambda for each particle.
        let n = sys.particles.len();
        let mut densities = vec![0.0_f32; n];
        for (i, density) in densities.iter_mut().enumerate() {
            let pi = sys.particles[i].predicted;
            let mut rho = 0.0_f32;
            for j in 0..n {
                let pj = sys.particles[j].predicted;
                let r = dist3(pi, pj);
                rho += sys.particles[j].mass * pbf_poly6_kernel(r, h);
            }
            *density = rho;
        }

        // Compute lambdas (simplified: no gradient term).
        for (i, p) in sys.particles.iter_mut().enumerate() {
            let ci = densities[i] / rho0 - 1.0;
            p.lambda = -ci / (1.0 / rho0 + eps);
        }

        // Apply position correction (simplified).
        let lambdas: Vec<f32> = sys.particles.iter().map(|p| p.lambda).collect();
        let preds: Vec<[f32; 3]> = sys.particles.iter().map(|p| p.predicted).collect();
        let masses: Vec<f32> = sys.particles.iter().map(|p| p.mass).collect();

        for i in 0..n {
            let pi = preds[i];
            let mut dx = [0.0_f32; 3];
            for j in 0..n {
                let pj = preds[j];
                let r = dist3(pi, pj);
                if r < h && r > 1e-6 {
                    let w = pbf_poly6_kernel(r, h);
                    let scale = (lambdas[i] + lambdas[j]) * masses[j] * w / rho0;
                    let inv_r = 1.0 / r;
                    dx[0] += scale * (pi[0] - pj[0]) * inv_r;
                    dx[1] += scale * (pi[1] - pj[1]) * inv_r;
                    dx[2] += scale * (pi[2] - pj[2]) * inv_r;
                }
            }
            sys.particles[i].predicted[0] += dx[0];
            sys.particles[i].predicted[1] += dx[1];
            sys.particles[i].predicted[2] += dx[2];
        }
    }

    // 3. Update velocity and position from corrected predicted positions.
    let inv_dt = 1.0 / dt;
    for p in &mut sys.particles {
        p.velocity[0] = (p.predicted[0] - p.position[0]) * inv_dt;
        p.velocity[1] = (p.predicted[1] - p.position[1]) * inv_dt;
        p.velocity[2] = (p.predicted[2] - p.position[2]) * inv_dt;
        p.position = p.predicted;
    }
    sys.time += dt;

    // Compute diagnostics.
    let n = sys.particles.len();
    if n == 0 {
        return PbfResult {
            avg_density: 0.0,
            max_density_error: 0.0,
            particle_count: 0,
        };
    }

    let h2 = h;
    let mut total_rho = 0.0_f32;
    let mut max_err = 0.0_f32;
    for i in 0..n {
        let pi = sys.particles[i].position;
        let mut rho = 0.0_f32;
        for j in 0..n {
            let pj = sys.particles[j].position;
            let r = dist3(pi, pj);
            rho += sys.particles[j].mass * pbf_poly6_kernel(r, h2);
        }
        let err = (rho - rho0).abs();
        if err > max_err {
            max_err = err;
        }
        total_rho += rho;
    }

    PbfResult {
        avg_density: total_rho / n as f32,
        max_density_error: max_err,
        particle_count: n,
    }
}

/// Poly6 smoothing kernel.
#[allow(dead_code)]
pub fn pbf_poly6_kernel(r: f32, h: f32) -> f32 {
    if r >= h || r < 0.0 {
        return 0.0;
    }
    let h2 = h * h;
    let r2 = r * r;
    let diff = h2 - r2;
    // 315 / (64 π h^9) * (h²-r²)³  — normalisation constant dropped for brevity
    let norm = 315.0 / (64.0 * std::f32::consts::PI * h.powi(9));
    norm * diff.powi(3)
}

/// Return the number of particles in the system.
#[allow(dead_code)]
pub fn pbf_particle_count(sys: &PbfSystem) -> usize {
    sys.particles.len()
}

/// Serialize the system to a compact JSON string.
#[allow(dead_code)]
pub fn pbf_system_to_json(sys: &PbfSystem) -> String {
    format!(
        "{{\"particle_count\":{},\"time\":{},\"rest_density\":{}}}",
        sys.particles.len(),
        sys.time,
        sys.config.rest_density
    )
}

/// Serialize a `PbfResult` to a JSON string.
#[allow(dead_code)]
pub fn pbf_result_to_json(r: &PbfResult) -> String {
    format!(
        "{{\"avg_density\":{},\"max_density_error\":{},\"particle_count\":{}}}",
        r.avg_density, r.max_density_error, r.particle_count
    )
}

/// Compute the total kinetic energy of all particles (½ m v²).
#[allow(dead_code)]
pub fn pbf_kinetic_energy(sys: &PbfSystem) -> f32 {
    sys.particles
        .iter()
        .map(|p| {
            let v2 = p.velocity[0] * p.velocity[0]
                + p.velocity[1] * p.velocity[1]
                + p.velocity[2] * p.velocity[2];
            0.5 * p.mass * v2
        })
        .sum()
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_sane() {
        let cfg = default_pbf_config();
        assert!(cfg.rest_density > 0.0);
        assert!(cfg.solver_iterations > 0);
        assert!(cfg.particle_radius > 0.0);
    }

    #[test]
    fn new_particle_at_origin() {
        let p = new_pbf_particle([0.0, 1.0, 2.0], 1.0);
        assert_eq!(p.position, [0.0, 1.0, 2.0]);
        assert_eq!(p.mass, 1.0);
        assert_eq!(p.lambda, 0.0);
    }

    #[test]
    fn add_particle_increments_count() {
        let mut sys = new_pbf_system(default_pbf_config());
        add_pbf_particle(&mut sys, new_pbf_particle([0.0, 1.0, 0.0], 1.0));
        assert_eq!(pbf_particle_count(&sys), 1);
    }

    #[test]
    fn poly6_zero_outside_h() {
        assert_eq!(pbf_poly6_kernel(2.0, 1.0), 0.0);
    }

    #[test]
    fn poly6_positive_inside_h() {
        assert!(pbf_poly6_kernel(0.0, 1.0) > 0.0);
    }

    #[test]
    fn step_moves_particle_under_gravity() {
        let cfg = default_pbf_config();
        let mut sys = new_pbf_system(cfg);
        add_pbf_particle(&mut sys, new_pbf_particle([0.0, 10.0, 0.0], 1.0));
        let before_y = sys.particles[0].position[1];
        step_pbf(&mut sys, 0.016);
        let after_y = sys.particles[0].position[1];
        assert!(after_y < before_y, "gravity should pull particle down");
    }

    #[test]
    fn empty_system_step_returns_zero_count() {
        let mut sys = new_pbf_system(default_pbf_config());
        let r = step_pbf(&mut sys, 0.016);
        assert_eq!(r.particle_count, 0);
        assert_eq!(r.avg_density, 0.0);
    }

    #[test]
    fn kinetic_energy_at_rest_is_zero() {
        let sys = new_pbf_system(default_pbf_config());
        assert_eq!(pbf_kinetic_energy(&sys), 0.0);
    }

    #[test]
    fn json_contains_particle_count() {
        let mut sys = new_pbf_system(default_pbf_config());
        add_pbf_particle(&mut sys, new_pbf_particle([0.0, 0.0, 0.0], 1.0));
        let j = pbf_system_to_json(&sys);
        assert!(j.contains("\"particle_count\":1"));
    }

    #[test]
    fn result_json_fields_present() {
        let r = PbfResult {
            avg_density: 1000.0,
            max_density_error: 5.0,
            particle_count: 10,
        };
        let j = pbf_result_to_json(&r);
        assert!(j.contains("\"avg_density\":"));
        assert!(j.contains("\"particle_count\":10"));
    }
}
