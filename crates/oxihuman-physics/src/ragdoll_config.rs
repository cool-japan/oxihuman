// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single bone in a ragdoll mapped to a physics body.
#[allow(dead_code)]
pub struct RagdollBone {
    pub bone_name: String,
    pub body_id: u32,
    pub half_extents: [f32; 3],
    pub mass: f32,
}

/// Complete ragdoll configuration.
#[allow(dead_code)]
pub struct RagdollConfig {
    pub bones: Vec<RagdollBone>,
}

/// Create a new empty `RagdollConfig`.
#[allow(dead_code)]
pub fn new_ragdoll_config() -> RagdollConfig {
    RagdollConfig { bones: Vec::new() }
}

/// Add a bone to the ragdoll configuration.
#[allow(dead_code)]
pub fn add_ragdoll_bone(
    cfg: &mut RagdollConfig,
    bone: &str,
    id: u32,
    half: [f32; 3],
    mass: f32,
) {
    cfg.bones.push(RagdollBone {
        bone_name: bone.to_string(),
        body_id: id,
        half_extents: half,
        mass,
    });
}

/// Look up a bone by name.
#[allow(dead_code)]
pub fn get_ragdoll_bone<'a>(cfg: &'a RagdollConfig, bone: &str) -> Option<&'a RagdollBone> {
    cfg.bones.iter().find(|b| b.bone_name == bone)
}

/// Return the number of bones.
#[allow(dead_code)]
pub fn ragdoll_bone_count(cfg: &RagdollConfig) -> usize {
    cfg.bones.len()
}

/// Return the total mass of all bones.
#[allow(dead_code)]
pub fn ragdoll_total_mass(cfg: &RagdollConfig) -> f32 {
    cfg.bones.iter().map(|b| b.mass).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_config_empty() {
        let cfg = new_ragdoll_config();
        assert_eq!(ragdoll_bone_count(&cfg), 0);
    }

    #[test]
    fn add_bone_increments_count() {
        let mut cfg = new_ragdoll_config();
        add_ragdoll_bone(&mut cfg, "spine", 1, [0.1, 0.2, 0.1], 5.0);
        assert_eq!(ragdoll_bone_count(&cfg), 1);
    }

    #[test]
    fn get_bone_by_name() {
        let mut cfg = new_ragdoll_config();
        add_ragdoll_bone(&mut cfg, "head", 10, [0.1, 0.1, 0.1], 4.0);
        let bone = get_ragdoll_bone(&cfg, "head").unwrap();
        assert_eq!(bone.body_id, 10);
    }

    #[test]
    fn get_missing_bone_returns_none() {
        let cfg = new_ragdoll_config();
        assert!(get_ragdoll_bone(&cfg, "missing").is_none());
    }

    #[test]
    fn total_mass_sums_correctly() {
        let mut cfg = new_ragdoll_config();
        add_ragdoll_bone(&mut cfg, "arm_l", 1, [0.05, 0.2, 0.05], 1.5);
        add_ragdoll_bone(&mut cfg, "arm_r", 2, [0.05, 0.2, 0.05], 1.5);
        assert!((ragdoll_total_mass(&cfg) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn total_mass_empty_config() {
        let cfg = new_ragdoll_config();
        assert_eq!(ragdoll_total_mass(&cfg), 0.0);
    }

    #[test]
    fn half_extents_stored() {
        let mut cfg = new_ragdoll_config();
        let half = [0.15, 0.3, 0.1];
        add_ragdoll_bone(&mut cfg, "torso", 5, half, 10.0);
        let bone = get_ragdoll_bone(&cfg, "torso").unwrap();
        assert_eq!(bone.half_extents, half);
    }

    #[test]
    fn mass_stored() {
        let mut cfg = new_ragdoll_config();
        add_ragdoll_bone(&mut cfg, "leg_l", 3, [0.07, 0.4, 0.07], 7.5);
        let bone = get_ragdoll_bone(&cfg, "leg_l").unwrap();
        assert!((bone.mass - 7.5).abs() < 1e-6);
    }

    #[test]
    fn multiple_bones_independent() {
        let mut cfg = new_ragdoll_config();
        add_ragdoll_bone(&mut cfg, "a", 1, [0.1; 3], 1.0);
        add_ragdoll_bone(&mut cfg, "b", 2, [0.2; 3], 2.0);
        assert_eq!(get_ragdoll_bone(&cfg, "a").unwrap().body_id, 1);
        assert_eq!(get_ragdoll_bone(&cfg, "b").unwrap().body_id, 2);
    }

    #[test]
    fn bone_count_multiple() {
        let mut cfg = new_ragdoll_config();
        for i in 0..8 {
            add_ragdoll_bone(&mut cfg, &format!("bone{i}"), i as u32, [0.1; 3], 1.0);
        }
        assert_eq!(ragdoll_bone_count(&cfg), 8);
    }
}
