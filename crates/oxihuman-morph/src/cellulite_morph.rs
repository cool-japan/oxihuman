// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cellulite texture morph stub.

/// Cellulite grade (Nürnberger–Müller scale).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CelluliteGrade {
    Grade0,
    Grade1,
    Grade2,
    Grade3,
}

/// Cellulite morph controller.
#[derive(Debug, Clone)]
pub struct CelluliteMorph {
    pub grade: CelluliteGrade,
    pub coverage: f32,
    pub depth: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl CelluliteMorph {
    pub fn new(morph_count: usize) -> Self {
        CelluliteMorph {
            grade: CelluliteGrade::Grade0,
            coverage: 0.0,
            depth: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new cellulite morph.
pub fn new_cellulite_morph(morph_count: usize) -> CelluliteMorph {
    CelluliteMorph::new(morph_count)
}

/// Set cellulite grade.
pub fn clm_set_grade(morph: &mut CelluliteMorph, grade: CelluliteGrade) {
    morph.grade = grade;
}

/// Set coverage (0.0 = no coverage, 1.0 = full coverage).
pub fn clm_set_coverage(morph: &mut CelluliteMorph, coverage: f32) {
    morph.coverage = coverage.clamp(0.0, 1.0);
}

/// Set dimple depth factor.
pub fn clm_set_depth(morph: &mut CelluliteMorph, depth: f32) {
    morph.depth = depth.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from coverage).
pub fn clm_evaluate(morph: &CelluliteMorph) -> Vec<f32> {
    /* Stub: uniform weight from coverage */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.coverage; morph.morph_count]
}

/// Enable or disable.
pub fn clm_set_enabled(morph: &mut CelluliteMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn clm_to_json(morph: &CelluliteMorph) -> String {
    let grade = match morph.grade {
        CelluliteGrade::Grade0 => "grade0",
        CelluliteGrade::Grade1 => "grade1",
        CelluliteGrade::Grade2 => "grade2",
        CelluliteGrade::Grade3 => "grade3",
    };
    format!(
        r#"{{"grade":"{}","coverage":{},"depth":{},"morph_count":{},"enabled":{}}}"#,
        grade, morph.coverage, morph.depth, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_grade() {
        let m = new_cellulite_morph(4);
        assert_eq!(
            m.grade,
            CelluliteGrade::Grade0 /* default grade must be Grade0 */
        );
    }

    #[test]
    fn test_set_grade() {
        let mut m = new_cellulite_morph(4);
        clm_set_grade(&mut m, CelluliteGrade::Grade3);
        assert_eq!(m.grade, CelluliteGrade::Grade3 /* grade must be set */);
    }

    #[test]
    fn test_coverage_clamp_high() {
        let mut m = new_cellulite_morph(4);
        clm_set_coverage(&mut m, 1.5);
        assert!((m.coverage - 1.0).abs() < 1e-6 /* coverage clamped to 1.0 */);
    }

    #[test]
    fn test_coverage_clamp_low() {
        let mut m = new_cellulite_morph(4);
        clm_set_coverage(&mut m, -0.1);
        assert!(m.coverage.abs() < 1e-6 /* coverage clamped to 0.0 */);
    }

    #[test]
    fn test_depth_clamp() {
        let mut m = new_cellulite_morph(4);
        clm_set_depth(&mut m, 2.0);
        assert!((m.depth - 1.0).abs() < 1e-6 /* depth clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_cellulite_morph(6);
        clm_set_coverage(&mut m, 0.5);
        assert_eq!(
            clm_evaluate(&m).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_cellulite_morph(4);
        clm_set_enabled(&mut m, false);
        assert!(clm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_grade() {
        let m = new_cellulite_morph(4);
        let j = clm_to_json(&m);
        assert!(j.contains("\"grade\"") /* JSON must have grade */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_cellulite_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }
}
