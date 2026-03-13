// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// An IK target export.
#[allow(dead_code)]
#[derive(Clone)]
pub struct IkTargetExport {
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4], // quaternion xyzw
    pub chain_length: usize,
    pub pole_target: Option<[f32; 3]>,
}

/// Bundle of IK targets.
#[allow(dead_code)]
#[derive(Default)]
pub struct IkTargetBundle {
    pub targets: Vec<IkTargetExport>,
}

/// Create a new IK target bundle.
#[allow(dead_code)]
pub fn new_ik_target_bundle() -> IkTargetBundle {
    IkTargetBundle::default()
}

/// Add an IK target.
#[allow(dead_code)]
pub fn add_ik_target(bundle: &mut IkTargetBundle, name: &str, pos: [f32; 3], chain_length: usize) {
    bundle.targets.push(IkTargetExport {
        name: name.to_string(),
        position: pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
        chain_length,
        pole_target: None,
    });
}

/// Set pole target.
#[allow(dead_code)]
pub fn set_pole_target(bundle: &mut IkTargetBundle, name: &str, pole: [f32; 3]) {
    for t in &mut bundle.targets {
        if t.name == name {
            t.pole_target = Some(pole);
        }
    }
}

/// Count IK targets.
#[allow(dead_code)]
pub fn ik_target_count(bundle: &IkTargetBundle) -> usize {
    bundle.targets.len()
}

/// Count targets with pole.
#[allow(dead_code)]
pub fn targets_with_pole(bundle: &IkTargetBundle) -> usize {
    bundle
        .targets
        .iter()
        .filter(|t| t.pole_target.is_some())
        .count()
}

/// Find target by name.
#[allow(dead_code)]
pub fn find_ik_target<'a>(bundle: &'a IkTargetBundle, name: &str) -> Option<&'a IkTargetExport> {
    bundle.targets.iter().find(|t| t.name == name)
}

/// Average chain length.
#[allow(dead_code)]
pub fn avg_chain_length(bundle: &IkTargetBundle) -> f32 {
    if bundle.targets.is_empty() {
        return 0.0;
    }
    bundle
        .targets
        .iter()
        .map(|t| t.chain_length as f32)
        .sum::<f32>()
        / bundle.targets.len() as f32
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn ik_target_to_json(bundle: &IkTargetBundle) -> String {
    format!(
        r#"{{"ik_targets":{},"with_pole":{}}}"#,
        bundle.targets.len(),
        targets_with_pole(bundle)
    )
}

/// Validate targets (chain length > 0).
#[allow(dead_code)]
pub fn validate_ik_targets(bundle: &IkTargetBundle) -> bool {
    bundle.targets.iter().all(|t| t.chain_length > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut b = new_ik_target_bundle();
        add_ik_target(&mut b, "hand_l", [0.0; 3], 3);
        assert_eq!(ik_target_count(&b), 1);
    }

    #[test]
    fn set_and_count_pole() {
        let mut b = new_ik_target_bundle();
        add_ik_target(&mut b, "arm", [0.0; 3], 2);
        set_pole_target(&mut b, "arm", [1.0, 0.0, 0.0]);
        assert_eq!(targets_with_pole(&b), 1);
    }

    #[test]
    fn find_target() {
        let mut b = new_ik_target_bundle();
        add_ik_target(&mut b, "leg", [0.0; 3], 3);
        assert!(find_ik_target(&b, "leg").is_some());
    }

    #[test]
    fn find_missing() {
        let b = new_ik_target_bundle();
        assert!(find_ik_target(&b, "x").is_none());
    }

    #[test]
    fn avg_chain_len_test() {
        let mut b = new_ik_target_bundle();
        add_ik_target(&mut b, "a", [0.0; 3], 2);
        add_ik_target(&mut b, "b", [0.0; 3], 4);
        assert!((avg_chain_length(&b) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn json_has_count() {
        let b = new_ik_target_bundle();
        let j = ik_target_to_json(&b);
        assert!(j.contains("\"ik_targets\":0"));
    }

    #[test]
    fn validate_valid() {
        let mut b = new_ik_target_bundle();
        add_ik_target(&mut b, "x", [0.0; 3], 2);
        assert!(validate_ik_targets(&b));
    }

    #[test]
    fn validate_zero_chain_fails() {
        let mut b = new_ik_target_bundle();
        add_ik_target(&mut b, "x", [0.0; 3], 0);
        assert!(!validate_ik_targets(&b));
    }

    #[test]
    fn default_rotation_identity() {
        let mut b = new_ik_target_bundle();
        add_ik_target(&mut b, "x", [0.0; 3], 1);
        assert_eq!(b.targets[0].rotation[3], 1.0);
    }

    #[test]
    fn no_pole_by_default() {
        let mut b = new_ik_target_bundle();
        add_ik_target(&mut b, "x", [0.0; 3], 1);
        assert!(b.targets[0].pole_target.is_none());
    }
}
