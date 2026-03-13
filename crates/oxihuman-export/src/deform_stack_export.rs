// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Type of deformer in the stack.
#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum DeformerType {
    Lattice,
    BlendShape,
    Skinning,
    Wrap,
    Custom(String),
}

impl DeformerType {
    #[allow(dead_code)]
    pub fn name(&self) -> String {
        match self {
            DeformerType::Lattice => "lattice".to_string(),
            DeformerType::BlendShape => "blend_shape".to_string(),
            DeformerType::Skinning => "skinning".to_string(),
            DeformerType::Wrap => "wrap".to_string(),
            DeformerType::Custom(s) => s.clone(),
        }
    }
}

/// A deformer stack entry.
#[allow(dead_code)]
pub struct DeformerEntry {
    pub name: String,
    pub deformer_type: DeformerType,
    pub enabled: bool,
    pub order: usize,
}

/// A deform stack export.
#[allow(dead_code)]
#[derive(Default)]
pub struct DeformStackExport {
    pub entries: Vec<DeformerEntry>,
}

/// Create a new deform stack export.
#[allow(dead_code)]
pub fn new_deform_stack() -> DeformStackExport {
    DeformStackExport::default()
}

/// Add a deformer to the stack.
#[allow(dead_code)]
pub fn add_deformer(stack: &mut DeformStackExport, name: &str, dt: DeformerType, enabled: bool) {
    let order = stack.entries.len();
    stack.entries.push(DeformerEntry {
        name: name.to_string(),
        deformer_type: dt,
        enabled,
        order,
    });
}

/// Count deformers in the stack.
#[allow(dead_code)]
pub fn deformer_count(stack: &DeformStackExport) -> usize {
    stack.entries.len()
}

/// Count enabled deformers.
#[allow(dead_code)]
pub fn enabled_deformer_count(stack: &DeformStackExport) -> usize {
    stack.entries.iter().filter(|e| e.enabled).count()
}

/// Find deformer by name.
#[allow(dead_code)]
pub fn find_deformer<'a>(stack: &'a DeformStackExport, name: &str) -> Option<&'a DeformerEntry> {
    stack.entries.iter().find(|e| e.name == name)
}

/// Get deformers of a given type.
#[allow(dead_code)]
pub fn deformers_of_type<'a>(
    stack: &'a DeformStackExport,
    dt: &DeformerType,
) -> Vec<&'a DeformerEntry> {
    stack
        .entries
        .iter()
        .filter(|e| &e.deformer_type == dt)
        .collect()
}

/// Toggle enabled state of a deformer.
#[allow(dead_code)]
pub fn toggle_deformer(stack: &mut DeformStackExport, name: &str) {
    for e in &mut stack.entries {
        if e.name == name {
            e.enabled = !e.enabled;
        }
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn deform_stack_to_json(stack: &DeformStackExport) -> String {
    format!(
        r#"{{"deformers":{},"enabled":{}}}"#,
        stack.entries.len(),
        enabled_deformer_count(stack)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut s = new_deform_stack();
        add_deformer(&mut s, "skin", DeformerType::Skinning, true);
        assert_eq!(deformer_count(&s), 1);
    }

    #[test]
    fn enabled_count() {
        let mut s = new_deform_stack();
        add_deformer(&mut s, "a", DeformerType::Lattice, true);
        add_deformer(&mut s, "b", DeformerType::Wrap, false);
        assert_eq!(enabled_deformer_count(&s), 1);
    }

    #[test]
    fn find_deformer_found() {
        let mut s = new_deform_stack();
        add_deformer(&mut s, "skin", DeformerType::Skinning, true);
        assert!(find_deformer(&s, "skin").is_some());
    }

    #[test]
    fn find_deformer_missing() {
        let s = new_deform_stack();
        assert!(find_deformer(&s, "x").is_none());
    }

    #[test]
    fn deformers_of_type_filter() {
        let mut s = new_deform_stack();
        add_deformer(&mut s, "a", DeformerType::Skinning, true);
        add_deformer(&mut s, "b", DeformerType::Lattice, true);
        assert_eq!(deformers_of_type(&s, &DeformerType::Skinning).len(), 1);
    }

    #[test]
    fn toggle() {
        let mut s = new_deform_stack();
        add_deformer(&mut s, "x", DeformerType::Wrap, true);
        toggle_deformer(&mut s, "x");
        assert_eq!(enabled_deformer_count(&s), 0);
    }

    #[test]
    fn json_has_deformers() {
        let s = new_deform_stack();
        let j = deform_stack_to_json(&s);
        assert!(j.contains("\"deformers\":0"));
    }

    #[test]
    fn type_name() {
        assert_eq!(DeformerType::Skinning.name(), "skinning");
    }

    #[test]
    fn custom_type_name() {
        let t = DeformerType::Custom("ffd".to_string());
        assert_eq!(t.name(), "ffd");
    }

    #[test]
    fn order_increments() {
        let mut s = new_deform_stack();
        add_deformer(&mut s, "a", DeformerType::Lattice, true);
        add_deformer(&mut s, "b", DeformerType::Skinning, true);
        assert_eq!(s.entries[1].order, 1);
    }
}
