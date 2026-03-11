// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// Tracks dependencies between morph targets (A depends on B).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphDependency {
    /// key depends on values
    pub deps: HashMap<String, Vec<String>>,
}

/// Create a new empty dependency tracker.
#[allow(dead_code)]
pub fn new_morph_dependency() -> MorphDependency {
    MorphDependency {
        deps: HashMap::new(),
    }
}

/// Record that `target` depends on `dependency`.
#[allow(dead_code)]
pub fn add_dependency(md: &mut MorphDependency, target: &str, dependency: &str) {
    let list = md.deps.entry(target.to_string()).or_default();
    if !list.contains(&dependency.to_string()) {
        list.push(dependency.to_string());
    }
}

/// Check if `target` has any dependency on `dependency`.
#[allow(dead_code)]
pub fn has_dependency(md: &MorphDependency, target: &str, dependency: &str) -> bool {
    md.deps
        .get(target)
        .is_some_and(|v| v.contains(&dependency.to_string()))
}

/// Return the total number of dependency edges.
#[allow(dead_code)]
pub fn dependency_count(md: &MorphDependency) -> usize {
    md.deps.values().map(|v| v.len()).sum()
}

/// Evaluate dependencies: given a set of weights, return adjusted weights.
/// If a target depends on others, its weight is multiplied by the average of its deps' weights.
#[allow(dead_code)]
pub fn evaluate_dependencies(
    md: &MorphDependency,
    weights: &HashMap<String, f32>,
) -> HashMap<String, f32> {
    let mut result = weights.clone();
    for (target, dep_list) in &md.deps {
        if dep_list.is_empty() {
            continue;
        }
        let avg: f32 = dep_list
            .iter()
            .map(|d| weights.get(d).copied().unwrap_or(0.0))
            .sum::<f32>()
            / dep_list.len() as f32;
        if let Some(w) = result.get_mut(target) {
            *w *= avg;
        }
    }
    result
}

/// Return the dependency chain for a given target (direct deps only).
#[allow(dead_code)]
pub fn dependency_chain(md: &MorphDependency, target: &str) -> Vec<String> {
    md.deps.get(target).cloned().unwrap_or_default()
}

/// Clear all dependencies.
#[allow(dead_code)]
pub fn clear_dependencies(md: &mut MorphDependency) {
    md.deps.clear();
}

/// Serialize dependencies to a JSON string.
#[allow(dead_code)]
pub fn dependencies_to_json(md: &MorphDependency) -> String {
    let mut entries: Vec<String> = md
        .deps
        .iter()
        .map(|(k, v)| {
            let deps: Vec<String> = v.iter().map(|d| format!("\"{}\"", d)).collect();
            format!("\"{}\":[{}]", k, deps.join(","))
        })
        .collect();
    entries.sort();
    format!("{{{}}}", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let md = new_morph_dependency();
        assert_eq!(dependency_count(&md), 0);
    }

    #[test]
    fn add_and_check() {
        let mut md = new_morph_dependency();
        add_dependency(&mut md, "smile", "jaw_open");
        assert!(has_dependency(&md, "smile", "jaw_open"));
    }

    #[test]
    fn no_false_positive() {
        let md = new_morph_dependency();
        assert!(!has_dependency(&md, "smile", "jaw_open"));
    }

    #[test]
    fn count_edges() {
        let mut md = new_morph_dependency();
        add_dependency(&mut md, "a", "b");
        add_dependency(&mut md, "a", "c");
        assert_eq!(dependency_count(&md), 2);
    }

    #[test]
    fn no_duplicate_deps() {
        let mut md = new_morph_dependency();
        add_dependency(&mut md, "a", "b");
        add_dependency(&mut md, "a", "b");
        assert_eq!(dependency_count(&md), 1);
    }

    #[test]
    fn chain_returns_deps() {
        let mut md = new_morph_dependency();
        add_dependency(&mut md, "x", "y");
        add_dependency(&mut md, "x", "z");
        let chain = dependency_chain(&md, "x");
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn clear_works() {
        let mut md = new_morph_dependency();
        add_dependency(&mut md, "a", "b");
        clear_dependencies(&mut md);
        assert_eq!(dependency_count(&md), 0);
    }

    #[test]
    fn evaluate_deps() {
        let mut md = new_morph_dependency();
        add_dependency(&mut md, "smile", "jaw");
        let mut weights = HashMap::new();
        weights.insert("smile".to_string(), 1.0);
        weights.insert("jaw".to_string(), 0.5);
        let result = evaluate_dependencies(&md, &weights);
        assert!((result["smile"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_json() {
        let mut md = new_morph_dependency();
        add_dependency(&mut md, "a", "b");
        let j = dependencies_to_json(&md);
        assert!(j.contains("\"a\""));
    }

    #[test]
    fn chain_empty_for_unknown() {
        let md = new_morph_dependency();
        assert!(dependency_chain(&md, "unknown").is_empty());
    }
}
