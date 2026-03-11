// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Joint orientation export for skeleton rigs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointOrientExport {
    pub joints: Vec<JointOrient>,
}

/// Single joint orientation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointOrient {
    pub name: String,
    pub orientation: [f32; 4], // quaternion (x,y,z,w)
    pub parent_index: i32,
}

#[allow(dead_code)]
impl JointOrientExport {
    pub fn new() -> Self { Self { joints: Vec::new() } }

    pub fn add(&mut self, name: &str, orientation: [f32; 4], parent: i32) {
        self.joints.push(JointOrient { name: name.to_string(), orientation, parent_index: parent });
    }

    pub fn count(&self) -> usize { self.joints.len() }

    pub fn find(&self, name: &str) -> Option<&JointOrient> {
        self.joints.iter().find(|j| j.name == name)
    }

    pub fn root_count(&self) -> usize {
        self.joints.iter().filter(|j| j.parent_index < 0).count()
    }

    pub fn is_normalized(&self) -> bool {
        self.joints.iter().all(|j| {
            let q = &j.orientation;
            let len = (q[0]*q[0]+q[1]*q[1]+q[2]*q[2]+q[3]*q[3]).sqrt();
            (len - 1.0).abs() < 0.01
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.joints.len() as u32).to_le_bytes());
        for j in &self.joints {
            bytes.extend_from_slice(&j.parent_index.to_le_bytes());
            for &q in &j.orientation { bytes.extend_from_slice(&q.to_le_bytes()); }
        }
        bytes
    }

    pub fn to_json(&self) -> String {
        let mut s = String::from("[");
        for (i, j) in self.joints.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str(&format!("{{\"name\":\"{}\",\"parent\":{}}}", j.name, j.parent_index));
        }
        s.push(']');
        s
    }
}

impl Default for JointOrientExport {
    fn default() -> Self { Self::new() }
}

/// Validate joint orientations.
#[allow(dead_code)]
pub fn validate_joint_orient(jo: &JointOrientExport) -> bool {
    jo.is_normalized() && jo.root_count() >= 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> JointOrientExport {
        let mut jo = JointOrientExport::new();
        jo.add("root", [0.0, 0.0, 0.0, 1.0], -1);
        jo.add("spine", [0.0, 0.0, 0.0, 1.0], 0);
        jo
    }

    #[test]
    fn test_count() { assert_eq!(sample().count(), 2); }

    #[test]
    fn test_find() { assert!(sample().find("root").is_some()); }

    #[test]
    fn test_root_count() { assert_eq!(sample().root_count(), 1); }

    #[test]
    fn test_is_normalized() { assert!(sample().is_normalized()); }

    #[test]
    fn test_validate() { assert!(validate_joint_orient(&sample())); }

    #[test]
    fn test_to_bytes() { assert!(!sample().to_bytes().is_empty()); }

    #[test]
    fn test_to_json() { assert!(sample().to_json().contains("root")); }

    #[test]
    fn test_empty() { assert_eq!(JointOrientExport::new().count(), 0); }

    #[test]
    fn test_default() { assert_eq!(JointOrientExport::default().count(), 0); }

    #[test]
    fn test_not_normalized() {
        let mut jo = JointOrientExport::new();
        jo.add("bad", [2.0, 0.0, 0.0, 0.0], -1);
        assert!(!jo.is_normalized());
    }
}
