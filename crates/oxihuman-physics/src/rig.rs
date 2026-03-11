// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Skeleton rig adapter: hierarchy of joints linking body-part capsule proxies
//! into a physics skeleton.

use crate::BodyProxies;

// ── RigJoint ─────────────────────────────────────────────────────────────────

/// A single joint in the physics rig.
#[derive(Debug, Clone)]
pub struct RigJoint {
    /// Human-readable name for this joint (e.g. "pelvis", "torso", "head").
    pub name: String,
    /// Index into [`PhysicsRig::joints`] of the parent joint, or `None` for root.
    pub parent: Option<usize>,
    /// Position relative to parent joint (or world-space if root).
    pub local_position: [f32; 3],
    /// Which `CapsuleProxy` / `SphereProxy` label this joint drives, if any.
    pub capsule_label: Option<String>,
}

impl RigJoint {
    fn new(
        name: &str,
        parent: Option<usize>,
        local_position: [f32; 3],
        capsule_label: Option<&str>,
    ) -> Self {
        RigJoint {
            name: name.to_string(),
            parent,
            local_position,
            capsule_label: capsule_label.map(|s| s.to_string()),
        }
    }
}

// ── PhysicsRig ───────────────────────────────────────────────────────────────

/// A complete physics rig built from body proxies.
#[derive(Debug, Clone, Default)]
pub struct PhysicsRig {
    pub joints: Vec<RigJoint>,
}

impl PhysicsRig {
    /// Create an empty rig.
    pub fn new() -> Self {
        PhysicsRig { joints: Vec::new() }
    }

    /// Total number of joints in this rig.
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    /// All root joints (those with `parent == None`).
    pub fn root_joints(&self) -> Vec<&RigJoint> {
        self.joints.iter().filter(|j| j.parent.is_none()).collect()
    }

    /// All children of joint at `idx`: returns `(child_index, &RigJoint)` pairs.
    pub fn children_of(&self, idx: usize) -> Vec<(usize, &RigJoint)> {
        self.joints
            .iter()
            .enumerate()
            .filter(|(_, j)| j.parent == Some(idx))
            .collect()
    }

    /// Find the index of the first joint with the given name, or `None`.
    pub fn find_joint(&self, name: &str) -> Option<usize> {
        self.joints.iter().position(|j| j.name == name)
    }

    /// Serialize the rig to hand-written JSON.
    pub fn to_json(&self) -> String {
        let mut out = String::from("{\n  \"joints\": [\n");
        for (i, j) in self.joints.iter().enumerate() {
            let comma = if i + 1 < self.joints.len() { "," } else { "" };
            let parent_str = match j.parent {
                Some(p) => format!("{}", p),
                None => "null".to_string(),
            };
            let label_str = match &j.capsule_label {
                Some(l) => format!("\"{}\"", l),
                None => "null".to_string(),
            };
            out.push_str(&format!(
                "    {{\"name\":\"{}\",\"parent\":{},\"local_position\":[{},{},{}],\"capsule_label\":{}}}{}\n",
                j.name,
                parent_str,
                j.local_position[0],
                j.local_position[1],
                j.local_position[2],
                label_str,
                comma
            ));
        }
        out.push_str("  ]\n}");
        out
    }
}

// ── build_rig ────────────────────────────────────────────────────────────────

