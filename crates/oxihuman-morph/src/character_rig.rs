// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// RigJoint
// ---------------------------------------------------------------------------

/// A joint definition in the rig.
#[derive(Clone, Debug)]
pub struct RigJoint {
    pub name: String,
    pub parent: Option<String>,
    /// Bind pose position (world space).
    pub position: [f32; 3],
    /// Bind pose rotation (axis-angle: \[ax, ay, az, angle\]).
    pub rotation: [f32; 4],
    /// Scale.
    pub scale: [f32; 3],
}

impl RigJoint {
    /// Create a new joint with default identity pose.
    pub fn new(name: impl Into<String>) -> Self {
        RigJoint {
            name: name.into(),
            parent: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 1.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    /// Set the parent joint name.
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Set the bind-pose position.
    pub fn with_position(mut self, pos: [f32; 3]) -> Self {
        self.position = pos;
        self
    }

    /// Return references to all joints in `joints` whose parent is `parent`.
    pub fn children_of<'a>(joints: &'a [RigJoint], parent: &str) -> Vec<&'a RigJoint> {
        joints
            .iter()
            .filter(|j| j.parent.as_deref() == Some(parent))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// MorphBinding
// ---------------------------------------------------------------------------

/// Binding between a morph target and its driving parameters.
#[derive(Clone, Debug)]
pub struct MorphBinding {
    pub morph_name: String,
    /// Which parameter drives it.
    pub param_name: String,
    /// Linear map: weight = (param - input_min) / (input_max - input_min).
    pub input_min: f32,
    pub input_max: f32,
    /// Optional joint that also contributes to this morph.
    pub joint_driver: Option<String>,
}

impl MorphBinding {
    /// Create a new binding with default range [0.0, 1.0].
    pub fn new(morph: impl Into<String>, param: impl Into<String>) -> Self {
        MorphBinding {
            morph_name: morph.into(),
            param_name: param.into(),
            input_min: 0.0,
            input_max: 1.0,
            joint_driver: None,
        }
    }

    /// Compute the morph weight for a given parameter value.
    pub fn compute_weight(&self, param_value: f32) -> f32 {
        ((param_value - self.input_min) / (self.input_max - self.input_min)).clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// RigBodyRegion
// ---------------------------------------------------------------------------

/// A named region of the body (e.g., "upper_arm", "torso").
#[derive(Clone, Debug)]
pub struct RigBodyRegion {
    pub name: String,
    /// Joints that belong to this region.
    pub joints: Vec<String>,
    /// Morph targets relevant to this region.
    pub morph_targets: Vec<String>,
    /// Vertex group name.
    pub vertex_group: Option<String>,
}

// ---------------------------------------------------------------------------
// CharacterRig
// ---------------------------------------------------------------------------

/// The complete character rig.
pub struct CharacterRig {
    pub name: String,
    pub joints: Vec<RigJoint>,
    pub morph_bindings: Vec<MorphBinding>,
    pub regions: Vec<RigBodyRegion>,
    pub metadata: HashMap<String, String>,
}

impl CharacterRig {
    /// Create a new, empty rig.
    pub fn new(name: impl Into<String>) -> Self {
        CharacterRig {
            name: name.into(),
            joints: Vec::new(),
            morph_bindings: Vec::new(),
            regions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a joint to the rig.
    pub fn add_joint(&mut self, joint: RigJoint) {
        self.joints.push(joint);
    }

    /// Add a morph binding.
    pub fn add_morph_binding(&mut self, binding: MorphBinding) {
        self.morph_bindings.push(binding);
    }

    /// Add a body region.
    pub fn add_region(&mut self, region: RigBodyRegion) {
        self.regions.push(region);
    }

    /// Number of joints in the rig.
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    /// Number of morph bindings.
    pub fn morph_binding_count(&self) -> usize {
        self.morph_bindings.len()
    }

    /// Look up a joint by name.
    pub fn get_joint(&self, name: &str) -> Option<&RigJoint> {
        self.joints.iter().find(|j| j.name == name)
    }

    /// Return all joints that have no parent (i.e. root joints).
    pub fn root_joints(&self) -> Vec<&RigJoint> {
        self.joints.iter().filter(|j| j.parent.is_none()).collect()
    }

    /// Return all joints whose parent is `parent`.
    pub fn children_of(&self, parent: &str) -> Vec<&RigJoint> {
        RigJoint::children_of(&self.joints, parent)
    }

    /// Return the depth of `name` in the joint hierarchy (root = 0).
    pub fn joint_depth(&self, name: &str) -> usize {
        let mut depth = 0;
        let mut current = name.to_string();
        loop {
            match self.get_joint(&current) {
                None => break,
                Some(j) => match &j.parent {
                    None => break,
                    Some(p) => {
                        depth += 1;
                        current = p.clone();
                    }
                },
            }
        }
        depth
    }

    /// Return all bindings driven by `param`.
    pub fn bindings_for_param(&self, param: &str) -> Vec<&MorphBinding> {
        self.morph_bindings
            .iter()
            .filter(|b| b.param_name == param)
            .collect()
    }

    /// Return all bindings that have `joint` as their joint driver.
    pub fn bindings_for_joint(&self, joint: &str) -> Vec<&MorphBinding> {
        self.morph_bindings
            .iter()
            .filter(|b| b.joint_driver.as_deref() == Some(joint))
            .collect()
    }

    /// Evaluate all morph weights given the current parameter state.
    pub fn evaluate_morphs(&self, params: &HashMap<String, f32>) -> HashMap<String, f32> {
        let mut result = HashMap::new();
        for binding in &self.morph_bindings {
            let param_value = params.get(&binding.param_name).copied().unwrap_or(0.0);
            let weight = binding.compute_weight(param_value);
            result.insert(binding.morph_name.clone(), weight);
        }
        result
    }

    /// Serialize the rig to a JSON string.
    pub fn to_json(&self) -> String {
        let joints_json: Vec<serde_json::Value> = self
            .joints
            .iter()
            .map(|j| {
                serde_json::json!({
                    "name": j.name,
                    "parent": j.parent,
                    "position": j.position,
                    "rotation": j.rotation,
                    "scale": j.scale,
                })
            })
            .collect();

        let bindings_json: Vec<serde_json::Value> = self
            .morph_bindings
            .iter()
            .map(|b| {
                serde_json::json!({
                    "morph_name": b.morph_name,
                    "param_name": b.param_name,
                    "input_min": b.input_min,
                    "input_max": b.input_max,
                    "joint_driver": b.joint_driver,
                })
            })
            .collect();

        let regions_json: Vec<serde_json::Value> = self
            .regions
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "joints": r.joints,
                    "morph_targets": r.morph_targets,
                    "vertex_group": r.vertex_group,
                })
            })
            .collect();

        let obj = serde_json::json!({
            "name": self.name,
            "joints": joints_json,
            "morph_bindings": bindings_json,
            "regions": regions_json,
            "metadata": self.metadata,
        });

        obj.to_string()
    }
}

// ---------------------------------------------------------------------------
// standard_human_rig  (~22 joints, hm08 skeleton approximation)
// ---------------------------------------------------------------------------

/// Build a standard human rig (hm08 skeleton approximation, ~22 joints).
pub fn standard_human_rig() -> CharacterRig {
    let mut rig = CharacterRig::new("standard_human");
    rig.metadata
        .insert("source".to_string(), "hm08_approximation".to_string());
    rig.metadata
        .insert("version".to_string(), "1.0".to_string());

    // Root
    rig.add_joint(RigJoint::new("hips").with_position([0.0, 1.0, 0.0]));

    // Spine chain
    rig.add_joint(
        RigJoint::new("spine1")
            .with_parent("hips")
            .with_position([0.0, 1.1, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("spine2")
            .with_parent("spine1")
            .with_position([0.0, 1.3, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("spine3")
            .with_parent("spine2")
            .with_position([0.0, 1.5, 0.0]),
    );

    // Neck and head
    rig.add_joint(
        RigJoint::new("neck")
            .with_parent("spine3")
            .with_position([0.0, 1.6, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("head")
            .with_parent("neck")
            .with_position([0.0, 1.75, 0.0]),
    );

    // Left arm chain
    rig.add_joint(
        RigJoint::new("l_shoulder")
            .with_parent("spine3")
            .with_position([-0.2, 1.5, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("l_elbow")
            .with_parent("l_shoulder")
            .with_position([-0.45, 1.2, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("l_wrist")
            .with_parent("l_elbow")
            .with_position([-0.65, 0.95, 0.0]),
    );

    // Right arm chain
    rig.add_joint(
        RigJoint::new("r_shoulder")
            .with_parent("spine3")
            .with_position([0.2, 1.5, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("r_elbow")
            .with_parent("r_shoulder")
            .with_position([0.45, 1.2, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("r_wrist")
            .with_parent("r_elbow")
            .with_position([0.65, 0.95, 0.0]),
    );

    // Left leg chain
    rig.add_joint(
        RigJoint::new("l_hip")
            .with_parent("hips")
            .with_position([-0.1, 0.9, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("l_knee")
            .with_parent("l_hip")
            .with_position([-0.1, 0.55, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("l_ankle")
            .with_parent("l_knee")
            .with_position([-0.1, 0.1, 0.0]),
    );

    // Right leg chain
    rig.add_joint(
        RigJoint::new("r_hip")
            .with_parent("hips")
            .with_position([0.1, 0.9, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("r_knee")
            .with_parent("r_hip")
            .with_position([0.1, 0.55, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("r_ankle")
            .with_parent("r_knee")
            .with_position([0.1, 0.1, 0.0]),
    );

    // Left hand fingers (representative)
    rig.add_joint(
        RigJoint::new("l_hand")
            .with_parent("l_wrist")
            .with_position([-0.7, 0.85, 0.0]),
    );
    // Right hand fingers (representative)
    rig.add_joint(
        RigJoint::new("r_hand")
            .with_parent("r_wrist")
            .with_position([0.7, 0.85, 0.0]),
    );

    // Left foot
    rig.add_joint(
        RigJoint::new("l_foot")
            .with_parent("l_ankle")
            .with_position([-0.1, 0.0, 0.1]),
    );
    // Right foot
    rig.add_joint(
        RigJoint::new("r_foot")
            .with_parent("r_ankle")
            .with_position([0.1, 0.0, 0.1]),
    );

    // Sample morph bindings
    rig.add_morph_binding(MorphBinding::new("head_size", "head_scale"));
    rig.add_morph_binding(MorphBinding::new("body_weight", "weight"));
    rig.add_morph_binding(MorphBinding::new("muscle_tone", "muscle"));

    // Body regions
    rig.add_region(RigBodyRegion {
        name: "head_region".to_string(),
        joints: vec!["neck".to_string(), "head".to_string()],
        morph_targets: vec!["head_size".to_string()],
        vertex_group: Some("head_vg".to_string()),
    });
    rig.add_region(RigBodyRegion {
        name: "torso_region".to_string(),
        joints: vec![
            "spine1".to_string(),
            "spine2".to_string(),
            "spine3".to_string(),
        ],
        morph_targets: vec!["body_weight".to_string(), "muscle_tone".to_string()],
        vertex_group: Some("torso_vg".to_string()),
    });

    rig
}

// ---------------------------------------------------------------------------
// minimal_human_rig  (16 joints)
// ---------------------------------------------------------------------------

/// Build a simplified 16-joint rig (head, spine, limbs).
pub fn minimal_human_rig() -> CharacterRig {
    let mut rig = CharacterRig::new("minimal_human");
    rig.metadata
        .insert("source".to_string(), "minimal_16".to_string());

    // Root
    rig.add_joint(RigJoint::new("pelvis").with_position([0.0, 1.0, 0.0]));

    // Spine chain
    rig.add_joint(
        RigJoint::new("spine")
            .with_parent("pelvis")
            .with_position([0.0, 1.2, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("chest")
            .with_parent("spine")
            .with_position([0.0, 1.45, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("head")
            .with_parent("chest")
            .with_position([0.0, 1.75, 0.0]),
    );

    // Left arm
    rig.add_joint(
        RigJoint::new("l_upper_arm")
            .with_parent("chest")
            .with_position([-0.2, 1.45, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("l_forearm")
            .with_parent("l_upper_arm")
            .with_position([-0.4, 1.2, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("l_hand")
            .with_parent("l_forearm")
            .with_position([-0.6, 0.95, 0.0]),
    );

    // Right arm
    rig.add_joint(
        RigJoint::new("r_upper_arm")
            .with_parent("chest")
            .with_position([0.2, 1.45, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("r_forearm")
            .with_parent("r_upper_arm")
            .with_position([0.4, 1.2, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("r_hand")
            .with_parent("r_forearm")
            .with_position([0.6, 0.95, 0.0]),
    );

    // Left leg
    rig.add_joint(
        RigJoint::new("l_thigh")
            .with_parent("pelvis")
            .with_position([-0.1, 0.9, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("l_shin")
            .with_parent("l_thigh")
            .with_position([-0.1, 0.5, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("l_foot")
            .with_parent("l_shin")
            .with_position([-0.1, 0.05, 0.0]),
    );

    // Right leg
    rig.add_joint(
        RigJoint::new("r_thigh")
            .with_parent("pelvis")
            .with_position([0.1, 0.9, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("r_shin")
            .with_parent("r_thigh")
            .with_position([0.1, 0.5, 0.0]),
    );
    rig.add_joint(
        RigJoint::new("r_foot")
            .with_parent("r_shin")
            .with_position([0.1, 0.05, 0.0]),
    );

    // Sample morph bindings
    rig.add_morph_binding(MorphBinding::new("head_size", "head_scale"));
    rig.add_morph_binding(MorphBinding::new("body_weight", "weight"));

    rig
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rig_joint_new() {
        let j = RigJoint::new("hips");
        assert_eq!(j.name, "hips");
        assert!(j.parent.is_none());
        assert_eq!(j.position, [0.0, 0.0, 0.0]);
        assert_eq!(j.rotation, [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(j.scale, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_rig_joint_with_parent() {
        let j = RigJoint::new("spine").with_parent("hips");
        assert_eq!(j.parent.as_deref(), Some("hips"));
    }

    #[test]
    fn test_morph_binding_compute_weight() {
        let b = MorphBinding::new("fat_belly", "weight");
        // At mid-range (0.5 with default [0, 1])
        let w = b.compute_weight(0.5);
        assert!((w - 0.5).abs() < 1e-6);
        // At input_max
        assert!((b.compute_weight(1.0) - 1.0).abs() < 1e-6);
        // At input_min
        assert!((b.compute_weight(0.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_morph_binding_clamped() {
        let b = MorphBinding::new("fat_belly", "weight");
        // Beyond max → clamped to 1.0
        assert!((b.compute_weight(2.0) - 1.0).abs() < 1e-6);
        // Below min → clamped to 0.0
        assert!((b.compute_weight(-1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_character_rig_new() {
        let rig = CharacterRig::new("test_rig");
        assert_eq!(rig.name, "test_rig");
        assert_eq!(rig.joint_count(), 0);
        assert_eq!(rig.morph_binding_count(), 0);
        assert!(rig.regions.is_empty());
        assert!(rig.metadata.is_empty());
    }

    #[test]
    fn test_add_joint() {
        let mut rig = CharacterRig::new("rig");
        rig.add_joint(RigJoint::new("hips"));
        rig.add_joint(RigJoint::new("spine").with_parent("hips"));
        assert_eq!(rig.joint_count(), 2);
    }

    #[test]
    fn test_get_joint() {
        let mut rig = CharacterRig::new("rig");
        rig.add_joint(RigJoint::new("hips"));
        assert!(rig.get_joint("hips").is_some());
        assert!(rig.get_joint("nonexistent").is_none());
    }

    #[test]
    fn test_root_joints() {
        let mut rig = CharacterRig::new("rig");
        rig.add_joint(RigJoint::new("hips"));
        rig.add_joint(RigJoint::new("spine").with_parent("hips"));
        rig.add_joint(RigJoint::new("chest").with_parent("spine"));
        let roots = rig.root_joints();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "hips");
    }

    #[test]
    fn test_children_of() {
        let mut rig = CharacterRig::new("rig");
        rig.add_joint(RigJoint::new("hips"));
        rig.add_joint(RigJoint::new("l_hip").with_parent("hips"));
        rig.add_joint(RigJoint::new("r_hip").with_parent("hips"));
        rig.add_joint(RigJoint::new("spine").with_parent("hips"));
        let children = rig.children_of("hips");
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn test_joint_depth() {
        let mut rig = CharacterRig::new("rig");
        rig.add_joint(RigJoint::new("hips"));
        rig.add_joint(RigJoint::new("spine").with_parent("hips"));
        rig.add_joint(RigJoint::new("chest").with_parent("spine"));
        rig.add_joint(RigJoint::new("neck").with_parent("chest"));
        rig.add_joint(RigJoint::new("head").with_parent("neck"));

        assert_eq!(rig.joint_depth("hips"), 0);
        assert_eq!(rig.joint_depth("spine"), 1);
        assert_eq!(rig.joint_depth("chest"), 2);
        assert_eq!(rig.joint_depth("neck"), 3);
        assert_eq!(rig.joint_depth("head"), 4);
    }

    #[test]
    fn test_bindings_for_param() {
        let mut rig = CharacterRig::new("rig");
        rig.add_morph_binding(MorphBinding::new("fat_belly", "weight"));
        rig.add_morph_binding(MorphBinding::new("fat_arms", "weight"));
        rig.add_morph_binding(MorphBinding::new("muscle", "muscle"));
        let by_weight = rig.bindings_for_param("weight");
        assert_eq!(by_weight.len(), 2);
        let by_muscle = rig.bindings_for_param("muscle");
        assert_eq!(by_muscle.len(), 1);
    }

    #[test]
    fn test_evaluate_morphs() {
        let mut rig = CharacterRig::new("rig");
        rig.add_morph_binding(MorphBinding::new("fat_belly", "weight"));
        rig.add_morph_binding(MorphBinding::new("muscle", "muscle"));

        let mut params = HashMap::new();
        params.insert("weight".to_string(), 0.75_f32);
        // "muscle" not in params → defaults to 0.0

        let weights = rig.evaluate_morphs(&params);
        assert!((weights["fat_belly"] - 0.75).abs() < 1e-6);
        assert!((weights["muscle"] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let mut rig = CharacterRig::new("test_rig");
        rig.add_joint(RigJoint::new("hips"));
        rig.add_morph_binding(MorphBinding::new("fat_belly", "weight"));
        let json = rig.to_json();
        assert!(json.contains("test_rig"));
        assert!(json.contains("hips"));
        assert!(json.contains("fat_belly"));
        assert!(json.contains("weight"));
    }

    #[test]
    fn test_standard_human_rig() {
        let rig = standard_human_rig();
        assert_eq!(rig.name, "standard_human");
        // Should have ~22 joints
        assert!(rig.joint_count() >= 20);
        assert!(rig.morph_binding_count() > 0);
        // hips should be root
        let roots = rig.root_joints();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "hips");
        // verify depth of head
        assert!(rig.joint_depth("head") >= 2);
    }

    #[test]
    fn test_minimal_human_rig() {
        let rig = minimal_human_rig();
        assert_eq!(rig.name, "minimal_human");
        assert_eq!(rig.joint_count(), 16);
        // pelvis is root
        let roots = rig.root_joints();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "pelvis");
        // head depth: pelvis -> spine -> chest -> head = 3
        assert_eq!(rig.joint_depth("head"), 3);
        // bindings exist
        assert!(rig.morph_binding_count() >= 2);
    }
}
