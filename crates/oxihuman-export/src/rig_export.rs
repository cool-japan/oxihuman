// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Skeleton/rig export for animation pipelines.
//!
//! Provides structs and functions for building, validating, and serializing
//! hierarchical bone rigs to JSON and CSV formats.

// ── Types ──────────────────────────────────────────────────────────────────

/// A single bone in an exported rig hierarchy.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigExportBone {
    /// Unique numeric identifier for this bone.
    pub id: u32,
    /// Human-readable bone name.
    pub name: String,
    /// Parent bone ID, or `None` for root bones.
    pub parent_id: Option<u32>,
    /// World-space head position `[x, y, z]`.
    pub head: [f32; 3],
    /// World-space tail position `[x, y, z]`.
    pub tail: [f32; 3],
    /// Rotation quaternion `[x, y, z, w]`.
    pub rotation: [f32; 4],
    /// Bone length in world units.
    pub length: f32,
    /// Bind-pose transform (head + rotation snapshot).
    pub bind_pose: ([f32; 3], [f32; 4]),
}

/// An assembled rig ready for export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExportRig {
    /// Rig name (e.g. `"humanoid"`).
    pub name: String,
    /// All bones in the rig.
    pub bones: Vec<RigExportBone>,
}

/// Configuration controlling how a rig is exported.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigExportConfig {
    /// Include the bind-pose in output.
    pub include_bind_pose: bool,
    /// Floating-point precision (decimal places).
    pub precision: u32,
    /// Export only bones whose names match this prefix (empty = all).
    pub name_filter_prefix: String,
}

// ── Type aliases ───────────────────────────────────────────────────────────

/// Result of a rig validation check.
pub type RigValidationResult = Result<(), String>;

// ── Config ─────────────────────────────────────────────────────────────────

/// Return a sensible default [`RigExportConfig`].
#[allow(dead_code)]
pub fn default_rig_export_config() -> RigExportConfig {
    RigExportConfig {
        include_bind_pose: true,
        precision: 6,
        name_filter_prefix: String::new(),
    }
}

// ── Rig construction ───────────────────────────────────────────────────────

/// Create a new, empty [`ExportRig`] with the given name.
#[allow(dead_code)]
pub fn new_export_rig(name: &str) -> ExportRig {
    ExportRig {
        name: name.to_string(),
        bones: Vec::new(),
    }
}

/// Append a bone to the rig.
#[allow(dead_code)]
pub fn add_bone(rig: &mut ExportRig, bone: RigExportBone) {
    rig.bones.push(bone);
}

/// Remove the bone with the given `id` from the rig.
/// Returns `true` if a bone was removed.
#[allow(dead_code)]
pub fn remove_bone(rig: &mut ExportRig, id: u32) -> bool {
    let before = rig.bones.len();
    rig.bones.retain(|b| b.id != id);
    rig.bones.len() < before
}

/// Return the total number of bones in the rig.
#[allow(dead_code)]
pub fn bone_count(rig: &ExportRig) -> usize {
    rig.bones.len()
}

// ── Hierarchy queries ──────────────────────────────────────────────────────

/// Return all root bones (bones with no parent).
#[allow(dead_code)]
pub fn rig_root_bones(rig: &ExportRig) -> Vec<&RigExportBone> {
    rig.bones.iter().filter(|b| b.parent_id.is_none()).collect()
}

/// Find a bone by name (case-sensitive). Returns `None` if not found.
#[allow(dead_code)]
pub fn find_bone_by_name<'a>(rig: &'a ExportRig, name: &str) -> Option<&'a RigExportBone> {
    rig.bones.iter().find(|b| b.name == name)
}

/// Return the ancestor chain of the bone with `start_id`, starting at
/// `start_id` and walking up to the root.  Returns an empty `Vec` if
/// `start_id` does not exist.
#[allow(dead_code)]
pub fn bone_chain(rig: &ExportRig, start_id: u32) -> Vec<&RigExportBone> {
    let mut result = Vec::new();
    let mut current_id = Some(start_id);
    let mut visited = std::collections::HashSet::new();
    while let Some(cid) = current_id {
        if !visited.insert(cid) {
            break; // cycle guard
        }
        if let Some(bone) = rig.bones.iter().find(|b| b.id == cid) {
            result.push(bone);
            current_id = bone.parent_id;
        } else {
            break;
        }
    }
    result
}

