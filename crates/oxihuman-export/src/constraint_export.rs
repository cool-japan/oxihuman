// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Constraint export.

/* ── legacy API (kept for backward compat) ── */

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintKind {
    CopyLocation,
    CopyRotation,
    CopyScale,
    LimitRotation,
    LimitLocation,
}

#[derive(Debug, Clone)]
pub struct ConstraintExport {
    pub name: String,
    pub kind: ConstraintKind,
    pub target: String,
    pub influence: f32,
}

pub fn new_constraint_export(name: &str, kind: ConstraintKind, target: &str) -> ConstraintExport {
    ConstraintExport {
        name: name.to_string(),
        kind,
        target: target.to_string(),
        influence: 1.0,
    }
}

pub fn constraint_is_active(constraint: &ConstraintExport) -> bool {
    constraint.influence > 0.0
}

pub fn constraint_to_json_legacy(constraint: &ConstraintExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"kind\":\"{}\",\"target\":\"{}\",\"influence\":{}}}",
        constraint.name,
        constraint_kind_name(constraint),
        constraint.target,
        constraint.influence
    )
}

pub fn constraint_kind_name(constraint: &ConstraintExport) -> &'static str {
    match constraint.kind {
        ConstraintKind::CopyLocation => "CopyLocation",
        ConstraintKind::CopyRotation => "CopyRotation",
        ConstraintKind::CopyScale => "CopyScale",
        ConstraintKind::LimitRotation => "LimitRotation",
        ConstraintKind::LimitLocation => "LimitLocation",
    }
}

pub fn constraint_set_influence(constraint: &mut ConstraintExport, influence: f32) {
    constraint.influence = influence.clamp(0.0, 1.0);
}

pub fn constraint_validate(constraint: &ConstraintExport) -> bool {
    !constraint.name.is_empty()
        && !constraint.target.is_empty()
        && (0.0..=1.0).contains(&constraint.influence)
}

/* ── spec functions (wave 150B) ── */

/// Spec-style constraint data.
#[derive(Debug, Clone)]
pub struct ConstraintData {
    pub name: String,
    pub constraint_type: String,
    pub target: String,
    pub influence: f32,
    pub active: bool,
}

/// Create a new `ConstraintData`.
pub fn new_constraint_data(name: &str, constraint_type: &str, target: &str) -> ConstraintData {
    ConstraintData {
        name: name.to_string(),
        constraint_type: constraint_type.to_string(),
        target: target.to_string(),
        influence: 1.0,
        active: true,
    }
}

/// Serialize a `ConstraintData` to JSON.
pub fn constraint_to_json(c: &ConstraintData) -> String {
    format!(
        "{{\"name\":\"{}\",\"type\":\"{}\",\"target\":\"{}\",\"influence\":{},\"active\":{}}}",
        c.name, c.constraint_type, c.target, c.influence, c.active
    )
}

/// Serialize multiple constraints to a JSON array.
pub fn constraints_to_json(cs: &[ConstraintData]) -> String {
    let inner: Vec<String> = cs.iter().map(constraint_to_json).collect();
    format!("[{}]", inner.join(","))
}

/// Returns `true` if the constraint is active and influence > 0.
pub fn constraint_is_active_spec(c: &ConstraintData) -> bool {
    c.active && c.influence > 0.0
}

/// Count of constraints in a slice.
pub fn constraint_count(cs: &[ConstraintData]) -> usize {
    cs.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_constraint_data() {
        let c = new_constraint_data("c", "COPY_LOC", "Head");
        assert_eq!(c.name, "c");
        assert!(c.active);
    }

    #[test]
    fn test_constraint_to_json() {
        let c = new_constraint_data("c", "COPY_ROT", "Spine");
        let j = constraint_to_json(&c);
        assert!(j.contains("COPY_ROT"));
    }

    #[test]
    fn test_constraints_to_json() {
        let cs = vec![
            new_constraint_data("a", "COPY_LOC", "X"),
            new_constraint_data("b", "LIMIT_ROT", "Y"),
        ];
        let j = constraints_to_json(&cs);
        assert!(j.starts_with('['));
    }

    #[test]
    fn test_constraint_is_active_spec() {
        let c = new_constraint_data("c", "T", "t");
        assert!(constraint_is_active_spec(&c));
    }

    #[test]
    fn test_constraint_count() {
        let cs = vec![new_constraint_data("a", "T", "x")];
        assert_eq!(constraint_count(&cs), 1);
    }
}
