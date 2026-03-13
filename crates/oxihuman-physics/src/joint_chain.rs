#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A chain of joints connected in sequence.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointChain {
    joints: Vec<(String, f32)>, // (name, error)
    complete: bool,
}

#[allow(dead_code)]
pub fn new_joint_chain() -> JointChain {
    JointChain {
        joints: Vec::new(),
        complete: false,
    }
}

#[allow(dead_code)]
pub fn chain_add_joint(chain: &mut JointChain, name: &str, error: f32) {
    chain.joints.push((name.to_string(), error));
    chain.complete = false;
}

#[allow(dead_code)]
pub fn chain_joint_count(chain: &JointChain) -> usize {
    chain.joints.len()
}

#[allow(dead_code)]
pub fn chain_solve(chain: &mut JointChain, damping: f32) -> f32 {
    let mut total = 0.0_f32;
    for joint in &mut chain.joints {
        joint.1 *= damping;
        total += joint.1.abs();
    }
    if total < 1e-6 {
        chain.complete = true;
    }
    total
}

#[allow(dead_code)]
pub fn chain_error_total(chain: &JointChain) -> f32 {
    chain.joints.iter().map(|(_, e)| e.abs()).sum()
}

#[allow(dead_code)]
pub fn chain_reset(chain: &mut JointChain) {
    chain.joints.clear();
    chain.complete = false;
}

#[allow(dead_code)]
pub fn chain_to_json(chain: &JointChain) -> String {
    let joints: Vec<String> = chain
        .joints
        .iter()
        .map(|(n, e)| format!("{{\"name\":\"{}\",\"error\":{:.6}}}", n, e))
        .collect();
    format!(
        "{{\"count\":{},\"complete\":{},\"joints\":[{}]}}",
        chain.joints.len(),
        chain.complete,
        joints.join(",")
    )
}

#[allow(dead_code)]
pub fn chain_is_complete(chain: &JointChain) -> bool {
    chain.complete
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_joint_chain() {
        let c = new_joint_chain();
        assert_eq!(chain_joint_count(&c), 0);
    }

    #[test]
    fn test_chain_add_joint() {
        let mut c = new_joint_chain();
        chain_add_joint(&mut c, "hip", 0.1);
        assert_eq!(chain_joint_count(&c), 1);
    }

    #[test]
    fn test_chain_joint_count() {
        let mut c = new_joint_chain();
        chain_add_joint(&mut c, "a", 0.1);
        chain_add_joint(&mut c, "b", 0.2);
        assert_eq!(chain_joint_count(&c), 2);
    }

    #[test]
    fn test_chain_solve() {
        let mut c = new_joint_chain();
        chain_add_joint(&mut c, "a", 1.0);
        let total = chain_solve(&mut c, 0.5);
        assert!((total - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_chain_error_total() {
        let mut c = new_joint_chain();
        chain_add_joint(&mut c, "a", 1.0);
        chain_add_joint(&mut c, "b", 2.0);
        assert!((chain_error_total(&c) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_chain_reset() {
        let mut c = new_joint_chain();
        chain_add_joint(&mut c, "a", 1.0);
        chain_reset(&mut c);
        assert_eq!(chain_joint_count(&c), 0);
    }

    #[test]
    fn test_chain_to_json() {
        let mut c = new_joint_chain();
        chain_add_joint(&mut c, "test", 0.5);
        let json = chain_to_json(&c);
        assert!(json.contains("\"name\":\"test\""));
    }

    #[test]
    fn test_chain_is_complete() {
        let mut c = new_joint_chain();
        chain_add_joint(&mut c, "a", 0.0);
        chain_solve(&mut c, 0.0);
        assert!(chain_is_complete(&c));
    }

    #[test]
    fn test_chain_not_complete() {
        let mut c = new_joint_chain();
        chain_add_joint(&mut c, "a", 1.0);
        chain_solve(&mut c, 0.9);
        assert!(!chain_is_complete(&c));
    }

    #[test]
    fn test_empty_chain() {
        let c = new_joint_chain();
        assert!((chain_error_total(&c)).abs() < 1e-6);
    }
}
