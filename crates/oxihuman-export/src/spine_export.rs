// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Spine 2D animation export stub.

/// A Spine slot definition.
#[derive(Debug, Clone)]
pub struct SpineSlot {
    pub name: String,
    pub bone: String,
    pub attachment: Option<String>,
}

/// A Spine bone definition.
#[derive(Debug, Clone)]
pub struct SpineBone {
    pub name: String,
    pub parent: Option<String>,
    pub length: f32,
}

/// Spine skeleton export document.
#[derive(Debug, Clone)]
pub struct SpineExport {
    pub name: String,
    pub bones: Vec<SpineBone>,
    pub slots: Vec<SpineSlot>,
}

impl SpineExport {
    /// Create a new Spine export document.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bones: Vec::new(),
            slots: Vec::new(),
        }
    }

    /// Add a bone.
    pub fn add_bone(&mut self, bone: SpineBone) {
        self.bones.push(bone);
    }

    /// Add a slot.
    pub fn add_slot(&mut self, slot: SpineSlot) {
        self.slots.push(slot);
    }

    /// Return bone count.
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Return slot count.
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }
}

/// Serialize Spine export to JSON string (stub).
pub fn export_spine_json(doc: &SpineExport) -> String {
    format!(
        "{{\"skeleton\":{{\"name\":\"{}\"}},\"bones_count\":{},\"slots_count\":{}}}",
        doc.name,
        doc.bone_count(),
        doc.slot_count()
    )
}

/// Find bone by name.
pub fn find_bone<'a>(doc: &'a SpineExport, name: &str) -> Option<&'a SpineBone> {
    doc.bones.iter().find(|b| b.name == name)
}

/// Total bone length across all bones.
pub fn total_bone_length(doc: &SpineExport) -> f32 {
    doc.bones.iter().map(|b| b.length).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> SpineExport {
        let mut doc = SpineExport::new("character");
        doc.add_bone(SpineBone {
            name: "root".into(),
            parent: None,
            length: 0.0,
        });
        doc.add_bone(SpineBone {
            name: "spine".into(),
            parent: Some("root".into()),
            length: 50.0,
        });
        doc.add_slot(SpineSlot {
            name: "body".into(),
            bone: "spine".into(),
            attachment: None,
        });
        doc
    }

    #[test]
    fn test_bone_count() {
        /* document has correct bone count */
        let d = sample_doc();
        assert_eq!(d.bone_count(), 2);
    }

    #[test]
    fn test_slot_count() {
        /* document has correct slot count */
        let d = sample_doc();
        assert_eq!(d.slot_count(), 1);
    }

    #[test]
    fn test_export_json_not_empty() {
        /* JSON export is non-empty */
        let d = sample_doc();
        let json = export_spine_json(&d);
        assert!(!json.is_empty());
    }

    #[test]
    fn test_export_json_contains_name() {
        /* JSON contains document name */
        let d = sample_doc();
        let json = export_spine_json(&d);
        assert!(json.contains("character"));
    }

    #[test]
    fn test_find_bone_found() {
        /* find_bone locates existing bone */
        let d = sample_doc();
        assert!(find_bone(&d, "spine").is_some());
    }

    #[test]
    fn test_find_bone_not_found() {
        /* find_bone returns None for missing bone */
        let d = sample_doc();
        assert!(find_bone(&d, "arm").is_none());
    }

    #[test]
    fn test_total_bone_length() {
        /* total bone length sums correctly */
        let d = sample_doc();
        assert!((total_bone_length(&d) - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_document() {
        /* empty document has zero bones and slots */
        let d = SpineExport::new("empty");
        assert_eq!(d.bone_count(), 0);
        assert_eq!(d.slot_count(), 0);
    }
}
