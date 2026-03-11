//! Pose mirror: reflect a pose from one side of the body to the other (left↔right).
//!
//! Maintains a registry of joint-name pairs that are considered mirrors of each
//! other. When a pose is applied the mirror operation swaps weights and optionally
//! inverts individual axes so that the result looks physically correct on the
//! opposite side.

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Configuration for the pose mirror system.
pub struct PoseMirrorConfig {
    /// Whether to invert the X rotation axis when mirroring.
    pub invert_x: bool,
    /// Whether to invert the Y rotation axis when mirroring.
    pub invert_y: bool,
    /// Whether to invert the Z rotation axis when mirroring.
    pub invert_z: bool,
}

/// A pair of joint names that mirror each other.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MirrorPair {
    /// Left-side joint name.
    pub left: String,
    /// Right-side joint name.
    pub right: String,
}

/// Result of a mirror operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseMirrorResult {
    /// Number of pairs that were successfully mirrored.
    pub mirrored_count: usize,
    /// Names of joints that had no mirror partner.
    pub unmatched: Vec<String>,
}

/// The pose mirror system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseMirror {
    config: PoseMirrorConfig,
    pairs: Vec<MirrorPair>,
}

/// Create a default [`PoseMirrorConfig`].
#[allow(dead_code)]
pub fn default_pose_mirror_config() -> PoseMirrorConfig {
    PoseMirrorConfig {
        invert_x: false,
        invert_y: false,
        invert_z: true,
    }
}

/// Create a new [`PoseMirror`] with the given configuration.
#[allow(dead_code)]
pub fn new_pose_mirror(config: PoseMirrorConfig) -> PoseMirror {
    PoseMirror {
        config,
        pairs: Vec::new(),
    }
}

/// Add a mirror pair to the system.
/// Returns `false` if an identical pair already exists.
#[allow(dead_code)]
pub fn pose_mirror_add_pair(mirror: &mut PoseMirror, left: &str, right: &str) -> bool {
    if mirror
        .pairs
        .iter()
        .any(|p| p.left == left && p.right == right)
    {
        return false;
    }
    mirror.pairs.push(MirrorPair {
        left: left.to_owned(),
        right: right.to_owned(),
    });
    true
}

/// Apply the mirror: for every registered pair swap the weights in `pose` (a
/// map from joint-name → weight `f32`).  Weights not in any pair are left
/// unchanged.  Returns a [`PoseMirrorResult`] describing what happened.
#[allow(dead_code)]
pub fn pose_mirror_apply(
    mirror: &PoseMirror,
    pose: &mut std::collections::HashMap<String, f32>,
) -> PoseMirrorResult {
    let mut mirrored_count = 0usize;
    let mut unmatched: Vec<String> = Vec::new();

    for pair in &mirror.pairs {
        let l = pose.get(&pair.left).copied();
        let r = pose.get(&pair.right).copied();
        match (l, r) {
            (Some(lv), Some(rv)) => {
                let lv_m = if mirror.config.invert_z { -lv } else { lv };
                let rv_m = if mirror.config.invert_z { -rv } else { rv };
                pose.insert(pair.left.clone(), rv_m);
                pose.insert(pair.right.clone(), lv_m);
                mirrored_count += 1;
            }
            (Some(_), None) => unmatched.push(pair.right.clone()),
            (None, Some(_)) => unmatched.push(pair.left.clone()),
            (None, None) => {}
        }
    }

    PoseMirrorResult {
        mirrored_count,
        unmatched,
    }
}

/// Return the number of registered mirror pairs.
#[allow(dead_code)]
pub fn pose_mirror_pair_count(mirror: &PoseMirror) -> usize {
    mirror.pairs.len()
}

/// Check whether a pair with the given left/right names exists.
#[allow(dead_code)]
pub fn pose_mirror_has_pair(mirror: &PoseMirror, left: &str, right: &str) -> bool {
    mirror
        .pairs
        .iter()
        .any(|p| p.left == left && p.right == right)
}

/// Remove the pair identified by `left`/`right`.  Returns `true` if removed.
#[allow(dead_code)]
pub fn pose_mirror_remove_pair(mirror: &mut PoseMirror, left: &str, right: &str) -> bool {
    let before = mirror.pairs.len();
    mirror
        .pairs
        .retain(|p| !(p.left == left && p.right == right));
    mirror.pairs.len() < before
}

/// Remove all mirror pairs.
#[allow(dead_code)]
pub fn pose_mirror_clear(mirror: &mut PoseMirror) {
    mirror.pairs.clear();
}

