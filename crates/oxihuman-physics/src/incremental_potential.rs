// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Incremental potential contact (IPC) stub.

/// A particle for IPC.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IpcParticle {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub inv_mass: f32,
    pub pinned: bool,
}

impl IpcParticle {
    #[allow(dead_code)]
    pub fn new(pos: [f32; 3], inv_mass: f32) -> Self {
        Self {
            pos,
            vel: [0.0; 3],
            inv_mass,
            pinned: false,
        }
    }
}

/// IPC configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IpcConfig {
    /// Barrier offset parameter (hat d).
    pub dhat: f32,
    /// Barrier stiffness κ.
    pub kappa: f32,
    /// Newton iterations.
    pub newton_iters: usize,
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            dhat: 1e-3,
            kappa: 1e5,
            newton_iters: 10,
        }
    }
}

/// IPC barrier energy for a pair of particles.
/// Activates when distance d < dhat.
#[allow(dead_code)]
pub fn ipc_barrier_energy(d_sq: f32, dhat: f32, kappa: f32) -> f32 {
    let d = d_sq.sqrt();
    if d >= dhat {
        return 0.0;
    }
    let s = d / dhat;
    let log_s = if s > 1e-10 { s.ln() } else { -23.0 };
    kappa * (-log_s + s - 1.0)
}

/// IPC barrier gradient (w.r.t. distance).
#[allow(dead_code)]
pub fn ipc_barrier_gradient(d_sq: f32, dhat: f32, kappa: f32) -> f32 {
    let d = d_sq.sqrt();
    if d < 1e-10 || d >= dhat {
        return 0.0;
    }
    let s = d / dhat;
    kappa * (-1.0 / s + 1.0) / dhat
}

/// Squared distance between two particles.
#[allow(dead_code)]
pub fn ipc_dist_sq(a: &IpcParticle, b: &IpcParticle) -> f32 {
    (0..3).map(|i| (a.pos[i] - b.pos[i]).powi(2)).sum()
}

/// Compute total IPC contact energy for all pairs.
#[allow(dead_code)]
pub fn ipc_total_contact_energy(particles: &[IpcParticle], config: &IpcConfig) -> f32 {
    let mut energy = 0.0;
    for i in 0..particles.len() {
        for j in (i + 1)..particles.len() {
            let d_sq = ipc_dist_sq(&particles[i], &particles[j]);
            energy += ipc_barrier_energy(d_sq, config.dhat, config.kappa);
        }
    }
    energy
}

/// Apply a simple gradient step to separate overlapping particles.
#[allow(dead_code)]
pub fn ipc_gradient_step(particles: &mut [IpcParticle], config: &IpcConfig, step: f32) {
    let n = particles.len();
    let mut forces = vec![[0.0f32; 3]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let d_sq = ipc_dist_sq(&particles[i], &particles[j]);
            let grad = ipc_barrier_gradient(d_sq, config.dhat, config.kappa);
            if grad.abs() < 1e-12 {
                continue;
            }
            let d = d_sq.sqrt().max(1e-10);
            let dir = [
                (particles[i].pos[0] - particles[j].pos[0]) / d,
                (particles[i].pos[1] - particles[j].pos[1]) / d,
                (particles[i].pos[2] - particles[j].pos[2]) / d,
            ];
            let (left, right) = forces.split_at_mut(j);
            for (fi, (fj, dk)) in left[i].iter_mut().zip(right[0].iter_mut().zip(dir.iter())) {
                *fi += grad * dk;
                *fj -= grad * dk;
            }
        }
    }
    for (i, p) in particles.iter_mut().enumerate() {
        if p.pinned {
            continue;
        }
        for (pos_k, &force_k) in p.pos.iter_mut().zip(forces[i].iter()) {
            *pos_k -= step * p.inv_mass * force_k;
        }
    }
}

/// Number of active contact pairs (d < dhat).
#[allow(dead_code)]
pub fn ipc_active_pair_count(particles: &[IpcParticle], config: &IpcConfig) -> usize {
    let mut count = 0;
    for i in 0..particles.len() {
        for j in (i + 1)..particles.len() {
            let d_sq = ipc_dist_sq(&particles[i], &particles[j]);
            if d_sq < config.dhat * config.dhat {
                count += 1;
            }
        }
    }
    count
}

/// Total kinetic energy.
#[allow(dead_code)]
pub fn ipc_kinetic_energy(particles: &[IpcParticle]) -> f32 {
    particles
        .iter()
        .map(|p| {
            if p.inv_mass < 1e-10 {
                return 0.0;
            }
            let m = 1.0 / p.inv_mass;
            let v2: f32 = p.vel.iter().map(|x| x * x).sum();
            0.5 * m * v2
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close_pair() -> Vec<IpcParticle> {
        vec![
            IpcParticle::new([0.0, 0.0, 0.0], 1.0),
            IpcParticle::new([0.0005, 0.0, 0.0], 1.0),
        ]
    }

    fn far_pair() -> Vec<IpcParticle> {
        vec![
            IpcParticle::new([0.0, 0.0, 0.0], 1.0),
            IpcParticle::new([10.0, 0.0, 0.0], 1.0),
        ]
    }

    #[test]
    fn barrier_energy_positive_for_close_pair() {
        let config = IpcConfig::default();
        let ps = close_pair();
        let e = ipc_total_contact_energy(&ps, &config);
        assert!(e > 0.0);
    }

    #[test]
    fn barrier_energy_zero_for_far_pair() {
        let config = IpcConfig::default();
        let ps = far_pair();
        let e = ipc_total_contact_energy(&ps, &config);
        assert!(e < 1e-10);
    }

    #[test]
    fn active_pair_count_close() {
        let config = IpcConfig::default();
        let ps = close_pair();
        assert_eq!(ipc_active_pair_count(&ps, &config), 1);
    }

    #[test]
    fn active_pair_count_far() {
        let config = IpcConfig::default();
        let ps = far_pair();
        assert_eq!(ipc_active_pair_count(&ps, &config), 0);
    }

    #[test]
    fn gradient_step_separates_particles() {
        let config = IpcConfig::default();
        let mut ps = close_pair();
        let d_before = ipc_dist_sq(&ps[0], &ps[1]);
        ipc_gradient_step(&mut ps, &config, 1e-5);
        let d_after = ipc_dist_sq(&ps[0], &ps[1]);
        assert!(d_after >= d_before);
    }

    #[test]
    fn barrier_gradient_zero_outside_dhat() {
        let config = IpcConfig::default();
        let d_sq = (config.dhat * 2.0).powi(2);
        let g = ipc_barrier_gradient(d_sq, config.dhat, config.kappa);
        assert_eq!(g, 0.0);
    }

    #[test]
    fn kinetic_energy_zero_at_rest() {
        let ps = close_pair();
        assert!(ipc_kinetic_energy(&ps) < 1e-10);
    }

    #[test]
    fn new_particle_zero_vel() {
        let p = IpcParticle::new([1.0, 0.0, 0.0], 1.0);
        assert_eq!(p.vel, [0.0; 3]);
    }

    #[test]
    fn dist_sq_zero_for_same_pos() {
        let a = IpcParticle::new([1.0, 2.0, 3.0], 1.0);
        let b = IpcParticle::new([1.0, 2.0, 3.0], 1.0);
        assert!(ipc_dist_sq(&a, &b) < 1e-10);
    }

    #[test]
    fn ipc_config_default_values() {
        let c = IpcConfig::default();
        assert!(c.dhat > 0.0);
        assert!(c.kappa > 0.0);
        assert!(c.newton_iters > 0);
    }
}
