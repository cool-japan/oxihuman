// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export a skeleton hierarchy to a structured JSON format.

#![allow(dead_code)]

/// Configuration for skeleton JSON export.
#[derive(Debug, Clone)]
pub struct SkeletonJsonConfig {
    /// Whether to embed bind-pose matrices.
    pub include_bind_pose: bool,
    /// Float precision for matrix / position values.
    pub precision: usize,
}

/// A single joint node in the exported skeleton.
#[derive(Debug, Clone)]
pub struct JointJsonNode {
    /// Joint identifier.
    pub id: u32,
    /// Joint name.
    pub name: String,
    /// Parent joint id; `None` for root joints.
    pub parent_id: Option<u32>,
    /// Local-space head position [x, y, z].
    pub head: [f64; 3],
    /// Local-space tail position [x, y, z].
    pub tail: [f64; 3],
}

/// Result holding the skeleton to be exported.
#[derive(Debug, Clone)]
pub struct SkeletonJsonResult {
    /// All joint nodes.
    pub joints: Vec<JointJsonNode>,
}

/// Returns the default [`SkeletonJsonConfig`].
#[allow(dead_code)]
pub fn default_skeleton_json_config() -> SkeletonJsonConfig {
    SkeletonJsonConfig {
        include_bind_pose: false,
        precision: 6,
    }
}

/// Creates a new, empty [`SkeletonJsonResult`].
#[allow(dead_code)]
pub fn new_skeleton_json_export() -> SkeletonJsonResult {
    SkeletonJsonResult { joints: Vec::new() }
}

/// Adds a joint to the skeleton.
#[allow(dead_code)]
pub fn skeleton_json_add_joint(result: &mut SkeletonJsonResult, joint: JointJsonNode) {
    result.joints.push(joint);
}

/// Sets the parent of a joint identified by `child_id`.
#[allow(dead_code)]
pub fn skeleton_json_set_parent(
    result: &mut SkeletonJsonResult,
    child_id: u32,
    parent_id: u32,
) -> bool {
    if let Some(j) = result.joints.iter_mut().find(|j| j.id == child_id) {
        j.parent_id = Some(parent_id);
        true
    } else {
        false
    }
}

/// Serialises the skeleton to a JSON string.
#[allow(dead_code)]
pub fn skeleton_json_to_string(result: &SkeletonJsonResult, cfg: &SkeletonJsonConfig) -> String {
    let prec = cfg.precision;
    let mut out = String::from("{\"joints\":[\n");
    for (i, j) in result.joints.iter().enumerate() {
        let comma = if i + 1 < result.joints.len() { "," } else { "" };
        let parent = match j.parent_id {
            Some(p) => format!("{}", p),
            None => "null".to_string(),
        };
        let head = format!(
            "[{:.prec$},{:.prec$},{:.prec$}]",
            j.head[0], j.head[1], j.head[2]
        );
        let tail = format!(
            "[{:.prec$},{:.prec$},{:.prec$}]",
            j.tail[0], j.tail[1], j.tail[2]
        );
        out.push_str(&format!(
            "  {{\"id\":{},\"name\":\"{}\",\"parent\":{},\"head\":{},\"tail\":{}}}{}",
            j.id, j.name, parent, head, tail, comma
        ));
        out.push('\n');
    }
    out.push_str("]}");
    out
}

/// Returns the number of joints.
#[allow(dead_code)]
pub fn skeleton_json_joint_count(result: &SkeletonJsonResult) -> usize {
    result.joints.len()
}

/// Writes the JSON export to a file (stub – returns byte count).
#[allow(dead_code)]
pub fn skeleton_json_write_to_file(
    result: &SkeletonJsonResult,
    cfg: &SkeletonJsonConfig,
    _path: &str,
) -> usize {
    skeleton_json_to_string(result, cfg).len()
}

/// Returns the ids of joints that have no parent.
#[allow(dead_code)]
pub fn skeleton_json_root_joints(result: &SkeletonJsonResult) -> Vec<u32> {
    result
        .joints
        .iter()
        .filter(|j| j.parent_id.is_none())
        .map(|j| j.id)
        .collect()
}

