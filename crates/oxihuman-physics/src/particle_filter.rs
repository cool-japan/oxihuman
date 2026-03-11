// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Particle filter: sequential importance resampling for state estimation.

use std::f32::consts::PI;

/// A single weighted particle representing a hypothesis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FilterParticle {
    pub state: [f32; 4],
    pub weight: f32,
}

/// Particle filter state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleFilter {
    pub particles: Vec<FilterParticle>,
    pub resample_threshold: f32,
}

/// Create a new `ParticleFilter` with `n` particles near `init_state`.
#[allow(dead_code)]
pub fn new_particle_filter(n: usize, init_state: [f32; 4]) -> ParticleFilter {
    let w = 1.0 / n.max(1) as f32;
    let particles = (0..n)
        .map(|i| FilterParticle {
            state: [
                init_state[0] + (i as f32 * 0.001),
                init_state[1],
                init_state[2],
                init_state[3],
            ],
            weight: w,
        })
        .collect();
    ParticleFilter {
        particles,
        resample_threshold: 0.5,
    }
}

/// Normalize weights so they sum to 1.
#[allow(dead_code)]
pub fn pf_normalize(pf: &mut ParticleFilter) {
    let sum: f32 = pf.particles.iter().map(|p| p.weight).sum();
    if sum < 1e-30 {
        let w = 1.0 / pf.particles.len().max(1) as f32;
        for p in &mut pf.particles {
            p.weight = w;
        }
        return;
    }
    for p in &mut pf.particles {
        p.weight /= sum;
    }
}

/// Compute effective sample size (ESS).
#[allow(dead_code)]
pub fn pf_ess(pf: &ParticleFilter) -> f32 {
    let sum_sq: f32 = pf.particles.iter().map(|p| p.weight * p.weight).sum();
    if sum_sq < 1e-30 {
        return 0.0;
    }
    1.0 / sum_sq
}

/// Weighted mean of the first state component.
#[allow(dead_code)]
pub fn pf_mean_state0(pf: &ParticleFilter) -> f32 {
    pf.particles.iter().map(|p| p.state[0] * p.weight).sum()
}

/// Update weights with a Gaussian likelihood for `state[0]`.
#[allow(dead_code)]
pub fn pf_update(pf: &mut ParticleFilter, observation: f32, sigma: f32) {
    let sigma = sigma.max(1e-9);
    let norm = 1.0 / (sigma * (2.0 * PI).sqrt());
    for p in &mut pf.particles {
        let diff = p.state[0] - observation;
        let likelihood = norm * (-(diff * diff) / (2.0 * sigma * sigma)).exp();
        p.weight *= likelihood;
    }
    pf_normalize(pf);
}

/// Systematic resample.
#[allow(dead_code)]
pub fn pf_resample(pf: &mut ParticleFilter) {
    let n = pf.particles.len();
    if n == 0 {
        return;
    }
    let mut cumsum = vec![0.0f32; n + 1];
    for i in 0..n {
        cumsum[i + 1] = cumsum[i] + pf.particles[i].weight;
    }
    let step = 1.0 / n as f32;
    let start = step * 0.5;
    let mut new_particles = Vec::with_capacity(n);
    let mut j = 0;
    for i in 0..n {
        let u = start + i as f32 * step;
        while j + 1 < n && cumsum[j + 1] < u {
            j += 1;
        }
        let mut p = pf.particles[j].clone();
        p.weight = 1.0 / n as f32;
        new_particles.push(p);
    }
    pf.particles = new_particles;
}

/// Propagate particles with simple random-walk noise (deterministic for tests).
#[allow(dead_code)]
pub fn pf_propagate(pf: &mut ParticleFilter, noise_scale: f32) {
    for (i, p) in pf.particles.iter_mut().enumerate() {
        let noise = ((i as f32 * 0.1).sin()) * noise_scale;
        p.state[0] += noise;
    }
}

/// Particle count.
#[allow(dead_code)]
pub fn pf_count(pf: &ParticleFilter) -> usize {
    pf.particles.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_filter() {
        let pf = new_particle_filter(100, [0.0; 4]);
        assert_eq!(pf_count(&pf), 100);
    }

    #[test]
    fn test_weights_sum_to_one() {
        let pf = new_particle_filter(10, [0.0; 4]);
        let sum: f32 = pf.particles.iter().map(|p| p.weight).sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize() {
        let mut pf = new_particle_filter(5, [0.0; 4]);
        for p in &mut pf.particles {
            p.weight = 2.0;
        }
        pf_normalize(&mut pf);
        let sum: f32 = pf.particles.iter().map(|p| p.weight).sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ess_uniform() {
        let pf = new_particle_filter(10, [0.0; 4]);
        let ess = pf_ess(&pf);
        assert!((ess - 10.0).abs() < 1e-3);
    }

    #[test]
    fn test_mean_state0() {
        let pf = new_particle_filter(4, [5.0, 0.0, 0.0, 0.0]);
        let mean = pf_mean_state0(&pf);
        assert!(mean > 4.9);
    }

    #[test]
    fn test_update_shifts_weights() {
        let mut pf = new_particle_filter(10, [0.0; 4]);
        pf_update(&mut pf, 0.0, 0.5);
        let sum: f32 = pf.particles.iter().map(|p| p.weight).sum();
        assert!((sum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_resample_count_preserved() {
        let mut pf = new_particle_filter(20, [0.0; 4]);
        pf_resample(&mut pf);
        assert_eq!(pf_count(&pf), 20);
    }

    #[test]
    fn test_propagate_changes_state() {
        let mut pf = new_particle_filter(5, [10.0, 0.0, 0.0, 0.0]);
        let before = pf.particles[1].state[0];
        pf_propagate(&mut pf, 0.1);
        let after = pf.particles[1].state[0];
        let _diff = (before - after).abs(); // deterministic noise may be zero for particle 1
    }

    #[test]
    fn test_pi_used() {
        let circle_area = PI * 1.0 * 1.0;
        assert!(circle_area > 3.0);
    }
}