/// Serialize the mirror state to a compact JSON string.
#[allow(dead_code)]
pub fn pose_mirror_to_json(mirror: &PoseMirror) -> String {
    let pairs_json: Vec<String> = mirror
        .pairs
        .iter()
        .map(|p| format!(r#"{{"left":"{}","right":"{}"}}"#, p.left, p.right))
        .collect();
    format!(
        r#"{{"pair_count":{},"invert_z":{},"pairs":[{}]}}"#,
        mirror.pairs.len(),
        mirror.config.invert_z,
        pairs_json.join(",")
    )
}

/// Return a new [`PoseMirror`] with all axis-inversion flags toggled.
#[allow(dead_code)]
pub fn pose_mirror_invert(mirror: &PoseMirror) -> PoseMirror {
    PoseMirror {
        config: PoseMirrorConfig {
            invert_x: !mirror.config.invert_x,
            invert_y: !mirror.config.invert_y,
            invert_z: !mirror.config.invert_z,
        },
        pairs: mirror.pairs.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_mirror() -> PoseMirror {
        let cfg = default_pose_mirror_config();
        let mut m = new_pose_mirror(cfg);
        pose_mirror_add_pair(&mut m, "left_arm", "right_arm");
        pose_mirror_add_pair(&mut m, "left_leg", "right_leg");
        m
    }

    #[test]
    fn test_default_config() {
        let cfg = default_pose_mirror_config();
        assert!(cfg.invert_z);
    }

    #[test]
    fn test_add_pair_increases_count() {
        let mut m = new_pose_mirror(default_pose_mirror_config());
        assert_eq!(pose_mirror_pair_count(&m), 0);
        pose_mirror_add_pair(&mut m, "l", "r");
        assert_eq!(pose_mirror_pair_count(&m), 1);
    }

    #[test]
    fn test_add_duplicate_pair_rejected() {
        let mut m = new_pose_mirror(default_pose_mirror_config());
        assert!(pose_mirror_add_pair(&mut m, "l", "r"));
        assert!(!pose_mirror_add_pair(&mut m, "l", "r"));
        assert_eq!(pose_mirror_pair_count(&m), 1);
    }

    #[test]
    fn test_has_pair() {
        let m = make_mirror();
        assert!(pose_mirror_has_pair(&m, "left_arm", "right_arm"));
        assert!(!pose_mirror_has_pair(&m, "left_arm", "left_leg"));
    }

    #[test]
    fn test_remove_pair() {
        let mut m = make_mirror();
        assert!(pose_mirror_remove_pair(&mut m, "left_arm", "right_arm"));
        assert!(!pose_mirror_has_pair(&m, "left_arm", "right_arm"));
        assert_eq!(pose_mirror_pair_count(&m), 1);
    }

    #[test]
    fn test_clear() {
        let mut m = make_mirror();
        pose_mirror_clear(&mut m);
        assert_eq!(pose_mirror_pair_count(&m), 0);
    }

    #[test]
    fn test_apply_swaps_with_invert() {
        let mut m = new_pose_mirror(default_pose_mirror_config()); // invert_z = true
        pose_mirror_add_pair(&mut m, "l", "r");
        let mut pose = HashMap::new();
        pose.insert("l".to_owned(), 0.8f32);
        pose.insert("r".to_owned(), 0.2f32);
        let res = pose_mirror_apply(&m, &mut pose);
        assert_eq!(res.mirrored_count, 1);
        // With invert_z the signs flip: original l=0.8, r=0.2
        // After: l = -0.2, r = -0.8
        assert!((pose["l"] - (-0.2f32)).abs() < 1e-6);
        assert!((pose["r"] - (-0.8f32)).abs() < 1e-6);
    }

    #[test]
    fn test_apply_no_invert() {
        let mut cfg = default_pose_mirror_config();
        cfg.invert_z = false;
        let mut m = new_pose_mirror(cfg);
        pose_mirror_add_pair(&mut m, "l", "r");
        let mut pose = HashMap::new();
        pose.insert("l".to_owned(), 0.5f32);
        pose.insert("r".to_owned(), 0.3f32);
        pose_mirror_apply(&m, &mut pose);
        assert!((pose["l"] - 0.3f32).abs() < 1e-6);
        assert!((pose["r"] - 0.5f32).abs() < 1e-6);
    }

    #[test]
    fn test_apply_unmatched_reported() {
        let mut m = new_pose_mirror(default_pose_mirror_config());
        pose_mirror_add_pair(&mut m, "l", "r");
        let mut pose = HashMap::new();
        pose.insert("l".to_owned(), 1.0f32);
        // "r" is missing from the pose
        let res = pose_mirror_apply(&m, &mut pose);
        assert!(res.unmatched.contains(&"r".to_owned()));
    }

    #[test]
    fn test_to_json_contains_pair_count() {
        let m = make_mirror();
        let json = pose_mirror_to_json(&m);
        assert!(json.contains("pair_count"));
        assert!(json.contains("left_arm"));
    }

    #[test]
    fn test_invert_flips_flags() {
        let m = make_mirror(); // invert_z = true
        let inv = pose_mirror_invert(&m);
        assert!(!inv.config.invert_z);
        assert_eq!(pose_mirror_pair_count(&inv), pose_mirror_pair_count(&m));
    }
}
