#![allow(dead_code)]

//! Pose retargeting with joint mapping.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointMapping {
    pub source_name: String,
    pub target_name: String,
    pub scale: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseRetargetV2 {
    pub mappings: Vec<JointMapping>,
    pub default_scale: f32,
}

#[allow(dead_code)]
pub fn new_pose_retarget_v2() -> PoseRetargetV2 {
    PoseRetargetV2 {
        mappings: Vec::new(),
        default_scale: 1.0,
    }
}

#[allow(dead_code)]
pub fn prv2_add_mapping(retarget: &mut PoseRetargetV2, source: &str, target: &str, scale: f32) {
    retarget.mappings.push(JointMapping {
        source_name: source.to_string(),
        target_name: target.to_string(),
        scale: scale.max(0.0),
    });
}

#[allow(dead_code)]
pub fn prv2_retarget(
    retarget: &PoseRetargetV2,
    source: &std::collections::HashMap<String, [f32; 4]>,
    target: &mut std::collections::HashMap<String, [f32; 4]>,
) {
    for mapping in &retarget.mappings {
        if let Some(&pose) = source.get(&mapping.source_name) {
            let scaled = [
                pose[0] * mapping.scale,
                pose[1] * mapping.scale,
                pose[2] * mapping.scale,
                pose[3],
            ];
            target.insert(mapping.target_name.clone(), scaled);
        }
    }
}

#[allow(dead_code)]
pub fn prv2_mapping_count(retarget: &PoseRetargetV2) -> usize {
    retarget.mappings.len()
}

#[allow(dead_code)]
pub fn prv2_has_mapping(retarget: &PoseRetargetV2, source: &str) -> bool {
    retarget.mappings.iter().any(|m| m.source_name == source)
}

#[allow(dead_code)]
pub fn prv2_remove_mapping(retarget: &mut PoseRetargetV2, source: &str) {
    retarget.mappings.retain(|m| m.source_name != source);
}

#[allow(dead_code)]
pub fn prv2_clear(retarget: &mut PoseRetargetV2) {
    retarget.mappings.clear();
}

#[allow(dead_code)]
pub fn prv2_set_default_scale(retarget: &mut PoseRetargetV2, scale: f32) {
    retarget.default_scale = scale.max(0.0);
}

#[allow(dead_code)]
pub fn prv2_to_json(retarget: &PoseRetargetV2) -> String {
    format!(
        "{{\"mapping_count\":{},\"default_scale\":{}}}",
        retarget.mappings.len(),
        retarget.default_scale
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_new_retarget() {
        let r = new_pose_retarget_v2();
        assert_eq!(prv2_mapping_count(&r), 0);
    }

    #[test]
    fn test_add_mapping() {
        let mut r = new_pose_retarget_v2();
        prv2_add_mapping(&mut r, "left_arm", "L_arm", 1.0);
        assert_eq!(prv2_mapping_count(&r), 1);
    }

    #[test]
    fn test_has_mapping() {
        let mut r = new_pose_retarget_v2();
        prv2_add_mapping(&mut r, "spine", "Spine", 1.0);
        assert!(prv2_has_mapping(&r, "spine"));
        assert!(!prv2_has_mapping(&r, "nonexistent"));
    }

    #[test]
    fn test_retarget_applies() {
        let mut r = new_pose_retarget_v2();
        prv2_add_mapping(&mut r, "hip", "Hips", 1.0);
        let mut src = HashMap::new();
        src.insert("hip".to_string(), [0.1, 0.2, 0.3, 1.0]);
        let mut tgt = HashMap::new();
        prv2_retarget(&r, &src, &mut tgt);
        assert!(tgt.contains_key("Hips"));
    }

    #[test]
    fn test_retarget_with_scale() {
        let mut r = new_pose_retarget_v2();
        prv2_add_mapping(&mut r, "hip", "Hips", 2.0);
        let mut src = HashMap::new();
        src.insert("hip".to_string(), [1.0, 0.0, 0.0, 1.0]);
        let mut tgt = HashMap::new();
        prv2_retarget(&r, &src, &mut tgt);
        assert!((tgt["Hips"][0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_remove_mapping() {
        let mut r = new_pose_retarget_v2();
        prv2_add_mapping(&mut r, "arm", "Arm", 1.0);
        prv2_remove_mapping(&mut r, "arm");
        assert_eq!(prv2_mapping_count(&r), 0);
    }

    #[test]
    fn test_clear() {
        let mut r = new_pose_retarget_v2();
        prv2_add_mapping(&mut r, "a", "A", 1.0);
        prv2_clear(&mut r);
        assert_eq!(prv2_mapping_count(&r), 0);
    }

    #[test]
    fn test_set_default_scale() {
        let mut r = new_pose_retarget_v2();
        prv2_set_default_scale(&mut r, 1.5);
        assert!((r.default_scale - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let r = new_pose_retarget_v2();
        let json = prv2_to_json(&r);
        assert!(json.contains("mapping_count"));
    }

    #[test]
    fn test_retarget_missing_source() {
        let mut r = new_pose_retarget_v2();
        prv2_add_mapping(&mut r, "missing", "Target", 1.0);
        let src = HashMap::new();
        let mut tgt = HashMap::new();
        prv2_retarget(&r, &src, &mut tgt);
        assert!(!tgt.contains_key("Target"));
    }
}
