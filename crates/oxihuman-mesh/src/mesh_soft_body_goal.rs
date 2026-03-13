// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Soft body goal/pin vertex — specifies goal shape attraction for soft-body simulation.

/// Goal strength entry for a single vertex.
#[derive(Debug, Clone)]
pub struct SoftGoalVertex {
    pub vertex_index: usize,
    pub goal_strength: f32,
}

/// Soft body goal descriptor.
#[derive(Debug, Clone)]
pub struct SoftBodyGoal {
    pub vertices: Vec<SoftGoalVertex>,
    pub default_strength: f32,
    pub label: String,
}

/// Create a new soft body goal.
pub fn new_soft_body_goal(default_strength: f32, label: &str) -> SoftBodyGoal {
    SoftBodyGoal {
        vertices: Vec::new(),
        default_strength: default_strength.clamp(0.0, 1.0),
        label: label.to_owned(),
    }
}

/// Pin a vertex with a specific goal strength.
pub fn pin_vertex(goal: &mut SoftBodyGoal, vertex_index: usize, strength: f32) {
    let s = strength.clamp(0.0, 1.0);
    if let Some(v) = goal
        .vertices
        .iter_mut()
        .find(|v| v.vertex_index == vertex_index)
    {
        v.goal_strength = s;
    } else {
        goal.vertices.push(SoftGoalVertex {
            vertex_index,
            goal_strength: s,
        });
    }
}

/// Number of pinned vertices.
pub fn pinned_vertex_count(goal: &SoftBodyGoal) -> usize {
    goal.vertices.len()
}

/// Average goal strength across all pinned vertices.
pub fn average_goal_strength(goal: &SoftBodyGoal) -> f32 {
    if goal.vertices.is_empty() {
        return goal.default_strength;
    }
    let sum: f32 = goal.vertices.iter().map(|v| v.goal_strength).sum();
    sum / goal.vertices.len() as f32
}

/// Is the given vertex fully pinned (strength >= 0.999)?
pub fn is_fully_pinned(goal: &SoftBodyGoal, vertex_index: usize) -> bool {
    goal.vertices
        .iter()
        .find(|v| v.vertex_index == vertex_index)
        .map(|v| v.goal_strength >= 0.999)
        .unwrap_or(false)
}

/// Serialize the goal to a JSON-style string.
pub fn soft_body_goal_to_json(goal: &SoftBodyGoal) -> String {
    format!(
        r#"{{"label":"{}", "default_strength":{:.4}, "pinned_count":{}}}"#,
        goal.label,
        goal.default_strength,
        goal.vertices.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_goal_has_no_pins() {
        /* fresh goal has no pinned vertices */
        let g = new_soft_body_goal(0.5, "body");
        assert_eq!(pinned_vertex_count(&g), 0);
    }

    #[test]
    fn pin_vertex_increases_count() {
        /* pinning a vertex increments count */
        let mut g = new_soft_body_goal(0.5, "body");
        pin_vertex(&mut g, 0, 1.0);
        assert_eq!(pinned_vertex_count(&g), 1);
    }

    #[test]
    fn pin_same_vertex_twice_does_not_duplicate() {
        /* pinning the same vertex twice should update, not duplicate */
        let mut g = new_soft_body_goal(0.5, "body");
        pin_vertex(&mut g, 0, 0.5);
        pin_vertex(&mut g, 0, 0.9);
        assert_eq!(pinned_vertex_count(&g), 1);
    }

    #[test]
    fn is_fully_pinned_strength_one() {
        /* vertex with strength 1.0 is fully pinned */
        let mut g = new_soft_body_goal(0.5, "body");
        pin_vertex(&mut g, 3, 1.0);
        assert!(is_fully_pinned(&g, 3));
    }

    #[test]
    fn is_fully_pinned_partial_strength_false() {
        /* vertex with strength 0.5 is not fully pinned */
        let mut g = new_soft_body_goal(0.5, "body");
        pin_vertex(&mut g, 3, 0.5);
        assert!(!is_fully_pinned(&g, 3));
    }

    #[test]
    fn average_strength_correct() {
        /* average of 0.4 and 0.6 is 0.5 */
        let mut g = new_soft_body_goal(0.5, "body");
        pin_vertex(&mut g, 0, 0.4);
        pin_vertex(&mut g, 1, 0.6);
        assert!((average_goal_strength(&g) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn default_strength_clamped_to_one() {
        /* strength above 1 should be clamped */
        let g = new_soft_body_goal(2.0, "body");
        assert!((g.default_strength - 1.0).abs() < 1e-5);
    }

    #[test]
    fn json_contains_label() {
        /* JSON should include label */
        let g = new_soft_body_goal(0.5, "softGoal");
        assert!(soft_body_goal_to_json(&g).contains("softGoal"));
    }

    #[test]
    fn average_empty_returns_default() {
        /* empty vertex list average returns default_strength */
        let g = new_soft_body_goal(0.7, "body");
        assert!((average_goal_strength(&g) - 0.7).abs() < 1e-5);
    }
}
