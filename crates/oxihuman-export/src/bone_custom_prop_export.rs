// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A custom property value.
#[allow(dead_code)]
#[derive(Clone)]
pub enum BonePropValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Text(String),
}

/// A custom property entry on a bone.
#[allow(dead_code)]
pub struct BoneCustomProp {
    pub bone_name: String,
    pub key: String,
    pub value: BonePropValue,
}

/// Export bundle for bone custom properties.
#[allow(dead_code)]
#[derive(Default)]
pub struct BoneCustomPropExport {
    pub props: Vec<BoneCustomProp>,
}

/// Create a new bone custom prop export.
#[allow(dead_code)]
pub fn new_bone_custom_prop_export() -> BoneCustomPropExport {
    BoneCustomPropExport::default()
}

/// Add a float property.
#[allow(dead_code)]
pub fn add_float_prop(export: &mut BoneCustomPropExport, bone: &str, key: &str, val: f32) {
    export.props.push(BoneCustomProp {
        bone_name: bone.to_string(),
        key: key.to_string(),
        value: BonePropValue::Float(val),
    });
}

/// Add a bool property.
#[allow(dead_code)]
pub fn add_bool_prop(export: &mut BoneCustomPropExport, bone: &str, key: &str, val: bool) {
    export.props.push(BoneCustomProp {
        bone_name: bone.to_string(),
        key: key.to_string(),
        value: BonePropValue::Bool(val),
    });
}

/// Add a text property.
#[allow(dead_code)]
pub fn add_text_prop(export: &mut BoneCustomPropExport, bone: &str, key: &str, val: &str) {
    export.props.push(BoneCustomProp {
        bone_name: bone.to_string(),
        key: key.to_string(),
        value: BonePropValue::Text(val.to_string()),
    });
}

/// Count total properties.
#[allow(dead_code)]
pub fn prop_count(export: &BoneCustomPropExport) -> usize {
    export.props.len()
}

/// Find a property by bone name and key.
#[allow(dead_code)]
pub fn find_prop<'a>(
    export: &'a BoneCustomPropExport,
    bone: &str,
    key: &str,
) -> Option<&'a BoneCustomProp> {
    export
        .props
        .iter()
        .find(|p| p.bone_name == bone && p.key == key)
}

/// Props for a given bone.
#[allow(dead_code)]
pub fn props_for_bone<'a>(export: &'a BoneCustomPropExport, bone: &str) -> Vec<&'a BoneCustomProp> {
    export
        .props
        .iter()
        .filter(|p| p.bone_name == bone)
        .collect()
}

/// Unique bone names with properties.
#[allow(dead_code)]
pub fn bone_names_with_props(export: &BoneCustomPropExport) -> Vec<String> {
    let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for p in &export.props {
        names.insert(p.bone_name.clone());
    }
    let mut v: Vec<_> = names.into_iter().collect();
    v.sort();
    v
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn bone_custom_prop_to_json(export: &BoneCustomPropExport) -> String {
    format!(r#"{{"custom_props":{}}}"#, export.props.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_float_and_count() {
        let mut e = new_bone_custom_prop_export();
        add_float_prop(&mut e, "spine", "stiffness", 0.5);
        assert_eq!(prop_count(&e), 1);
    }

    #[test]
    fn find_prop_found() {
        let mut e = new_bone_custom_prop_export();
        add_float_prop(&mut e, "arm", "twist", 0.3);
        assert!(find_prop(&e, "arm", "twist").is_some());
    }

    #[test]
    fn find_prop_missing() {
        let e = new_bone_custom_prop_export();
        assert!(find_prop(&e, "arm", "twist").is_none());
    }

    #[test]
    fn props_for_bone_filtered() {
        let mut e = new_bone_custom_prop_export();
        add_float_prop(&mut e, "arm", "a", 1.0);
        add_float_prop(&mut e, "leg", "b", 2.0);
        assert_eq!(props_for_bone(&e, "arm").len(), 1);
    }

    #[test]
    fn bone_names_unique_sorted() {
        let mut e = new_bone_custom_prop_export();
        add_float_prop(&mut e, "b", "x", 1.0);
        add_float_prop(&mut e, "a", "y", 1.0);
        let names = bone_names_with_props(&e);
        assert_eq!(names[0], "a");
    }

    #[test]
    fn add_bool_prop_ok() {
        let mut e = new_bone_custom_prop_export();
        add_bool_prop(&mut e, "head", "ik_enabled", true);
        assert_eq!(prop_count(&e), 1);
    }

    #[test]
    fn add_text_prop_ok() {
        let mut e = new_bone_custom_prop_export();
        add_text_prop(&mut e, "root", "group", "physics");
        assert!(find_prop(&e, "root", "group").is_some());
    }

    #[test]
    fn json_has_count() {
        let mut e = new_bone_custom_prop_export();
        add_float_prop(&mut e, "x", "y", 1.0);
        let j = bone_custom_prop_to_json(&e);
        assert!(j.contains("\"custom_props\":1"));
    }

    #[test]
    fn empty_export() {
        let e = new_bone_custom_prop_export();
        assert_eq!(prop_count(&e), 0);
    }

    #[test]
    fn multiple_props_same_bone() {
        let mut e = new_bone_custom_prop_export();
        add_float_prop(&mut e, "arm", "a", 1.0);
        add_float_prop(&mut e, "arm", "b", 2.0);
        assert_eq!(props_for_bone(&e, "arm").len(), 2);
    }
}
