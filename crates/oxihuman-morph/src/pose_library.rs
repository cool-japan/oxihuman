#![allow(dead_code)]
//! Library of named poses with parameter snapshots.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PoseEntry {
    pub name: String,
    pub params: HashMap<String, f32>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PoseLibrary {
    poses: Vec<PoseEntry>,
}

#[allow(dead_code)]
pub fn new_pose_library() -> PoseLibrary {
    PoseLibrary { poses: Vec::new() }
}

#[allow(dead_code)]
pub fn add_pose(lib: &mut PoseLibrary, name: &str, params: HashMap<String, f32>) {
    lib.poses.push(PoseEntry {
        name: name.to_string(),
        params,
    });
}

#[allow(dead_code)]
pub fn get_pose<'a>(lib: &'a PoseLibrary, name: &str) -> Option<&'a PoseEntry> {
    lib.poses.iter().find(|p| p.name == name)
}

#[allow(dead_code)]
pub fn remove_pose(lib: &mut PoseLibrary, name: &str) -> bool {
    let before = lib.poses.len();
    lib.poses.retain(|p| p.name != name);
    lib.poses.len() < before
}

#[allow(dead_code)]
pub fn pose_count(lib: &PoseLibrary) -> usize {
    lib.poses.len()
}

#[allow(dead_code)]
pub fn pose_names(lib: &PoseLibrary) -> Vec<&str> {
    lib.poses.iter().map(|p| p.name.as_str()).collect()
}

#[allow(dead_code)]
pub fn pose_to_json(entry: &PoseEntry) -> String {
    let params: Vec<String> = entry
        .params
        .iter()
        .map(|(k, v)| format!("\"{k}\":{v}"))
        .collect();
    format!("{{\"name\":\"{}\",\"params\":{{{}}}}}", entry.name, params.join(","))
}

#[allow(dead_code)]
pub fn library_clear(lib: &mut PoseLibrary) {
    lib.poses.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pose_library() {
        let lib = new_pose_library();
        assert_eq!(pose_count(&lib), 0);
    }

    #[test]
    fn test_add_pose() {
        let mut lib = new_pose_library();
        add_pose(&mut lib, "idle", HashMap::new());
        assert_eq!(pose_count(&lib), 1);
    }

    #[test]
    fn test_get_pose() {
        let mut lib = new_pose_library();
        let mut p = HashMap::new();
        p.insert("x".to_string(), 0.5);
        add_pose(&mut lib, "walk", p);
        assert!(get_pose(&lib, "walk").is_some());
        assert!(get_pose(&lib, "run").is_none());
    }

    #[test]
    fn test_remove_pose() {
        let mut lib = new_pose_library();
        add_pose(&mut lib, "a", HashMap::new());
        assert!(remove_pose(&mut lib, "a"));
        assert!(!remove_pose(&mut lib, "a"));
    }

    #[test]
    fn test_pose_names() {
        let mut lib = new_pose_library();
        add_pose(&mut lib, "a", HashMap::new());
        add_pose(&mut lib, "b", HashMap::new());
        let names = pose_names(&lib);
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_pose_to_json() {
        let entry = PoseEntry {
            name: "test".to_string(),
            params: HashMap::new(),
        };
        let json = pose_to_json(&entry);
        assert!(json.contains("\"name\":\"test\""));
    }

    #[test]
    fn test_library_clear() {
        let mut lib = new_pose_library();
        add_pose(&mut lib, "x", HashMap::new());
        library_clear(&mut lib);
        assert_eq!(pose_count(&lib), 0);
    }

    #[test]
    fn test_pose_with_params() {
        let mut lib = new_pose_library();
        let mut p = HashMap::new();
        p.insert("arm_l".to_string(), 0.3);
        p.insert("arm_r".to_string(), 0.7);
        add_pose(&mut lib, "arms", p);
        let pose = get_pose(&lib, "arms").unwrap();
        assert_eq!(pose.params.len(), 2);
    }

    #[test]
    fn test_multiple_poses() {
        let mut lib = new_pose_library();
        for i in 0..5 {
            add_pose(&mut lib, &format!("p{i}"), HashMap::new());
        }
        assert_eq!(pose_count(&lib), 5);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut lib = new_pose_library();
        assert!(!remove_pose(&mut lib, "nope"));
    }
}
