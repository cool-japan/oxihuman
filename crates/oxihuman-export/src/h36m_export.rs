// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct H36mSkeleton {
    pub joints: Vec<[f32; 3]>,
    pub subject: u32,
    pub action: u32,
    pub frame: u32,
}

pub fn new_h36m_skeleton(subject: u32, action: u32, frame: u32) -> H36mSkeleton {
    H36mSkeleton {
        joints: Vec::new(),
        subject,
        action,
        frame,
    }
}

pub fn h36m_push_joint(s: &mut H36mSkeleton, pos: [f32; 3]) {
    s.joints.push(pos);
}

pub fn h36m_joint_count(s: &H36mSkeleton) -> usize {
    s.joints.len()
}

pub fn h36m_to_csv_line(s: &H36mSkeleton) -> String {
    let vals: Vec<_> = s
        .joints
        .iter()
        .flat_map(|j| [j[0].to_string(), j[1].to_string(), j[2].to_string()])
        .collect();
    format!("{},{},{},{}", s.subject, s.action, s.frame, vals.join(","))
}

static H36M_JOINT_NAMES: &[&str] = &[
    "Hip",
    "RHip",
    "RKnee",
    "RFoot",
    "LHip",
    "LKnee",
    "LFoot",
    "Spine",
    "Thorax",
    "Neck/Nose",
    "Head",
    "LShoulder",
    "LElbow",
    "LWrist",
    "RShoulder",
    "RElbow",
    "RWrist",
];

pub fn h36m_joint_name(idx: usize) -> &'static str {
    H36M_JOINT_NAMES.get(idx).copied().unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_h36m_skeleton() {
        /* construction */
        let s = new_h36m_skeleton(1, 2, 0);
        assert_eq!(s.subject, 1);
    }

    #[test]
    fn test_h36m_push_joint() {
        /* push increases count */
        let mut s = new_h36m_skeleton(1, 1, 0);
        h36m_push_joint(&mut s, [0.0, 0.0, 0.0]);
        assert_eq!(h36m_joint_count(&s), 1);
    }

    #[test]
    fn test_h36m_to_csv_line() {
        /* csv line contains subject */
        let s = new_h36m_skeleton(5, 1, 0);
        let line = h36m_to_csv_line(&s);
        assert!(line.starts_with("5,"));
    }

    #[test]
    fn test_h36m_joint_name_hip() {
        /* idx 0 = Hip */
        assert_eq!(h36m_joint_name(0), "Hip");
    }

    #[test]
    fn test_h36m_joint_count_zero() {
        /* starts at 0 */
        let s = new_h36m_skeleton(1, 1, 0);
        assert_eq!(h36m_joint_count(&s), 0);
    }
}