/// Build a standard humanoid [`PhysicsRig`] from a [`BodyProxies`] set.
///
/// Joint hierarchy (13 joints total):
/// ```text
/// pelvis  (root)
/// ├── hips
/// ├── torso
/// │   └── head
/// ├── leg_l
/// │   └── shin_l
/// ├── leg_r
/// │   └── shin_r
/// ├── arm_l
/// │   └── forearm_l
/// └── arm_r
///     └── forearm_r
/// ```
///
/// Each joint's `local_position` is derived from `center_a` of the corresponding
/// `CapsuleProxy`, or the sphere center for "head".  For joints not found in
/// `proxies`, a zero position is used.  If `proxies` is empty the returned rig
/// will have only the root "pelvis" joint.
#[allow(dead_code)]
pub fn build_rig(proxies: &BodyProxies) -> PhysicsRig {
    // Helper: look up center_a of a named capsule proxy.
    let cap_pos = |label: &str| -> [f32; 3] {
        proxies
            .capsules
            .iter()
            .find(|c| c.label == label)
            .map(|c| c.center_a)
            .unwrap_or([0.0, 0.0, 0.0])
    };

    // Helper: look up center of a named sphere proxy.
    let sph_pos = |label: &str| -> [f32; 3] {
        proxies
            .spheres
            .iter()
            .find(|s| s.label == label)
            .map(|s| s.center)
            .unwrap_or([0.0, 0.0, 0.0])
    };

    let mut rig = PhysicsRig::new();

    // 0 — pelvis (root)
    let pelvis_pos = cap_pos("hips");
    rig.joints
        .push(RigJoint::new("pelvis", None, pelvis_pos, Some("hips")));

    // 1 — hips
    let hips_pos = cap_pos("hips");
    rig.joints
        .push(RigJoint::new("hips", Some(0), hips_pos, Some("hips")));

    // 2 — torso
    let torso_pos = cap_pos("torso");
    rig.joints
        .push(RigJoint::new("torso", Some(0), torso_pos, Some("torso")));

    // 3 — head (child of torso, idx 2)
    let head_pos = sph_pos("head");
    rig.joints
        .push(RigJoint::new("head", Some(2), head_pos, Some("head")));

    // 4 — leg_l
    let leg_l_pos = cap_pos("leg_l");
    rig.joints
        .push(RigJoint::new("leg_l", Some(0), leg_l_pos, Some("leg_l")));

    // 5 — shin_l (child of leg_l, idx 4)
    let shin_l_pos = cap_pos("shin_l");
    rig.joints
        .push(RigJoint::new("shin_l", Some(4), shin_l_pos, Some("shin_l")));

    // 6 — leg_r
    let leg_r_pos = cap_pos("leg_r");
    rig.joints
        .push(RigJoint::new("leg_r", Some(0), leg_r_pos, Some("leg_r")));

    // 7 — shin_r (child of leg_r, idx 6)
    let shin_r_pos = cap_pos("shin_r");
    rig.joints
        .push(RigJoint::new("shin_r", Some(6), shin_r_pos, Some("shin_r")));

    // 8 — arm_l
    let arm_l_pos = cap_pos("arm_l");
    rig.joints
        .push(RigJoint::new("arm_l", Some(0), arm_l_pos, Some("arm_l")));

    // 9 — forearm_l (child of arm_l, idx 8)
    let forearm_l_pos = cap_pos("forearm_l");
    rig.joints.push(RigJoint::new(
        "forearm_l",
        Some(8),
        forearm_l_pos,
        Some("forearm_l"),
    ));

    // 10 — arm_r
    let arm_r_pos = cap_pos("arm_r");
    rig.joints
        .push(RigJoint::new("arm_r", Some(0), arm_r_pos, Some("arm_r")));

    // 11 — forearm_r (child of arm_r, idx 10)
    let forearm_r_pos = cap_pos("forearm_r");
    rig.joints.push(RigJoint::new(
        "forearm_r",
        Some(10),
        forearm_r_pos,
        Some("forearm_r"),
    ));

    // If proxies were empty, strip everything except the root.
    if proxies.capsules.is_empty() && proxies.spheres.is_empty() {
        rig.joints.truncate(1);
    }

    rig
}

// ── CapsuleChain ─────────────────────────────────────────────────────────────

/// An ordered sequence of joint indices forming a single limb / spine chain.
#[derive(Debug, Clone)]
pub struct CapsuleChain {
    /// Descriptive name for this chain (e.g. "spine", "left_leg").
    pub name: String,
    /// Ordered joint indices within the parent [`PhysicsRig`].
    pub joint_indices: Vec<usize>,
}

impl CapsuleChain {
    /// Extract the five standard chains from `rig`.
    ///
    /// | Chain name  | Joints (in order)            |
    /// |-------------|------------------------------|
    /// | spine       | hips → torso → head          |
    /// | left_leg    | leg_l → shin_l               |
    /// | right_leg   | leg_r → shin_r               |
    /// | left_arm    | arm_l → forearm_l            |
    /// | right_arm   | arm_r → forearm_r            |
    ///
    /// Joints that cannot be found in `rig` are silently skipped; if that
    /// causes a chain to have fewer than 2 members it is still returned but
    /// will be empty (or a single joint).
    pub fn standard_chains(rig: &PhysicsRig) -> Vec<CapsuleChain> {
        let lookup = |name: &str| rig.find_joint(name);

        let chain = |chain_name: &str, names: &[&str]| -> CapsuleChain {
            let indices = names.iter().filter_map(|n| lookup(n)).collect();
            CapsuleChain {
                name: chain_name.to_string(),
                joint_indices: indices,
            }
        };

        vec![
            chain("spine", &["hips", "torso", "head"]),
            chain("left_leg", &["leg_l", "shin_l"]),
            chain("right_leg", &["leg_r", "shin_r"]),
            chain("left_arm", &["arm_l", "forearm_l"]),
            chain("right_arm", &["arm_r", "forearm_r"]),
        ]
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyProxies, CapsuleProxy, SphereProxy};

