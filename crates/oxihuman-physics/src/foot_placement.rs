// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Footstep placement planner stub.

/// A footstep target.
#[derive(Debug, Clone, PartialEq)]
pub struct Footstep {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub is_left: bool,
}

/// Configuration for the footstep planner.
#[derive(Debug, Clone)]
pub struct FootPlacementConfig {
    pub step_length: f32,
    pub step_width: f32,
    pub max_steps: usize,
}

impl Default for FootPlacementConfig {
    fn default() -> Self {
        Self {
            step_length: 0.3,
            step_width: 0.1,
            max_steps: 20,
        }
    }
}

/// Generate a sequence of footsteps towards a goal.
pub fn plan_footsteps(
    start_x: f32,
    start_y: f32,
    start_yaw: f32,
    goal_x: f32,
    goal_y: f32,
    config: &FootPlacementConfig,
) -> Vec<Footstep> {
    /* stub: straight-line footstep plan */
    let dx = goal_x - start_x;
    let dy = goal_y - start_y;
    let dist = (dx * dx + dy * dy).sqrt();
    let num_steps = ((dist / config.step_length) as usize + 1).min(config.max_steps);
    let mut steps = Vec::with_capacity(num_steps);
    for i in 0..num_steps {
        let alpha = if num_steps <= 1 {
            0.0
        } else {
            i as f32 / (num_steps - 1) as f32
        };
        let side = if i % 2 == 0 { 1.0f32 } else { -1.0 };
        steps.push(Footstep {
            x: start_x + alpha * dx,
            y: start_y + alpha * dy + side * config.step_width * 0.5,
            yaw: start_yaw,
            is_left: i % 2 == 0,
        });
    }
    steps
}

/// Return the distance between two consecutive footsteps.
pub fn footstep_distance(a: &Footstep, b: &Footstep) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

/// Return whether a footstep is kinematically feasible (stub check).
pub fn footstep_feasible(step: &Footstep, max_reach: f32) -> bool {
    (step.x * step.x + step.y * step.y).sqrt() <= max_reach
}

/// Return the total path length of a footstep sequence.
pub fn footstep_path_length(steps: &[Footstep]) -> f32 {
    if steps.len() < 2 {
        return 0.0;
    }
    steps
        .windows(2)
        .map(|w| footstep_distance(&w[0], &w[1]))
        .sum()
}

/// Alternate left/right checks.
pub fn footstep_alternates(steps: &[Footstep]) -> bool {
    steps.windows(2).all(|w| w[0].is_left != w[1].is_left)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_returns_steps() {
        /* planner returns at least one step */
        let cfg = FootPlacementConfig::default();
        let steps = plan_footsteps(0.0, 0.0, 0.0, 1.0, 0.0, &cfg);
        assert!(!steps.is_empty());
    }

    #[test]
    fn test_plan_max_steps() {
        /* never exceeds max_steps */
        let cfg = FootPlacementConfig {
            max_steps: 3,
            ..Default::default()
        };
        let steps = plan_footsteps(0.0, 0.0, 0.0, 100.0, 0.0, &cfg);
        assert!(steps.len() <= 3);
    }

    #[test]
    fn test_footstep_distance_zero() {
        /* same footstep has zero distance */
        let s = Footstep {
            x: 0.0,
            y: 0.0,
            yaw: 0.0,
            is_left: true,
        };
        assert!(footstep_distance(&s, &s) < 1e-6);
    }

    #[test]
    fn test_footstep_distance_known() {
        /* 3-4-5 triangle */
        let a = Footstep {
            x: 0.0,
            y: 0.0,
            yaw: 0.0,
            is_left: true,
        };
        let b = Footstep {
            x: 3.0,
            y: 4.0,
            yaw: 0.0,
            is_left: false,
        };
        assert!((footstep_distance(&a, &b) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_feasible_step() {
        /* step within reach is feasible */
        let s = Footstep {
            x: 0.1,
            y: 0.0,
            yaw: 0.0,
            is_left: true,
        };
        assert!(footstep_feasible(&s, 1.0));
    }

    #[test]
    fn test_infeasible_step() {
        /* step beyond reach is infeasible */
        let s = Footstep {
            x: 10.0,
            y: 0.0,
            yaw: 0.0,
            is_left: true,
        };
        assert!(!footstep_feasible(&s, 1.0));
    }

    #[test]
    fn test_path_length_empty() {
        /* empty sequence has zero path length */
        assert_eq!(footstep_path_length(&[]), 0.0);
    }

    #[test]
    fn test_path_length_nonzero() {
        /* multiple steps have positive length */
        let cfg = FootPlacementConfig::default();
        let steps = plan_footsteps(0.0, 0.0, 0.0, 1.0, 0.0, &cfg);
        if steps.len() > 1 {
            assert!(footstep_path_length(&steps) > 0.0);
        }
    }

    #[test]
    fn test_footstep_alternates() {
        /* planner alternates left and right */
        let cfg = FootPlacementConfig::default();
        let steps = plan_footsteps(0.0, 0.0, 0.0, 1.0, 0.0, &cfg);
        if steps.len() > 1 {
            assert!(footstep_alternates(&steps));
        }
    }
}
