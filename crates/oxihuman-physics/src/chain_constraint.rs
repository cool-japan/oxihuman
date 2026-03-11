// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chain of linked particles (rope constraint).

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub link_length: f32,
    pub compliance: f32,
    pub inv_mass: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainConstraint {
    pub positions: Vec<[f32; 3]>,
    pub lambdas: Vec<f32>,
    pub config: ChainConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainSolveResult {
    pub max_error: f32,
    pub iterations: usize,
}

#[allow(dead_code)]
pub fn default_chain_config() -> ChainConfig {
    ChainConfig { link_length: 1.0, compliance: 0.0, inv_mass: 1.0 }
}

#[allow(dead_code)]
pub fn new_chain_constraint(node_count: usize, config: ChainConfig) -> ChainConstraint {
    let link_count = node_count.saturating_sub(1);
    let positions = (0..node_count)
        .map(|i| [i as f32 * config.link_length, 0.0, 0.0])
        .collect();
    ChainConstraint {
        positions,
        lambdas: vec![0.0; link_count],
        config,
    }
}

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
pub fn chain_solve_iteration(chain: &mut ChainConstraint, dt: f32) -> f32 {
    let n = chain.positions.len();
    if n < 2 {
        return 0.0;
    }
    let compliance = chain.config.compliance / (dt * dt);
    let w = chain.config.inv_mass;
    let mut max_err = 0.0f32;

    for i in 0..n - 1 {
        let delta = vec3_sub(chain.positions[i + 1], chain.positions[i]);
        let dist = vec3_len(delta);
        let error = dist - chain.config.link_length;
        max_err = max_err.max(error.abs());

        if dist < 1e-9 {
            continue;
        }

        let dir = [delta[0] / dist, delta[1] / dist, delta[2] / dist];
        let denom = 2.0 * w + compliance;
        if denom < 1e-12 {
            continue;
        }
        let d_lambda = (-error - compliance * chain.lambdas[i]) / denom;
        chain.lambdas[i] += d_lambda;

        let correction = d_lambda * w;
        chain.positions[i][0] -= dir[0] * correction;
        chain.positions[i][1] -= dir[1] * correction;
        chain.positions[i][2] -= dir[2] * correction;
        chain.positions[i + 1][0] += dir[0] * correction;
        chain.positions[i + 1][1] += dir[1] * correction;
        chain.positions[i + 1][2] += dir[2] * correction;
    }

    max_err
}

#[allow(dead_code)]
pub fn chain_error(chain: &ChainConstraint) -> f32 {
    let n = chain.positions.len();
    let mut max_err = 0.0f32;
    for i in 0..n.saturating_sub(1) {
        let delta = vec3_sub(chain.positions[i + 1], chain.positions[i]);
        let err = (vec3_len(delta) - chain.config.link_length).abs();
        max_err = max_err.max(err);
    }
    max_err
}

#[allow(dead_code)]
pub fn chain_reset_lambdas(chain: &mut ChainConstraint) {
    for l in &mut chain.lambdas {
        *l = 0.0;
    }
}

#[allow(dead_code)]
pub fn chain_link_count(chain: &ChainConstraint) -> usize {
    chain.lambdas.len()
}

#[allow(dead_code)]
pub fn chain_total_length(chain: &ChainConstraint) -> f32 {
    chain.config.link_length * chain.lambdas.len() as f32
}

#[allow(dead_code)]
pub fn chain_endpoint_distance(chain: &ChainConstraint) -> f32 {
    let n = chain.positions.len();
    if n < 2 {
        return 0.0;
    }
    vec3_len(vec3_sub(chain.positions[n - 1], chain.positions[0]))
}

#[allow(dead_code)]
pub fn chain_to_json(chain: &ChainConstraint) -> String {
    format!(
        r#"{{"nodes":{},"links":{},"link_length":{}}}"#,
        chain.positions.len(),
        chain.lambdas.len(),
        chain.config.link_length
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_chain() {
        let cfg = default_chain_config();
        let c = new_chain_constraint(5, cfg);
        assert_eq!(c.positions.len(), 5);
        assert_eq!(chain_link_count(&c), 4);
    }

    #[test]
    fn test_chain_error_zero_at_rest() {
        let cfg = default_chain_config();
        let c = new_chain_constraint(4, cfg);
        assert!(chain_error(&c) < 1e-5);
    }

    #[test]
    fn test_chain_total_length() {
        let cfg = ChainConfig { link_length: 2.0, compliance: 0.0, inv_mass: 1.0 };
        let c = new_chain_constraint(5, cfg);
        assert!((chain_total_length(&c) - 8.0).abs() < 1e-5);
    }

    #[test]
    fn test_chain_endpoint_distance() {
        let cfg = default_chain_config();
        let c = new_chain_constraint(5, cfg);
        let d = chain_endpoint_distance(&c);
        assert!((d - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_solve_iteration() {
        let cfg = default_chain_config();
        let mut c = new_chain_constraint(3, cfg);
        // Perturb a node
        c.positions[1][1] = 0.5;
        let err = chain_solve_iteration(&mut c, 0.016);
        assert!(err >= 0.0);
    }

    #[test]
    fn test_reset_lambdas() {
        let cfg = default_chain_config();
        let mut c = new_chain_constraint(3, cfg);
        c.lambdas[0] = 5.0;
        chain_reset_lambdas(&mut c);
        assert!(c.lambdas[0].abs() < 1e-9);
    }

    #[test]
    fn test_single_node_chain() {
        let cfg = default_chain_config();
        let c = new_chain_constraint(1, cfg);
        assert_eq!(chain_link_count(&c), 0);
        assert!((chain_endpoint_distance(&c) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_chain_config();
        let c = new_chain_constraint(3, cfg);
        let json = chain_to_json(&c);
        assert!(json.contains("nodes"));
        assert!(json.contains("links"));
    }

    #[test]
    fn test_solve_reduces_error() {
        let cfg = default_chain_config();
        let mut c = new_chain_constraint(4, cfg);
        c.positions[2][1] = 2.0; // introduce error
        let before = chain_error(&c);
        for _ in 0..10 {
            chain_solve_iteration(&mut c, 0.016);
        }
        let after = chain_error(&c);
        assert!(after < before || before < 1e-5);
    }
}
