// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct XrSkeleton {
    pub joint_names: Vec<String>,
    pub parent_indices: Vec<i32>,
    pub positions: Vec<[f32; 3]>,
}

pub fn new_xr_skeleton() -> XrSkeleton {
    XrSkeleton {
        joint_names: Vec::new(),
        parent_indices: Vec::new(),
        positions: Vec::new(),
    }
}

pub fn xr_push_joint(s: &mut XrSkeleton, name: &str, parent: i32, pos: [f32; 3]) {
    s.joint_names.push(name.to_string());
    s.parent_indices.push(parent);
    s.positions.push(pos);
}

pub fn xr_joint_count(s: &XrSkeleton) -> usize {
    s.joint_names.len()
}

pub fn xr_to_json(s: &XrSkeleton) -> String {
    let joints: Vec<_> = s
        .joint_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let p = s.parent_indices[i];
            let pos = s.positions[i];
            format!(
                r#"{{"name":"{}","parent":{},"pos":[{},{},{}]}}"#,
                name, p, pos[0], pos[1], pos[2]
            )
        })
        .collect();
    format!(r#"{{"joints":[{}]}}"#, joints.join(","))
}

pub fn xr_is_hand_skeleton(s: &XrSkeleton) -> bool {
    s.joint_names.len() == 26
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_xr_skeleton_empty() {
        /* starts empty */
        let s = new_xr_skeleton();
        assert_eq!(xr_joint_count(&s), 0);
    }

    #[test]
    fn test_xr_push_joint() {
        /* push increases count */
        let mut s = new_xr_skeleton();
        xr_push_joint(&mut s, "root", -1, [0.0, 0.0, 0.0]);
        assert_eq!(xr_joint_count(&s), 1);
    }

    #[test]
    fn test_xr_to_json_contains_name() {
        /* json contains joint name */
        let mut s = new_xr_skeleton();
        xr_push_joint(&mut s, "wrist", -1, [0.0, 0.0, 0.0]);
        let j = xr_to_json(&s);
        assert!(j.contains("wrist"));
    }

    #[test]
    fn test_xr_is_hand_false() {
        /* not 26 joints = not hand */
        let s = new_xr_skeleton();
        assert!(!xr_is_hand_skeleton(&s));
    }

    #[test]
    fn test_xr_is_hand_true() {
        /* 26 joints = hand */
        let mut s = new_xr_skeleton();
        for i in 0..26 {
            xr_push_joint(&mut s, &format!("joint{}", i), -1, [0.0, 0.0, 0.0]);
        }
        assert!(xr_is_hand_skeleton(&s));
    }
}
