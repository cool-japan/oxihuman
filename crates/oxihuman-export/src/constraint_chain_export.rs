// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export constraint chains (IK chains, look-at chains, etc.).

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ChainConstraintType { IK, LookAt, CopyRotation, CopyLocation }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainConstraint {
    pub name: String,
    pub constraint_type: ChainConstraintType,
    pub target: String,
    pub chain_length: u32,
    pub influence: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintChainExport {
    pub constraints: Vec<ChainConstraint>,
}

#[allow(dead_code)]
pub fn new_constraint_chain_export() -> ConstraintChainExport {
    ConstraintChainExport { constraints: Vec::new() }
}

#[allow(dead_code)]
pub fn cce_add(cce: &mut ConstraintChainExport, name: &str, ct: ChainConstraintType, target: &str, chain_len: u32, influence: f32) {
    cce.constraints.push(ChainConstraint { name: name.to_string(), constraint_type: ct, target: target.to_string(), chain_length: chain_len, influence: influence.clamp(0.0, 1.0) });
}

#[allow(dead_code)]
pub fn cce_count(cce: &ConstraintChainExport) -> usize { cce.constraints.len() }

#[allow(dead_code)]
pub fn cce_find<'a>(cce: &'a ConstraintChainExport, name: &str) -> Option<&'a ChainConstraint> {
    cce.constraints.iter().find(|c| c.name == name)
}

#[allow(dead_code)]
pub fn cce_ik_count(cce: &ConstraintChainExport) -> usize {
    cce.constraints.iter().filter(|c| c.constraint_type == ChainConstraintType::IK).count()
}

#[allow(dead_code)]
pub fn cce_max_chain(cce: &ConstraintChainExport) -> u32 {
    cce.constraints.iter().map(|c| c.chain_length).max().unwrap_or(0)
}

#[allow(dead_code)]
pub fn cce_validate(cce: &ConstraintChainExport) -> bool {
    cce.constraints.iter().all(|c| !c.name.is_empty() && !c.target.is_empty() && (0.0..=1.0).contains(&c.influence))
}

#[allow(dead_code)]
pub fn cce_to_json(cce: &ConstraintChainExport) -> String {
    let items: Vec<String> = cce.constraints.iter().map(|c| format!("{{\"name\":\"{}\",\"target\":\"{}\"}}", c.name, c.target)).collect();
    format!("{{\"constraints\":[{}]}}", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ConstraintChainExport {
        let mut c = new_constraint_chain_export();
        cce_add(&mut c, "ik_leg", ChainConstraintType::IK, "foot_target", 3, 1.0);
        c
    }

    #[test] fn test_new() { assert_eq!(cce_count(&new_constraint_chain_export()), 0); }
    #[test] fn test_add() { assert_eq!(cce_count(&sample()), 1); }
    #[test] fn test_find() { assert!(cce_find(&sample(), "ik_leg").is_some()); }
    #[test] fn test_find_missing() { assert!(cce_find(&sample(), "nope").is_none()); }
    #[test] fn test_ik_count() { assert_eq!(cce_ik_count(&sample()), 1); }
    #[test] fn test_max_chain() { assert_eq!(cce_max_chain(&sample()), 3); }
    #[test] fn test_validate() { assert!(cce_validate(&sample())); }
    #[test] fn test_to_json() { assert!(cce_to_json(&sample()).contains("ik_leg")); }
    #[test] fn test_influence_clamp() {
        let mut c = new_constraint_chain_export();
        cce_add(&mut c, "x", ChainConstraintType::LookAt, "t", 1, 2.0);
        assert!((c.constraints[0].influence - 1.0).abs() < 1e-6);
    }
    #[test] fn test_type() { let s = sample(); assert_eq!(s.constraints[0].constraint_type, ChainConstraintType::IK); }
}