    /// Build a minimal BodyProxies that contains the standard labels so that
    /// `build_rig` produces a full 12-joint rig (pelvis + 11 body parts).
    fn make_proxies() -> BodyProxies {
        let mut p = BodyProxies::new();

        // head sphere
        p.spheres
            .push(SphereProxy::new([0.0, 1.7, 0.0], 0.12, "head"));

        // capsule proxies
        for &(label, ya, yb) in &[
            ("torso", 0.9f32, 1.4f32),
            ("hips", 0.75f32, 0.9f32),
            ("leg_l", 0.5f32, 0.75f32),
            ("leg_r", 0.5f32, 0.75f32),
            ("shin_l", 0.1f32, 0.5f32),
            ("shin_r", 0.1f32, 0.5f32),
            ("arm_l", 1.2f32, 1.4f32),
            ("arm_r", 1.2f32, 1.4f32),
            ("forearm_l", 0.9f32, 1.2f32),
            ("forearm_r", 0.9f32, 1.2f32),
        ] {
            p.capsules.push(CapsuleProxy::new(
                [0.0, ya, 0.0],
                [0.0, yb, 0.0],
                0.1,
                label,
            ));
        }

        p
    }

    // 1. build_rig produces a non-empty rig
    #[test]
    fn build_rig_non_empty() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        assert!(!rig.joints.is_empty());
    }

    // 2. root_joints returns exactly 1 root (pelvis)
    #[test]
    fn root_joints_exactly_one() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        let roots = rig.root_joints();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "pelvis");
    }

    // 3. joint_count is correct (12 joints: pelvis + 11 body parts)
    #[test]
    fn joint_count_is_twelve() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        // pelvis + hips + torso + head + leg_l + shin_l + leg_r + shin_r
        // + arm_l + forearm_l + arm_r + forearm_r = 12
        assert_eq!(rig.joint_count(), 12);
    }

    // 4. find_joint("torso") returns Some
    #[test]
    fn find_joint_torso_some() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        assert!(rig.find_joint("torso").is_some());
    }

    // 5. find_joint("nonexistent") returns None
    #[test]
    fn find_joint_nonexistent_none() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        assert!(rig.find_joint("nonexistent").is_none());
    }

    // 6. children_of(pelvis_idx) returns the expected direct children
    #[test]
    fn children_of_pelvis() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        let root_idx = rig.find_joint("pelvis").unwrap();
        let children: Vec<_> = rig
            .children_of(root_idx)
            .into_iter()
            .map(|(_, j)| j.name.as_str())
            .collect();

        // pelvis directly parents: hips, torso, leg_l, leg_r, arm_l, arm_r
        for expected in &["hips", "torso", "leg_l", "leg_r", "arm_l", "arm_r"] {
            assert!(
                children.contains(expected),
                "expected child '{}' of pelvis, got {:?}",
                expected,
                children
            );
        }
    }

    // 7. to_json is valid JSON and contains "joints"
    #[test]
    fn to_json_contains_joints_key() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        let json = rig.to_json();
        assert!(
            json.contains("\"joints\""),
            "JSON should contain 'joints' key"
        );
        // Rudimentary JSON validity: starts with '{' and ends with '}'
        let trimmed = json.trim();
        assert!(trimmed.starts_with('{'));
        assert!(trimmed.ends_with('}'));
    }

    // 8. standard_chains returns exactly 5 chains
    #[test]
    fn standard_chains_count_five() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        let chains = CapsuleChain::standard_chains(&rig);
        assert_eq!(chains.len(), 5);
    }

    // 9. each chain has at least 2 joints
    #[test]
    fn each_chain_has_at_least_two_joints() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        let chains = CapsuleChain::standard_chains(&rig);
        for chain in &chains {
            assert!(
                chain.joint_indices.len() >= 2,
                "chain '{}' has only {} joints",
                chain.name,
                chain.joint_indices.len()
            );
        }
    }

    // 10. joint local_positions are all finite f32 values
    #[test]
    fn joint_local_positions_are_finite() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        for joint in &rig.joints {
            for &coord in &joint.local_position {
                assert!(
                    coord.is_finite(),
                    "joint '{}' has non-finite local_position",
                    joint.name
                );
            }
        }
    }

    // 11. build_rig with empty proxies returns rig with only the root joint
    #[test]
    fn build_rig_empty_proxies_returns_only_root() {
        let proxies = BodyProxies::new();
        let rig = build_rig(&proxies);
        assert_eq!(rig.joint_count(), 1);
        assert_eq!(rig.joints[0].name, "pelvis");
        assert!(rig.joints[0].parent.is_none());
    }

    // 12. CapsuleChain "spine" contains the head joint index
    #[test]
    fn spine_chain_contains_head() {
        let proxies = make_proxies();
        let rig = build_rig(&proxies);
        let chains = CapsuleChain::standard_chains(&rig);
        let spine = chains.iter().find(|c| c.name == "spine").unwrap();
        let head_idx = rig.find_joint("head").unwrap();
        assert!(
            spine.joint_indices.contains(&head_idx),
            "spine chain should contain head joint index {}; got {:?}",
            head_idx,
            spine.joint_indices
        );
    }
}
