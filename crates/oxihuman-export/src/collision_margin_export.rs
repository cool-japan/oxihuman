// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Collision margin export: per-shape collision margin metadata.

/// Shape type for collision margin.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarginShapeType {
    Box,
    Sphere,
    Capsule,
    ConvexHull,
    TriangleMesh,
}

/// Collision margin entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionMarginEntry {
    pub name: String,
    pub shape_type: MarginShapeType,
    pub margin: f32,
    pub enabled: bool,
}

/// Collision margin export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionMarginExport {
    pub entries: Vec<CollisionMarginEntry>,
}

/// Create a new collision margin export.
#[allow(dead_code)]
pub fn new_collision_margin_export() -> CollisionMarginExport {
    CollisionMarginExport {
        entries: Vec::new(),
    }
}

/// Add a margin entry.
#[allow(dead_code)]
pub fn add_margin_entry(exp: &mut CollisionMarginExport, entry: CollisionMarginEntry) {
    exp.entries.push(entry);
}

/// Entry count.
#[allow(dead_code)]
pub fn margin_entry_count(exp: &CollisionMarginExport) -> usize {
    exp.entries.len()
}

/// Find entry by name.
#[allow(dead_code)]
pub fn find_margin_entry<'a>(
    exp: &'a CollisionMarginExport,
    name: &str,
) -> Option<&'a CollisionMarginEntry> {
    exp.entries.iter().find(|e| e.name == name)
}

/// Average margin across enabled entries.
#[allow(dead_code)]
pub fn avg_margin(exp: &CollisionMarginExport) -> f32 {
    let enabled: Vec<_> = exp.entries.iter().filter(|e| e.enabled).collect();
    if enabled.is_empty() {
        return 0.0;
    }
    enabled.iter().map(|e| e.margin).sum::<f32>() / enabled.len() as f32
}

/// Maximum margin.
#[allow(dead_code)]
pub fn max_margin(exp: &CollisionMarginExport) -> f32 {
    exp.entries.iter().map(|e| e.margin).fold(0.0_f32, f32::max)
}

/// Validate: margins non-negative.
#[allow(dead_code)]
pub fn validate_margins(exp: &CollisionMarginExport) -> bool {
    exp.entries.iter().all(|e| e.margin >= 0.0)
}

/// Shape type name.
#[allow(dead_code)]
pub fn shape_type_name(t: MarginShapeType) -> &'static str {
    match t {
        MarginShapeType::Box => "box",
        MarginShapeType::Sphere => "sphere",
        MarginShapeType::Capsule => "capsule",
        MarginShapeType::ConvexHull => "convex_hull",
        MarginShapeType::TriangleMesh => "triangle_mesh",
    }
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn collision_margin_to_json(exp: &CollisionMarginExport) -> String {
    format!(
        "{{\"entry_count\":{},\"avg_margin\":{}}}",
        margin_entry_count(exp),
        avg_margin(exp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, m: f32) -> CollisionMarginEntry {
        CollisionMarginEntry {
            name: name.to_string(),
            shape_type: MarginShapeType::Box,
            margin: m,
            enabled: true,
        }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_collision_margin_export();
        assert_eq!(margin_entry_count(&exp), 0);
    }

    #[test]
    fn add_entry_increments() {
        let mut exp = new_collision_margin_export();
        add_margin_entry(&mut exp, entry("a", 0.01));
        assert_eq!(margin_entry_count(&exp), 1);
    }

    #[test]
    fn find_existing() {
        let mut exp = new_collision_margin_export();
        add_margin_entry(&mut exp, entry("body", 0.02));
        assert!(find_margin_entry(&exp, "body").is_some());
    }

    #[test]
    fn find_missing_none() {
        let exp = new_collision_margin_export();
        assert!(find_margin_entry(&exp, "ghost").is_none());
    }

    #[test]
    fn avg_margin_correct() {
        let mut exp = new_collision_margin_export();
        add_margin_entry(&mut exp, entry("a", 0.1));
        add_margin_entry(&mut exp, entry("b", 0.3));
        assert!((avg_margin(&exp) - 0.2).abs() < 1e-5);
    }

    #[test]
    fn max_margin_correct() {
        let mut exp = new_collision_margin_export();
        add_margin_entry(&mut exp, entry("a", 0.05));
        add_margin_entry(&mut exp, entry("b", 0.5));
        assert!((max_margin(&exp) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn validate_valid() {
        let mut exp = new_collision_margin_export();
        add_margin_entry(&mut exp, entry("x", 0.01));
        assert!(validate_margins(&exp));
    }

    #[test]
    fn shape_type_name_box() {
        assert_eq!(shape_type_name(MarginShapeType::Box), "box");
    }

    #[test]
    fn json_contains_entry_count() {
        let exp = new_collision_margin_export();
        let j = collision_margin_to_json(&exp);
        assert!(j.contains("entry_count"));
    }

    #[test]
    fn margin_in_range() {
        let e = entry("t", 0.05);
        assert!((0.0..=1.0).contains(&e.margin));
    }
}
