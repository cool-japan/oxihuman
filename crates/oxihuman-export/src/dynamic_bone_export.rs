// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Export dynamic bone / spring bone chain data.
#[allow(dead_code)]
pub struct DynamicBone {
    pub name: String,
    pub root_bone: String,
    pub stiffness: f32,
    pub damping: f32,
    pub gravity: [f32; 3],
    pub radius: f32,
    pub end_length: f32,
    pub colliders: Vec<String>,
}

#[allow(dead_code)]
pub struct DynamicBoneExport {
    pub chains: Vec<DynamicBone>,
}

#[allow(dead_code)]
pub fn new_dynamic_bone_export() -> DynamicBoneExport {
    DynamicBoneExport { chains: vec![] }
}

#[allow(dead_code)]
pub fn add_dynamic_bone(export: &mut DynamicBoneExport, bone: DynamicBone) {
    export.chains.push(bone);
}

#[allow(dead_code)]
pub fn dynamic_bone_count(export: &DynamicBoneExport) -> usize {
    export.chains.len()
}

#[allow(dead_code)]
pub fn find_dynamic_bone<'a>(export: &'a DynamicBoneExport, name: &str) -> Option<&'a DynamicBone> {
    export.chains.iter().find(|b| b.name == name)
}

#[allow(dead_code)]
pub fn default_dynamic_bone(name: &str, root: &str) -> DynamicBone {
    DynamicBone {
        name: name.to_string(),
        root_bone: root.to_string(),
        stiffness: 0.2,
        damping: 0.2,
        gravity: [0.0, -9.81, 0.0],
        radius: 0.02,
        end_length: 0.0,
        colliders: vec![],
    }
}

#[allow(dead_code)]
pub fn validate_dynamic_bone(bone: &DynamicBone) -> bool {
    !bone.name.is_empty()
        && !bone.root_bone.is_empty()
        && (0.0..=1.0).contains(&bone.stiffness)
        && (0.0..=1.0).contains(&bone.damping)
        && bone.radius >= 0.0
}

#[allow(dead_code)]
pub fn total_colliders(export: &DynamicBoneExport) -> usize {
    export.chains.iter().map(|b| b.colliders.len()).sum()
}

#[allow(dead_code)]
pub fn dynamic_bone_to_json(bone: &DynamicBone) -> String {
    format!(
        "{{\"name\":\"{}\",\"root\":\"{}\",\"stiffness\":{},\"damping\":{}}}",
        bone.name, bone.root_bone, bone.stiffness, bone.damping
    )
}

#[allow(dead_code)]
pub fn dynamic_bone_export_to_json(export: &DynamicBoneExport) -> String {
    format!("{{\"chain_count\":{}}}", export.chains.len())
}

#[allow(dead_code)]
pub fn simulate_gravity_offset(bone: &DynamicBone, dt: f32) -> [f32; 3] {
    let g = bone.gravity;
    let stiffness = bone.stiffness.clamp(0.0, 1.0);
    let influence = (1.0 - stiffness) * dt;
    [g[0] * influence, g[1] * influence, g[2] * influence]
}

#[allow(dead_code)]
pub fn bones_with_colliders(export: &DynamicBoneExport) -> Vec<&DynamicBone> {
    export
        .chains
        .iter()
        .filter(|b| !b.colliders.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hair_bone() -> DynamicBone {
        default_dynamic_bone("hair_chain", "hair_root")
    }

    #[test]
    fn test_add_dynamic_bone() {
        let mut e = new_dynamic_bone_export();
        add_dynamic_bone(&mut e, hair_bone());
        assert_eq!(dynamic_bone_count(&e), 1);
    }

    #[test]
    fn test_find_dynamic_bone() {
        let mut e = new_dynamic_bone_export();
        add_dynamic_bone(&mut e, hair_bone());
        assert!(find_dynamic_bone(&e, "hair_chain").is_some());
    }

    #[test]
    fn test_validate_default() {
        let b = hair_bone();
        assert!(validate_dynamic_bone(&b));
    }

    #[test]
    fn test_validate_bad_stiffness() {
        let mut b = hair_bone();
        b.stiffness = 2.0;
        assert!(!validate_dynamic_bone(&b));
    }

    #[test]
    fn test_gravity_offset_nonzero() {
        let b = hair_bone();
        let off = simulate_gravity_offset(&b, 0.016);
        assert!(off[1] < 0.0);
    }

    #[test]
    fn test_bones_with_colliders_empty() {
        let mut e = new_dynamic_bone_export();
        add_dynamic_bone(&mut e, hair_bone());
        assert_eq!(bones_with_colliders(&e).len(), 0);
    }

    #[test]
    fn test_bones_with_colliders_found() {
        let mut e = new_dynamic_bone_export();
        let mut b = hair_bone();
        b.colliders.push("head_collider".to_string());
        add_dynamic_bone(&mut e, b);
        assert_eq!(bones_with_colliders(&e).len(), 1);
    }

    #[test]
    fn test_to_json() {
        let b = hair_bone();
        let j = dynamic_bone_to_json(&b);
        assert!(j.contains("hair_chain"));
    }

    #[test]
    fn test_export_to_json() {
        let mut e = new_dynamic_bone_export();
        add_dynamic_bone(&mut e, hair_bone());
        let j = dynamic_bone_export_to_json(&e);
        assert!(j.contains("chain_count"));
    }

    #[test]
    fn test_total_colliders() {
        let mut e = new_dynamic_bone_export();
        let mut b = hair_bone();
        b.colliders = vec!["c1".to_string(), "c2".to_string()];
        add_dynamic_bone(&mut e, b);
        assert_eq!(total_colliders(&e), 2);
    }
}
