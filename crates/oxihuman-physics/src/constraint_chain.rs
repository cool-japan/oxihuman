#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A chain of constraints solved sequentially.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintChain {
    constraints: Vec<(String, f32)>,
    converged: bool,
}

#[allow(dead_code)]
pub fn new_constraint_chain() -> ConstraintChain {
    ConstraintChain {
        constraints: Vec::new(),
        converged: false,
    }
}

#[allow(dead_code)]
pub fn add_constraint_cc(chain: &mut ConstraintChain, name: &str, error: f32) {
    chain.constraints.push((name.to_string(), error));
    chain.converged = false;
}

#[allow(dead_code)]
pub fn chain_solve_cc(chain: &mut ConstraintChain, damping: f32) -> f32 {
    let mut total = 0.0f32;
    for c in &mut chain.constraints {
        c.1 *= damping;
        total += c.1.abs();
    }
    chain.converged = total < 1e-6;
    total
}

#[allow(dead_code)]
pub fn chain_count(chain: &ConstraintChain) -> usize {
    chain.constraints.len()
}

#[allow(dead_code)]
pub fn chain_error_cc(chain: &ConstraintChain) -> f32 {
    chain.constraints.iter().map(|(_, e)| e.abs()).sum()
}

#[allow(dead_code)]
pub fn chain_reset_cc(chain: &mut ConstraintChain) {
    chain.constraints.clear();
    chain.converged = false;
}

#[allow(dead_code)]
pub fn chain_to_json_cc(chain: &ConstraintChain) -> String {
    let items: Vec<String> = chain
        .constraints
        .iter()
        .map(|(n, e)| format!("{{\"name\":\"{}\",\"error\":{:.6}}}", n, e))
        .collect();
    format!(
        "{{\"count\":{},\"converged\":{},\"constraints\":[{}]}}",
        chain.constraints.len(),
        chain.converged,
        items.join(",")
    )
}

#[allow(dead_code)]
pub fn chain_converged(chain: &ConstraintChain) -> bool {
    chain.converged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_constraint_chain() {
        let c = new_constraint_chain();
        assert_eq!(chain_count(&c), 0);
    }

    #[test]
    fn test_add_constraint_cc() {
        let mut c = new_constraint_chain();
        add_constraint_cc(&mut c, "dist", 0.5);
        assert_eq!(chain_count(&c), 1);
    }

    #[test]
    fn test_chain_solve_cc() {
        let mut c = new_constraint_chain();
        add_constraint_cc(&mut c, "dist", 1.0);
        let err = chain_solve_cc(&mut c, 0.5);
        assert!((err - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_chain_count() {
        let mut c = new_constraint_chain();
        add_constraint_cc(&mut c, "a", 0.1);
        add_constraint_cc(&mut c, "b", 0.2);
        assert_eq!(chain_count(&c), 2);
    }

    #[test]
    fn test_chain_error_cc() {
        let mut c = new_constraint_chain();
        add_constraint_cc(&mut c, "a", 1.0);
        add_constraint_cc(&mut c, "b", 2.0);
        assert!((chain_error_cc(&c) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_chain_reset_cc() {
        let mut c = new_constraint_chain();
        add_constraint_cc(&mut c, "a", 1.0);
        chain_reset_cc(&mut c);
        assert_eq!(chain_count(&c), 0);
    }

    #[test]
    fn test_chain_to_json_cc() {
        let mut c = new_constraint_chain();
        add_constraint_cc(&mut c, "test", 0.5);
        let json = chain_to_json_cc(&c);
        assert!(json.contains("\"name\":\"test\""));
    }

    #[test]
    fn test_chain_converged() {
        let mut c = new_constraint_chain();
        add_constraint_cc(&mut c, "a", 0.0);
        chain_solve_cc(&mut c, 0.0);
        assert!(chain_converged(&c));
    }

    #[test]
    fn test_chain_not_converged() {
        let mut c = new_constraint_chain();
        add_constraint_cc(&mut c, "a", 1.0);
        chain_solve_cc(&mut c, 0.9);
        assert!(!chain_converged(&c));
    }

    #[test]
    fn test_empty_chain_error() {
        let c = new_constraint_chain();
        assert!(chain_error_cc(&c).abs() < 1e-6);
    }
}
