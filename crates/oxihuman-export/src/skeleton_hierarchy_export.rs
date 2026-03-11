#![allow(dead_code)]
//! Skeleton hierarchy export.

/// Skeleton hierarchy export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonHierarchyExport {
    pub bones: Vec<BoneData>,
}

/// Bone data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BoneData {
    pub name: String,
    pub parent_index: Option<usize>,
    pub local_transform: [f32; 16],
}

/// Export skeleton hierarchy.
#[allow(dead_code)]
pub fn export_skeleton_hierarchy(bones: Vec<BoneData>) -> SkeletonHierarchyExport {
    SkeletonHierarchyExport { bones }
}

/// Get bone count.
#[allow(dead_code)]
pub fn bone_count_she(e: &SkeletonHierarchyExport) -> usize {
    e.bones.len()
}

/// Get bone name.
#[allow(dead_code)]
pub fn bone_name_she(e: &SkeletonHierarchyExport, index: usize) -> &str {
    if index < e.bones.len() {
        &e.bones[index].name
    } else {
        ""
    }
}

/// Get bone parent index.
#[allow(dead_code)]
pub fn bone_parent_index(e: &SkeletonHierarchyExport, index: usize) -> Option<usize> {
    if index < e.bones.len() {
        e.bones[index].parent_index
    } else {
        None
    }
}

/// Serialize bone to JSON.
#[allow(dead_code)]
pub fn bone_to_json(e: &SkeletonHierarchyExport, index: usize) -> String {
    if index < e.bones.len() {
        let b = &e.bones[index];
        format!("{{\"name\":\"{}\",\"parent\":{:?}}}", b.name, b.parent_index)
    } else {
        "{}".to_string()
    }
}

/// Get bone local transform.
#[allow(dead_code)]
pub fn bone_local_transform(e: &SkeletonHierarchyExport, index: usize) -> [f32; 16] {
    if index < e.bones.len() {
        e.bones[index].local_transform
    } else {
        let mut m = [0.0_f32; 16];
        m[0] = 1.0;
        m[5] = 1.0;
        m[10] = 1.0;
        m[15] = 1.0;
        m
    }
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn skeleton_export_size(e: &SkeletonHierarchyExport) -> usize {
    e.bones.len() * (64 + 32) // transform + name estimate
}

/// Validate skeleton.
#[allow(dead_code)]
pub fn validate_skeleton(e: &SkeletonHierarchyExport) -> bool {
    for (i, b) in e.bones.iter().enumerate() {
        if b.name.is_empty() {
            return false;
        }
        if let Some(parent) = b.parent_index {
            if parent >= i {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> [f32; 16] {
        let mut m = [0.0; 16];
        m[0] = 1.0; m[5] = 1.0; m[10] = 1.0; m[15] = 1.0;
        m
    }

    fn bone(name: &str, parent: Option<usize>) -> BoneData {
        BoneData { name: name.to_string(), parent_index: parent, local_transform: identity() }
    }

    #[test]
    fn test_export_skeleton() {
        let e = export_skeleton_hierarchy(vec![bone("root", None)]);
        assert_eq!(e.bones.len(), 1);
    }

    #[test]
    fn test_bone_count() {
        let e = export_skeleton_hierarchy(vec![bone("a", None), bone("b", Some(0))]);
        assert_eq!(bone_count_she(&e), 2);
    }

    #[test]
    fn test_bone_name() {
        let e = export_skeleton_hierarchy(vec![bone("hip", None)]);
        assert_eq!(bone_name_she(&e, 0), "hip");
        assert_eq!(bone_name_she(&e, 5), "");
    }

    #[test]
    fn test_bone_parent_index() {
        let e = export_skeleton_hierarchy(vec![bone("a", None), bone("b", Some(0))]);
        assert_eq!(bone_parent_index(&e, 0), None);
        assert_eq!(bone_parent_index(&e, 1), Some(0));
    }

    #[test]
    fn test_bone_to_json() {
        let e = export_skeleton_hierarchy(vec![bone("a", None)]);
        let j = bone_to_json(&e, 0);
        assert!(j.contains("name"));
    }

    #[test]
    fn test_bone_local_transform() {
        let e = export_skeleton_hierarchy(vec![bone("a", None)]);
        let t = bone_local_transform(&e, 0);
        assert!((t[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bone_local_transform_oob() {
        let e = export_skeleton_hierarchy(vec![]);
        let t = bone_local_transform(&e, 0);
        assert!((t[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_skeleton_export_size() {
        let e = export_skeleton_hierarchy(vec![bone("a", None)]);
        assert!(skeleton_export_size(&e) > 0);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_skeleton_hierarchy(vec![bone("root", None), bone("child", Some(0))]);
        assert!(validate_skeleton(&e));
    }

    #[test]
    fn test_validate_bad_parent() {
        let e = export_skeleton_hierarchy(vec![bone("a", Some(0))]);
        assert!(!validate_skeleton(&e));
    }
}
