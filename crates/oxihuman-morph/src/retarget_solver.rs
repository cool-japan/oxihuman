//! Retarget solver — maps morph target weights from a source rig to a different target rig.

// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Configuration for the retarget solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetargetSolverConfig {
    /// Global scale applied to all mapped weights.
    pub global_scale: f32,
    /// Whether unmapped source weights are discarded (true) or passed through as-is (false).
    pub discard_unmapped: bool,
}

/// A single mapping from a source morph name to a target morph name.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetargetMapping {
    pub source: String,
    pub target: String,
    pub scale: f32,
}

/// The retarget solver holds a list of mappings and a global scale.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetargetSolver {
    pub config: RetargetSolverConfig,
    pub mappings: Vec<RetargetMapping>,
}

/// Returns a default `RetargetSolverConfig`.
#[allow(dead_code)]
pub fn default_retarget_config() -> RetargetSolverConfig {
    RetargetSolverConfig {
        global_scale: 1.0,
        discard_unmapped: true,
    }
}

/// Creates a new `RetargetSolver` from the given config.
#[allow(dead_code)]
pub fn new_retarget_solver(cfg: &RetargetSolverConfig) -> RetargetSolver {
    RetargetSolver {
        config: cfg.clone(),
        mappings: Vec::new(),
    }
}

/// Adds a source-to-target mapping with the given per-mapping scale.
#[allow(dead_code)]
pub fn retarget_add_mapping(solver: &mut RetargetSolver, source: &str, target: &str, scale: f32) {
    // Remove any existing mapping for this source first.
    solver.mappings.retain(|m| m.source != source);
    solver.mappings.push(RetargetMapping {
        source: source.to_string(),
        target: target.to_string(),
        scale,
    });
}

/// Solves the retargeting: for each source weight, look up the mapping and
/// emit `(target_name, weight * mapping_scale * global_scale)`.
#[allow(dead_code)]
pub fn retarget_solve(
    solver: &RetargetSolver,
    source_weights: &[(&str, f32)],
) -> Vec<(String, f32)> {
    let mut result: Vec<(String, f32)> = Vec::new();
    for &(src_name, weight) in source_weights {
        if let Some(mapping) = solver.mappings.iter().find(|m| m.source == src_name) {
            let scaled = weight * mapping.scale * solver.config.global_scale;
            result.push((mapping.target.clone(), scaled));
        } else if !solver.config.discard_unmapped {
            let scaled = weight * solver.config.global_scale;
            result.push((src_name.to_string(), scaled));
        }
    }
    result
}

/// Returns the number of active mappings.
#[allow(dead_code)]
pub fn retarget_mapping_count(solver: &RetargetSolver) -> usize {
    solver.mappings.len()
}

/// Returns true if a mapping exists for `source`.
#[allow(dead_code)]
pub fn retarget_has_mapping(solver: &RetargetSolver, source: &str) -> bool {
    solver.mappings.iter().any(|m| m.source == source)
}

/// Removes the mapping for `source`. Returns true if one was found and removed.
#[allow(dead_code)]
pub fn retarget_remove_mapping(solver: &mut RetargetSolver, source: &str) -> bool {
    let before = solver.mappings.len();
    solver.mappings.retain(|m| m.source != source);
    solver.mappings.len() < before
}

/// Removes all mappings.
#[allow(dead_code)]
pub fn retarget_clear(solver: &mut RetargetSolver) {
    solver.mappings.clear();
}

/// Sets the global scale factor applied to all mapped weights.
#[allow(dead_code)]
pub fn retarget_set_global_scale(solver: &mut RetargetSolver, scale: f32) {
    solver.config.global_scale = scale;
}