/// Computes the maximum depth of the skeleton hierarchy.
#[allow(dead_code)]
pub fn skeleton_json_depth(result: &SkeletonJsonResult) -> usize {
    fn depth_of(result: &SkeletonJsonResult, id: u32) -> usize {
        let children: Vec<u32> = result
            .joints
            .iter()
            .filter(|j| j.parent_id == Some(id))
            .map(|j| j.id)
            .collect();
        if children.is_empty() {
            1
        } else {
            1 + children
                .iter()
                .map(|&c| depth_of(result, c))
                .max()
                .unwrap_or(0)
        }
    }

    let roots = skeleton_json_root_joints(result);
    roots
        .iter()
        .map(|&r| depth_of(result, r))
        .max()
        .unwrap_or(0)
}

/// Clears all joints.
#[allow(dead_code)]
pub fn skeleton_json_clear(result: &mut SkeletonJsonResult) {
    result.joints.clear();
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_joint(id: u32, name: &str, parent: Option<u32>) -> JointJsonNode {
    JointJsonNode {
        id,
        name: name.to_string(),
        parent_id: parent,
        head: [0.0, f64::from(id), 0.0],
        tail: [0.0, f64::from(id) + 1.0, 0.0],
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_skeleton_json_config();
        assert!(!cfg.include_bind_pose);
        assert_eq!(cfg.precision, 6);
    }

    #[test]
    fn new_export_is_empty() {
        let r = new_skeleton_json_export();
        assert_eq!(skeleton_json_joint_count(&r), 0);
    }

    #[test]
    fn add_joint_increases_count() {
        let mut r = new_skeleton_json_export();
        skeleton_json_add_joint(&mut r, make_joint(0, "root", None));
        assert_eq!(skeleton_json_joint_count(&r), 1);
    }

    #[test]
    fn set_parent_updates_joint() {
        let mut r = new_skeleton_json_export();
        skeleton_json_add_joint(&mut r, make_joint(0, "hip", None));
        skeleton_json_add_joint(&mut r, make_joint(1, "spine", None));
        let ok = skeleton_json_set_parent(&mut r, 1, 0);
        assert!(ok);
        assert_eq!(r.joints[1].parent_id, Some(0));
    }

    #[test]
    fn set_parent_missing_joint_returns_false() {
        let mut r = new_skeleton_json_export();
        let ok = skeleton_json_set_parent(&mut r, 99, 0);
        assert!(!ok);
    }

    #[test]
    fn root_joints_are_detected() {
        let mut r = new_skeleton_json_export();
        skeleton_json_add_joint(&mut r, make_joint(0, "hip", None));
        skeleton_json_add_joint(&mut r, make_joint(1, "spine", Some(0)));
        let roots = skeleton_json_root_joints(&r);
        assert_eq!(roots, vec![0]);
    }

    #[test]
    fn json_contains_joint_name() {
        let mut r = new_skeleton_json_export();
        skeleton_json_add_joint(&mut r, make_joint(0, "pelvis", None));
        let cfg = default_skeleton_json_config();
        let json = skeleton_json_to_string(&r, &cfg);
        assert!(json.contains("\"pelvis\""));
    }

    #[test]
    fn depth_single_chain() {
        let mut r = new_skeleton_json_export();
        skeleton_json_add_joint(&mut r, make_joint(0, "a", None));
        skeleton_json_add_joint(&mut r, make_joint(1, "b", Some(0)));
        skeleton_json_add_joint(&mut r, make_joint(2, "c", Some(1)));
        assert_eq!(skeleton_json_depth(&r), 3);
    }

    #[test]
    fn clear_removes_all_joints() {
        let mut r = new_skeleton_json_export();
        skeleton_json_add_joint(&mut r, make_joint(0, "x", None));
        skeleton_json_clear(&mut r);
        assert_eq!(skeleton_json_joint_count(&r), 0);
    }

    #[test]
    fn write_to_file_returns_nonzero_bytes() {
        let mut r = new_skeleton_json_export();
        skeleton_json_add_joint(&mut r, make_joint(0, "root", None));
        let cfg = default_skeleton_json_config();
        let bytes = skeleton_json_write_to_file(&r, &cfg, "/tmp/skel.json");
        assert!(bytes > 0);
    }
}
