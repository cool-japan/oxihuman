// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! IK chain export: export inverse kinematics chain definitions.

/* ── legacy API (keep for existing lib.rs exports) ── */

#[derive(Debug, Clone)]
pub struct IkJointExport {
    pub bone_name: String,
    pub length: f32,
}

#[derive(Debug, Clone)]
pub struct IkChainExport {
    pub name: String,
    pub joints: Vec<IkJointExport>,
    pub target: [f32; 3],
    pub pole_target: Option<[f32; 3]>,
}

pub fn new_ik_chain_export(name: &str) -> IkChainExport {
    IkChainExport {
        name: name.to_string(),
        joints: Vec::new(),
        target: [0.0; 3],
        pole_target: None,
    }
}

pub fn ik_add_joint(chain: &mut IkChainExport, bone: &str, length: f32) {
    chain.joints.push(IkJointExport {
        bone_name: bone.to_string(),
        length,
    });
}

pub fn ik_joint_count(chain: &IkChainExport) -> usize {
    chain.joints.len()
}
pub fn ik_total_length(chain: &IkChainExport) -> f32 {
    chain.joints.iter().map(|j| j.length).sum()
}
pub fn ik_set_target(chain: &mut IkChainExport, t: [f32; 3]) {
    chain.target = t;
}
pub fn ik_set_pole(chain: &mut IkChainExport, p: [f32; 3]) {
    chain.pole_target = Some(p);
}
pub fn ik_has_pole(chain: &IkChainExport) -> bool {
    chain.pole_target.is_some()
}

pub fn ik_chain_to_json(chain: &IkChainExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"joints\":{},\"total_length\":{:.6}}}",
        chain.name,
        chain.joints.len(),
        ik_total_length(chain)
    )
}

pub fn ik_validate(chain: &IkChainExport) -> bool {
    !chain.joints.is_empty() && chain.joints.iter().all(|j| j.length > 0.0)
}

pub fn ik_clear(chain: &mut IkChainExport) {
    chain.joints.clear();
}

/* ── spec functions (wave 150B) ── */

/// A single bone in an IK chain (spec API).
#[derive(Debug, Clone)]
pub struct IkBone {
    pub name: String,
    pub length: f32,
}

/// A spec-style IK chain.
#[derive(Debug, Clone)]
pub struct IkChain {
    pub name: String,
    pub bones: Vec<IkBone>,
}

/// Create a new IkBone.
pub fn new_ik_bone(name: &str, length: f32) -> IkBone {
    IkBone {
        name: name.to_string(),
        length,
    }
}

/// Create a new IkChain.
pub fn new_ik_chain(name: &str) -> IkChain {
    IkChain {
        name: name.to_string(),
        bones: Vec::new(),
    }
}

/// Push a bone onto the chain.
pub fn ik_chain_push(chain: &mut IkChain, bone: IkBone) {
    chain.bones.push(bone);
}

/// Total length of all bones.
pub fn ik_chain_length(chain: &IkChain) -> f32 {
    chain.bones.iter().map(|b| b.length).sum()
}

/// Serialize an IkChain to JSON.
pub fn ik_chain_spec_to_json(chain: &IkChain) -> String {
    format!(
        "{{\"name\":\"{}\",\"bones\":{},\"length\":{:.6}}}",
        chain.name,
        chain.bones.len(),
        ik_chain_length(chain)
    )
}

/// Length of a single bone.
pub fn ik_bone_length(bone: &IkBone) -> f32 {
    bone.length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ik_chain_export() {
        assert_eq!(ik_joint_count(&new_ik_chain_export("arm")), 0);
    }

    #[test]
    fn test_add_joint() {
        let mut c = new_ik_chain_export("arm");
        ik_add_joint(&mut c, "upper", 1.0);
        assert_eq!(ik_joint_count(&c), 1);
    }

    #[test]
    fn test_total_length() {
        let mut c = new_ik_chain_export("arm");
        ik_add_joint(&mut c, "upper", 1.0);
        ik_add_joint(&mut c, "lower", 0.8);
        assert!((ik_total_length(&c) - 1.8).abs() < 1e-6);
    }

    #[test]
    fn test_new_ik_chain_spec() {
        let c = new_ik_chain("leg");
        assert_eq!(c.name, "leg");
        assert_eq!(c.bones.len(), 0);
    }

    #[test]
    fn test_ik_chain_push_and_length() {
        let mut c = new_ik_chain("leg");
        let b = new_ik_bone("thigh", 0.5);
        ik_chain_push(&mut c, b);
        assert!((ik_chain_length(&c) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_ik_bone_length() {
        let b = new_ik_bone("shin", 0.4);
        assert!((ik_bone_length(&b) - 0.4).abs() < 1e-5);
    }
}