/// Compute the maximum hierarchy depth of the rig (longest path from root
/// to leaf).  Returns 0 for an empty rig.
#[allow(dead_code)]
pub fn rig_depth(rig: &ExportRig) -> usize {
    fn depth_of(rig: &ExportRig, id: u32, visited: &mut std::collections::HashSet<u32>) -> usize {
        if !visited.insert(id) {
            return 0;
        }
        let children: Vec<u32> = rig
            .bones
            .iter()
            .filter(|b| b.parent_id == Some(id))
            .map(|b| b.id)
            .collect();
        if children.is_empty() {
            return 1;
        }
        1 + children
            .into_iter()
            .map(|cid| depth_of(rig, cid, visited))
            .max()
            .unwrap_or(0)
    }
    rig_root_bones(rig)
        .iter()
        .map(|r| depth_of(rig, r.id, &mut std::collections::HashSet::new()))
        .max()
        .unwrap_or(0)
}

/// Validate the rig: checks there are no cycles and all parent IDs point to
/// existing bones.  Returns `Ok(())` on success or an error string.
#[allow(dead_code)]
pub fn validate_rig(rig: &ExportRig) -> RigValidationResult {
    // Check parent references
    let ids: std::collections::HashSet<u32> = rig.bones.iter().map(|b| b.id).collect();
    for bone in &rig.bones {
        if let Some(pid) = bone.parent_id {
            if !ids.contains(&pid) {
                return Err(format!(
                    "bone '{}' references non-existent parent id {}",
                    bone.name, pid
                ));
            }
        }
    }
    // Cycle check via DFS
    for bone in &rig.bones {
        let mut visited = std::collections::HashSet::new();
        let mut cur = bone.parent_id;
        while let Some(pid) = cur {
            if !visited.insert(pid) {
                return Err(format!("cycle detected involving bone id {pid}"));
            }
            cur = rig
                .bones
                .iter()
                .find(|b| b.id == pid)
                .and_then(|b| b.parent_id);
        }
    }
    Ok(())
}

/// Return the sum of `length` across all bones.
#[allow(dead_code)]
pub fn total_bone_length(rig: &ExportRig) -> f32 {
    rig.bones.iter().map(|b| b.length).sum()
}

/// Overwrite the bind-pose of the bone with `id`.
/// Returns `true` if the bone was found.
#[allow(dead_code)]
pub fn set_bone_bind_pose(rig: &mut ExportRig, id: u32, pos: [f32; 3], rot: [f32; 4]) -> bool {
    if let Some(bone) = rig.bones.iter_mut().find(|b| b.id == id) {
        bone.bind_pose = (pos, rot);
        true
    } else {
        false
    }
}

// ── Serialization ──────────────────────────────────────────────────────────

/// Serialize the rig to a compact JSON string.
#[allow(dead_code)]
pub fn rig_to_json(rig: &ExportRig) -> String {
    let bone_strs: Vec<String> = rig
        .bones
        .iter()
        .map(|b| {
            let parent = match b.parent_id {
                Some(p) => format!("{p}"),
                None => "null".to_string(),
            };
            let (bp, br) = b.bind_pose;
            format!(
                r#"{{"id":{},"name":"{}","parent_id":{},"head":[{},{},{}],"tail":[{},{},{}],"rotation":[{},{},{},{}],"length":{},"bind_pose":{{"pos":[{},{},{}],"rot":[{},{},{},{}]}}}}"#,
                b.id,
                b.name,
                parent,
                b.head[0], b.head[1], b.head[2],
                b.tail[0], b.tail[1], b.tail[2],
                b.rotation[0], b.rotation[1], b.rotation[2], b.rotation[3],
                b.length,
                bp[0], bp[1], bp[2],
                br[0], br[1], br[2], br[3],
            )
        })
        .collect();
    format!(
        r#"{{"name":"{}","bones":[{}]}}"#,
        rig.name,
        bone_strs.join(",")
    )
}