/// Returns the list of target names that have at least one mapping.
#[allow(dead_code)]
pub fn retarget_mapped_targets(solver: &RetargetSolver) -> Vec<&str> {
    solver.mappings.iter().map(|m| m.target.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_retarget_config();
        assert!((cfg.global_scale - 1.0).abs() < 1e-6);
        assert!(cfg.discard_unmapped);
    }

    #[test]
    fn test_new_solver_empty() {
        let cfg = default_retarget_config();
        let solver = new_retarget_solver(&cfg);
        assert_eq!(retarget_mapping_count(&solver), 0);
    }

    #[test]
    fn test_add_and_has_mapping() {
        let cfg = default_retarget_config();
        let mut solver = new_retarget_solver(&cfg);
        retarget_add_mapping(&mut solver, "smile", "mouth_smile", 1.0);
        assert!(retarget_has_mapping(&solver, "smile"));
        assert!(!retarget_has_mapping(&solver, "frown"));
        assert_eq!(retarget_mapping_count(&solver), 1);
    }

    #[test]
    fn test_solve_basic() {
        let cfg = default_retarget_config();
        let mut solver = new_retarget_solver(&cfg);
        retarget_add_mapping(&mut solver, "smile", "mouth_smile", 0.5);
        let result = retarget_solve(&solver, &[("smile", 1.0)]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "mouth_smile");
        assert!((result[0].1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_solve_global_scale() {
        let cfg = default_retarget_config();
        let mut solver = new_retarget_solver(&cfg);
        retarget_add_mapping(&mut solver, "blink", "eye_close", 1.0);
        retarget_set_global_scale(&mut solver, 2.0);
        let result = retarget_solve(&solver, &[("blink", 0.5)]);
        assert!((result[0].1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_discard_unmapped() {
        let cfg = default_retarget_config(); // discard_unmapped = true
        let solver = new_retarget_solver(&cfg);
        let result = retarget_solve(&solver, &[("smile", 0.8)]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_passthrough_unmapped() {
        let cfg = RetargetSolverConfig {
            global_scale: 1.0,
            discard_unmapped: false,
        };
        let solver = new_retarget_solver(&cfg);
        let result = retarget_solve(&solver, &[("smile", 0.8)]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "smile");
        assert!((result[0].1 - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_remove_mapping() {
        let cfg = default_retarget_config();
        let mut solver = new_retarget_solver(&cfg);
        retarget_add_mapping(&mut solver, "smile", "mouth_smile", 1.0);
        assert!(retarget_remove_mapping(&mut solver, "smile"));
        assert!(!retarget_has_mapping(&solver, "smile"));
        assert!(!retarget_remove_mapping(&mut solver, "smile")); // second removal = false
    }

    #[test]
    fn test_clear() {
        let cfg = default_retarget_config();
        let mut solver = new_retarget_solver(&cfg);
        retarget_add_mapping(&mut solver, "a", "b", 1.0);
        retarget_add_mapping(&mut solver, "c", "d", 1.0);
        retarget_clear(&mut solver);
        assert_eq!(retarget_mapping_count(&solver), 0);
    }

    #[test]
    fn test_mapped_targets() {
        let cfg = default_retarget_config();
        let mut solver = new_retarget_solver(&cfg);
        retarget_add_mapping(&mut solver, "smile", "mouth_smile", 1.0);
        retarget_add_mapping(&mut solver, "blink", "eye_close", 1.0);
        let targets = retarget_mapped_targets(&solver);
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&"mouth_smile"));
        assert!(targets.contains(&"eye_close"));
    }

    #[test]
    fn test_overwrite_mapping() {
        let cfg = default_retarget_config();
        let mut solver = new_retarget_solver(&cfg);
        retarget_add_mapping(&mut solver, "smile", "mouth_smile", 1.0);
        retarget_add_mapping(&mut solver, "smile", "mouth_grin", 0.5); // overwrite
        assert_eq!(retarget_mapping_count(&solver), 1);
        let result = retarget_solve(&solver, &[("smile", 1.0)]);
        assert_eq!(result[0].0, "mouth_grin");
        assert!((result[0].1 - 0.5).abs() < 1e-6);
    }
}
