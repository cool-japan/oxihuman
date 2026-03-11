//! Joint hierarchy export.
#![allow(dead_code)]

/// A single joint for export.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct JointExportData2 {
    pub name: String,
    pub parent: Option<usize>,
    pub local_position: [f32; 3],
}

/// A joint hierarchy.
#[allow(dead_code)]
pub struct JointHierarchy2 {
    pub joints: Vec<JointExportData2>,
}

/// Create a new, empty joint hierarchy.
#[allow(dead_code)]
pub fn new_joint_hierarchy2() -> JointHierarchy2 {
    JointHierarchy2 { joints: Vec::new() }
}

/// Add a joint to the hierarchy.
#[allow(dead_code)]
pub fn add_joint2(hier: &mut JointHierarchy2, joint: JointExportData2) {
    hier.joints.push(joint);
}

/// Get joint count.
#[allow(dead_code)]
pub fn joint2_count(hier: &JointHierarchy2) -> usize { hier.joints.len() }

/// Get parent of joint at index.
#[allow(dead_code)]
pub fn joint2_parent(hier: &JointHierarchy2, i: usize) -> Option<usize> {
    hier.joints.get(i)?.parent
}

/// Get name of joint at index.
#[allow(dead_code)]
pub fn joint2_name(hier: &JointHierarchy2, i: usize) -> &str {
    hier.joints.get(i).map(|j| j.name.as_str()).unwrap_or("")
}

/// Export joint hierarchy to JSON.
#[allow(dead_code)]
pub fn export_joint_hierarchy2(hier: &JointHierarchy2) -> String {
    let joints: Vec<String> = hier.joints.iter().enumerate().map(|(i, j)| {
        let parent_str = j.parent.map(|p| p.to_string()).unwrap_or("null".to_string());
        format!(r#"{{"index":{},"name":"{}","parent":{}}}"#, i, j.name, parent_str)
    }).collect();
    format!("[{}]", joints.join(","))
}

/// Convert a joint to JSON.
#[allow(dead_code)]
pub fn joint2_to_json(joint: &JointExportData2) -> String {
    format!(r#"{{"name":"{}","pos":[{},{},{}]}}"#,
        joint.name, joint.local_position[0], joint.local_position[1], joint.local_position[2])
}

/// Get the root joint index (first joint with no parent).
#[allow(dead_code)]
pub fn root_joint2_index(hier: &JointHierarchy2) -> Option<usize> {
    hier.joints.iter().position(|j| j.parent.is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_joint(name: &str, parent: Option<usize>) -> JointExportData2 {
        JointExportData2 { name: name.to_string(), parent, local_position: [0.0;3] }
    }

    #[test]
    fn test_new_hierarchy_empty() {
        let h = new_joint_hierarchy2();
        assert_eq!(joint2_count(&h), 0);
    }

    #[test]
    fn test_add_joint() {
        let mut h = new_joint_hierarchy2();
        add_joint2(&mut h, make_joint("root", None));
        assert_eq!(joint2_count(&h), 1);
    }

    #[test]
    fn test_joint_parent() {
        let mut h = new_joint_hierarchy2();
        add_joint2(&mut h, make_joint("root", None));
        add_joint2(&mut h, make_joint("child", Some(0)));
        assert_eq!(joint2_parent(&h, 1), Some(0));
    }

    #[test]
    fn test_joint_name() {
        let mut h = new_joint_hierarchy2();
        add_joint2(&mut h, make_joint("spine", None));
        assert_eq!(joint2_name(&h, 0), "spine");
    }

    #[test]
    fn test_export_joint_hierarchy() {
        let mut h = new_joint_hierarchy2();
        add_joint2(&mut h, make_joint("root", None));
        let s = export_joint_hierarchy2(&h);
        assert!(s.contains("root"));
    }

    #[test]
    fn test_joint_to_json() {
        let j = make_joint("hip", Some(0));
        let s = joint2_to_json(&j);
        assert!(s.contains("hip"));
    }

    #[test]
    fn test_root_joint_index() {
        let mut h = new_joint_hierarchy2();
        add_joint2(&mut h, make_joint("root", None));
        add_joint2(&mut h, make_joint("child", Some(0)));
        assert_eq!(root_joint2_index(&h), Some(0));
    }

    #[test]
    fn test_root_joint_no_root() {
        let h = new_joint_hierarchy2();
        assert!(root_joint2_index(&h).is_none());
    }

    #[test]
    fn test_joint_name_oob() {
        let h = new_joint_hierarchy2();
        assert_eq!(joint2_name(&h, 99), "");
    }
}
