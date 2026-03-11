// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! DragonBones animation export stub.

/// A DragonBones armature.
#[derive(Debug, Clone)]
pub struct DbArmature {
    pub name: String,
    pub frame_rate: u32,
    pub bone_count: usize,
}

/// A DragonBones export document.
#[derive(Debug, Clone)]
pub struct DragonBonesExport {
    pub version: String,
    pub armatures: Vec<DbArmature>,
}

impl DragonBonesExport {
    /// Create a new DragonBones export.
    pub fn new() -> Self {
        Self {
            version: "5.5".to_string(),
            armatures: Vec::new(),
        }
    }

    /// Add an armature.
    pub fn add_armature(&mut self, armature: DbArmature) {
        self.armatures.push(armature);
    }

    /// Return armature count.
    pub fn armature_count(&self) -> usize {
        self.armatures.len()
    }
}

impl Default for DragonBonesExport {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialize to JSON string (stub).
pub fn export_dragonbones_json(doc: &DragonBonesExport) -> String {
    format!(
        "{{\"version\":\"{}\",\"armature_count\":{}}}",
        doc.version,
        doc.armature_count()
    )
}

/// Total bone count across all armatures.
pub fn total_bone_count_db(doc: &DragonBonesExport) -> usize {
    doc.armatures.iter().map(|a| a.bone_count).sum()
}

/// Find armature by name.
pub fn find_armature<'a>(doc: &'a DragonBonesExport, name: &str) -> Option<&'a DbArmature> {
    doc.armatures.iter().find(|a| a.name == name)
}

/// Average frame rate across armatures.
pub fn avg_frame_rate(doc: &DragonBonesExport) -> f32 {
    if doc.armatures.is_empty() {
        return 0.0;
    }
    let sum: u32 = doc.armatures.iter().map(|a| a.frame_rate).sum();
    sum as f32 / doc.armatures.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> DragonBonesExport {
        let mut doc = DragonBonesExport::new();
        doc.add_armature(DbArmature {
            name: "hero".into(),
            frame_rate: 24,
            bone_count: 10,
        });
        doc.add_armature(DbArmature {
            name: "enemy".into(),
            frame_rate: 30,
            bone_count: 8,
        });
        doc
    }

    #[test]
    fn test_armature_count() {
        /* document has correct armature count */
        let d = sample_doc();
        assert_eq!(d.armature_count(), 2);
    }

    #[test]
    fn test_total_bone_count() {
        /* total bone count sums across armatures */
        let d = sample_doc();
        assert_eq!(total_bone_count_db(&d), 18);
    }

    #[test]
    fn test_export_json_not_empty() {
        /* JSON output is non-empty */
        let d = sample_doc();
        assert!(!export_dragonbones_json(&d).is_empty());
    }

    #[test]
    fn test_find_armature_found() {
        /* find_armature locates existing armature */
        let d = sample_doc();
        assert!(find_armature(&d, "hero").is_some());
    }

    #[test]
    fn test_find_armature_not_found() {
        /* find_armature returns None for missing armature */
        let d = sample_doc();
        assert!(find_armature(&d, "npc").is_none());
    }

    #[test]
    fn test_avg_frame_rate() {
        /* average frame rate is computed correctly */
        let d = sample_doc();
        assert!((avg_frame_rate(&d) - 27.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_document() {
        /* empty document has zero armatures */
        let d = DragonBonesExport::new();
        assert_eq!(d.armature_count(), 0);
    }

    #[test]
    fn test_version_string() {
        /* default version is set */
        let d = DragonBonesExport::new();
        assert!(!d.version.is_empty());
    }
}