/// Serialize the rig to a CSV string (one row per bone).
#[allow(dead_code)]
pub fn rig_to_csv(rig: &ExportRig) -> String {
    let mut out = String::from("id,name,parent_id,head_x,head_y,head_z,tail_x,tail_y,tail_z,rot_x,rot_y,rot_z,rot_w,length\n");
    for b in &rig.bones {
        let parent = match b.parent_id {
            Some(p) => format!("{p}"),
            None => "".to_string(),
        };
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            b.id,
            b.name,
            parent,
            b.head[0],
            b.head[1],
            b.head[2],
            b.tail[0],
            b.tail[1],
            b.tail[2],
            b.rotation[0],
            b.rotation[1],
            b.rotation[2],
            b.rotation[3],
            b.length,
        ));
    }
    out
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bone(id: u32, name: &str, parent: Option<u32>) -> RigExportBone {
        RigExportBone {
            id,
            name: name.to_string(),
            parent_id: parent,
            head: [0.0, f32::from(id as u8), 0.0],
            tail: [0.0, f32::from(id as u8) + 1.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            length: 1.0,
            bind_pose: ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]),
        }
    }

    #[test]
    fn test_default_rig_export_config() {
        let cfg = default_rig_export_config();
        assert!(cfg.include_bind_pose);
        assert_eq!(cfg.precision, 6);
        assert!(cfg.name_filter_prefix.is_empty());
    }

    #[test]
    fn test_new_export_rig() {
        let rig = new_export_rig("human");
        assert_eq!(rig.name, "human");
        assert!(rig.bones.is_empty());
    }

    #[test]
    fn test_add_bone() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "root", None));
        assert_eq!(bone_count(&rig), 1);
    }

    #[test]
    fn test_remove_bone_found() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "root", None));
        add_bone(&mut rig, make_bone(1, "spine", Some(0)));
        let removed = remove_bone(&mut rig, 1);
        assert!(removed);
        assert_eq!(bone_count(&rig), 1);
    }

    #[test]
    fn test_remove_bone_not_found() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "root", None));
        let removed = remove_bone(&mut rig, 99);
        assert!(!removed);
        assert_eq!(bone_count(&rig), 1);
    }

    #[test]
    fn test_bone_count_empty() {
        let rig = new_export_rig("r");
        assert_eq!(bone_count(&rig), 0);
    }

    #[test]
    fn test_rig_root_bones() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "hip", None));
        add_bone(&mut rig, make_bone(1, "spine", Some(0)));
        add_bone(&mut rig, make_bone(2, "neck", None));
        let roots = rig_root_bones(&rig);
        assert_eq!(roots.len(), 2);
    }

    #[test]
    fn test_find_bone_by_name_found() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "hip", None));
        add_bone(&mut rig, make_bone(1, "spine", Some(0)));
        let bone = find_bone_by_name(&rig, "spine");
        assert!(bone.is_some());
        assert_eq!(bone.unwrap().id, 1);
    }

    #[test]
    fn test_find_bone_by_name_missing() {
        let rig = new_export_rig("r");
        assert!(find_bone_by_name(&rig, "missing").is_none());
    }

    #[test]
    fn test_bone_chain_single_root() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "hip", None));
        add_bone(&mut rig, make_bone(1, "spine", Some(0)));
        add_bone(&mut rig, make_bone(2, "chest", Some(1)));
        let chain = bone_chain(&rig, 2);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].id, 2);
        assert_eq!(chain[1].id, 1);
        assert_eq!(chain[2].id, 0);
    }

    #[test]
    fn test_bone_chain_nonexistent() {
        let rig = new_export_rig("r");
        let chain = bone_chain(&rig, 99);
        assert!(chain.is_empty());
    }

    #[test]
    fn test_rig_depth_empty() {
        let rig = new_export_rig("r");
        assert_eq!(rig_depth(&rig), 0);
    }

    #[test]
    fn test_rig_depth_three_levels() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "hip", None));
        add_bone(&mut rig, make_bone(1, "spine", Some(0)));
        add_bone(&mut rig, make_bone(2, "chest", Some(1)));
        assert_eq!(rig_depth(&rig), 3);
    }

    #[test]
    fn test_validate_rig_ok() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "hip", None));
        add_bone(&mut rig, make_bone(1, "spine", Some(0)));
        assert!(validate_rig(&rig).is_ok());
    }

    #[test]
    fn test_validate_rig_bad_parent() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(5, "orphan", Some(99)));
        assert!(validate_rig(&rig).is_err());
    }

    #[test]
    fn test_total_bone_length() {
        let mut rig = new_export_rig("r");
        let mut b0 = make_bone(0, "hip", None);
        b0.length = 2.0;
        let mut b1 = make_bone(1, "spine", Some(0));
        b1.length = 3.0;
        add_bone(&mut rig, b0);
        add_bone(&mut rig, b1);
        assert!((total_bone_length(&rig) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_bone_bind_pose_found() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "hip", None));
        let ok = set_bone_bind_pose(&mut rig, 0, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0, 1.0]);
        assert!(ok);
        assert_eq!(rig.bones[0].bind_pose.0, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_set_bone_bind_pose_not_found() {
        let mut rig = new_export_rig("r");
        let ok = set_bone_bind_pose(&mut rig, 99, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]);
        assert!(!ok);
    }

    #[test]
    fn test_rig_to_json_nonempty() {
        let mut rig = new_export_rig("test_rig");
        add_bone(&mut rig, make_bone(0, "hip", None));
        let json = rig_to_json(&rig);
        assert!(!json.is_empty());
        assert!(json.contains("test_rig"));
        assert!(json.contains("hip"));
    }

    #[test]
    fn test_rig_to_csv_has_header() {
        let mut rig = new_export_rig("r");
        add_bone(&mut rig, make_bone(0, "hip", None));
        let csv = rig_to_csv(&rig);
        assert!(csv.starts_with("id,name"));
        assert!(csv.contains("hip"));
    }
}
