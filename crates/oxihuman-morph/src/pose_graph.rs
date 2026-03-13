// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

pub type PoseParams = HashMap<String, f32>;

/// Easing function for transitions
#[derive(Clone, Debug)]
pub enum Easing {
    Linear,
    EaseIn,    // quadratic ease-in
    EaseOut,   // quadratic ease-out
    EaseInOut, // smooth cubic
    Spring,    // spring-like overshoot
}

/// A transition between two poses
#[derive(Clone, Debug)]
pub struct PoseTransition {
    pub from: String,
    pub to: String,
    pub duration: f32,
    pub easing: Easing,
    /// Condition that triggers this transition (parameter name + threshold)
    pub trigger: Option<(String, f32)>,
}

impl PoseTransition {
    pub fn new(from: impl Into<String>, to: impl Into<String>, duration: f32) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            duration,
            easing: Easing::Linear,
            trigger: None,
        }
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn with_trigger(mut self, param: impl Into<String>, threshold: f32) -> Self {
        self.trigger = Some((param.into(), threshold));
        self
    }

    /// Evaluate easing at t in `[0,1]`
    pub fn ease(easing: &Easing, t: f32) -> f32 {
        apply_easing(easing, t)
    }
}

/// A node in the pose graph
#[derive(Clone, Debug)]
pub struct PoseNode {
    pub name: String,
    pub params: PoseParams,
    pub loop_animation: bool,
}

impl PoseNode {
    pub fn new(name: impl Into<String>, params: PoseParams) -> Self {
        Self {
            name: name.into(),
            params,
            loop_animation: false,
        }
    }
}

/// Pose graph state machine
pub struct PoseGraph {
    nodes: HashMap<String, PoseNode>,
    transitions: Vec<PoseTransition>,
    current_state: String,
    target_state: Option<String>,
    transition_progress: f32,
    transition_duration: f32,
    active_transition: Option<usize>,
}

impl PoseGraph {
    pub fn new(initial_state: &str, initial_params: PoseParams) -> Self {
        let node = PoseNode::new(initial_state, initial_params);
        let mut nodes = HashMap::new();
        nodes.insert(initial_state.to_string(), node);
        Self {
            nodes,
            transitions: Vec::new(),
            current_state: initial_state.to_string(),
            target_state: None,
            transition_progress: 0.0,
            transition_duration: 0.0,
            active_transition: None,
        }
    }

    pub fn add_node(&mut self, node: PoseNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    pub fn add_transition(&mut self, transition: PoseTransition) {
        self.transitions.push(transition);
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    pub fn current_state(&self) -> &str {
        &self.current_state
    }

    pub fn is_transitioning(&self) -> bool {
        self.target_state.is_some()
    }

    pub fn transition_progress(&self) -> f32 {
        self.transition_progress
    }

    /// Trigger a transition to the named state.
    /// Returns false if no transition is defined or already in that state.
    pub fn go_to(&mut self, state: &str) -> bool {
        if self.current_state == state {
            return false;
        }
        // Find a transition from current_state to state
        let found = self
            .transitions
            .iter()
            .enumerate()
            .find(|(_, t)| t.from == self.current_state && t.to == state);

        if let Some((idx, t)) = found {
            let duration = t.duration;
            self.target_state = Some(state.to_string());
            self.transition_duration = duration;
            self.transition_progress = 0.0;
            self.active_transition = Some(idx);
            true
        } else {
            false
        }
    }

    /// Update the state machine by dt seconds
    pub fn update(&mut self, dt: f32) {
        if self.target_state.is_none() {
            return;
        }
        let dur = if self.transition_duration > 0.0 {
            self.transition_duration
        } else {
            1.0
        };
        self.transition_progress += dt / dur;
        if self.transition_progress >= 1.0 {
            // Complete the transition
            if let Some(target) = self.target_state.take() {
                self.current_state = target;
            }
            self.transition_progress = 0.0;
            self.transition_duration = 0.0;
            self.active_transition = None;
        }
    }

    /// Check and auto-trigger transitions based on param values
    pub fn check_triggers(&mut self, params: &PoseParams) {
        if self.is_transitioning() {
            return;
        }
        // Collect candidates first to avoid borrow conflict
        let candidates: Vec<String> = self
            .transitions
            .iter()
            .filter(|t| t.from == self.current_state)
            .filter_map(|t| {
                if let Some((ref param_name, threshold)) = t.trigger {
                    if let Some(&val) = params.get(param_name) {
                        if val >= threshold {
                            return Some(t.to.clone());
                        }
                    }
                }
                None
            })
            .collect();

        if let Some(target) = candidates.into_iter().next() {
            self.go_to(&target);
        }
    }

    /// Get current interpolated parameter values
    pub fn evaluate(&self) -> PoseParams {
        let current_node = match self.nodes.get(&self.current_state) {
            Some(n) => n,
            None => return PoseParams::new(),
        };

        if !self.is_transitioning() {
            return current_node.params.clone();
        }

        let target_name = match &self.target_state {
            Some(s) => s,
            None => return current_node.params.clone(),
        };

        let target_node = match self.nodes.get(target_name) {
            Some(n) => n,
            None => return current_node.params.clone(),
        };

        // Get easing factor
        let eased_t = if let Some(idx) = self.active_transition {
            if let Some(trans) = self.transitions.get(idx) {
                apply_easing(&trans.easing, self.transition_progress.clamp(0.0, 1.0))
            } else {
                self.transition_progress.clamp(0.0, 1.0)
            }
        } else {
            self.transition_progress.clamp(0.0, 1.0)
        };

        // Lerp between current and target params
        let mut result = current_node.params.clone();
        // Add keys from target that might not be in current
        for (k, &tv) in &target_node.params {
            let cv = current_node.params.get(k).copied().unwrap_or(0.0);
            result.insert(k.clone(), cv + eased_t * (tv - cv));
        }
        // Keys only in current stay as-is (already in result, lerp toward 0 is not desired — keep them)
        result
    }

    /// Get all reachable states from current via BFS
    pub fn reachable_states(&self) -> Vec<&str> {
        let mut visited: Vec<&str> = Vec::new();
        let mut queue: VecDeque<&str> = VecDeque::new();
        queue.push_back(&self.current_state);

        while let Some(state) = queue.pop_front() {
            if visited.contains(&state) {
                continue;
            }
            visited.push(state);
            for t in &self.transitions {
                if t.from == state && !visited.contains(&t.to.as_str()) {
                    queue.push_back(&t.to);
                }
            }
        }

        // Remove the current state itself from results (reachable = others)
        visited
            .into_iter()
            .filter(|&s| s != self.current_state)
            .collect()
    }
}

/// Apply easing function to t
pub fn apply_easing(easing: &Easing, t: f32) -> f32 {
    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        Easing::EaseInOut => t * t * (3.0 - 2.0 * t),
        Easing::Spring => 1.0 - (1.0 - t).powi(3) * (1.0 + 3.0 * t),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    fn make_params(pairs: &[(&str, f32)]) -> PoseParams {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_ease_linear() {
        assert!(approx_eq(apply_easing(&Easing::Linear, 0.0), 0.0));
        assert!(approx_eq(apply_easing(&Easing::Linear, 0.5), 0.5));
        assert!(approx_eq(apply_easing(&Easing::Linear, 1.0), 1.0));
    }

    #[test]
    fn test_ease_ease_in() {
        assert!(approx_eq(apply_easing(&Easing::EaseIn, 0.0), 0.0));
        assert!(approx_eq(apply_easing(&Easing::EaseIn, 0.5), 0.25));
        assert!(approx_eq(apply_easing(&Easing::EaseIn, 1.0), 1.0));
    }

    #[test]
    fn test_ease_ease_out() {
        assert!(approx_eq(apply_easing(&Easing::EaseOut, 0.0), 0.0));
        assert!(approx_eq(apply_easing(&Easing::EaseOut, 0.5), 0.75));
        assert!(approx_eq(apply_easing(&Easing::EaseOut, 1.0), 1.0));
    }

    #[test]
    fn test_ease_ease_in_out() {
        assert!(approx_eq(apply_easing(&Easing::EaseInOut, 0.0), 0.0));
        // smoothstep at 0.5: 0.5*0.5*(3-1) = 0.5
        assert!(approx_eq(apply_easing(&Easing::EaseInOut, 0.5), 0.5));
        assert!(approx_eq(apply_easing(&Easing::EaseInOut, 1.0), 1.0));
    }

    #[test]
    fn test_ease_spring() {
        // At t=0: 1 - 1^3 * (1+0) = 0
        assert!(approx_eq(apply_easing(&Easing::Spring, 0.0), 0.0));
        // At t=1: 1 - 0^3 * (1+3) = 1
        assert!(approx_eq(apply_easing(&Easing::Spring, 1.0), 1.0));
        // At t=0.5: 1 - 0.5^3 * (1 + 1.5) = 1 - 0.125*2.5 = 1 - 0.3125 = 0.6875
        assert!(approx_eq(apply_easing(&Easing::Spring, 0.5), 0.6875));
    }

    #[test]
    fn test_pose_graph_new() {
        let params = make_params(&[("weight", 0.0)]);
        let graph = PoseGraph::new("idle", params);
        assert_eq!(graph.current_state(), "idle");
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.transition_count(), 0);
        assert!(!graph.is_transitioning());
        assert!(approx_eq(graph.transition_progress(), 0.0));
    }

    #[test]
    fn test_add_node() {
        let params = make_params(&[("weight", 0.0)]);
        let mut graph = PoseGraph::new("idle", params);
        let walk_params = make_params(&[("weight", 1.0)]);
        graph.add_node(PoseNode::new("walk", walk_params));
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_add_transition() {
        let params = make_params(&[("weight", 0.0)]);
        let mut graph = PoseGraph::new("idle", params);
        let walk_params = make_params(&[("weight", 1.0)]);
        graph.add_node(PoseNode::new("walk", walk_params));
        graph.add_transition(PoseTransition::new("idle", "walk", 0.5));
        assert_eq!(graph.transition_count(), 1);
    }

    #[test]
    fn test_go_to_valid() {
        let params = make_params(&[("weight", 0.0)]);
        let mut graph = PoseGraph::new("idle", params);
        let walk_params = make_params(&[("weight", 1.0)]);
        graph.add_node(PoseNode::new("walk", walk_params));
        graph.add_transition(PoseTransition::new("idle", "walk", 0.5));

        let result = graph.go_to("walk");
        assert!(result);
        assert!(graph.is_transitioning());
        assert_eq!(graph.current_state(), "idle");
    }

    #[test]
    fn test_go_to_invalid() {
        let params = make_params(&[("weight", 0.0)]);
        let mut graph = PoseGraph::new("idle", params);
        // No transition defined
        let result = graph.go_to("run");
        assert!(!result);
        assert!(!graph.is_transitioning());
    }

    #[test]
    fn test_update_completes_transition() {
        let params = make_params(&[("weight", 0.0)]);
        let mut graph = PoseGraph::new("idle", params);
        let walk_params = make_params(&[("weight", 1.0)]);
        graph.add_node(PoseNode::new("walk", walk_params));
        graph.add_transition(PoseTransition::new("idle", "walk", 1.0));

        graph.go_to("walk");
        assert!(graph.is_transitioning());

        // Advance past the end
        graph.update(1.5);
        assert!(!graph.is_transitioning());
        assert_eq!(graph.current_state(), "walk");
    }

    #[test]
    fn test_evaluate_no_transition() {
        let params = make_params(&[("weight", 0.5), ("height", 1.8)]);
        let graph = PoseGraph::new("idle", params.clone());
        let evaluated = graph.evaluate();
        assert!(approx_eq(
            *evaluated.get("weight").expect("should succeed"),
            0.5
        ));
        assert!(approx_eq(
            *evaluated.get("height").expect("should succeed"),
            1.8
        ));
    }

    #[test]
    fn test_evaluate_mid_transition() {
        let idle_params = make_params(&[("weight", 0.0)]);
        let mut graph = PoseGraph::new("idle", idle_params);
        let walk_params = make_params(&[("weight", 1.0)]);
        graph.add_node(PoseNode::new("walk", walk_params));
        graph.add_transition(PoseTransition::new("idle", "walk", 1.0).with_easing(Easing::Linear));

        graph.go_to("walk");
        // Advance halfway
        graph.update(0.5);
        assert!(graph.is_transitioning());

        let evaluated = graph.evaluate();
        let w = *evaluated.get("weight").expect("should succeed");
        // At t=0.5 linear: weight should be ~0.5
        assert!(approx_eq(w, 0.5));
    }

    #[test]
    fn test_check_triggers() {
        let idle_params = make_params(&[("speed", 0.0)]);
        let mut graph = PoseGraph::new("idle", idle_params);
        let walk_params = make_params(&[("speed", 1.0)]);
        graph.add_node(PoseNode::new("walk", walk_params));
        graph.add_transition(PoseTransition::new("idle", "walk", 0.5).with_trigger("speed", 0.5));

        // Trigger with speed below threshold — should NOT transition
        let low_params = make_params(&[("speed", 0.3)]);
        graph.check_triggers(&low_params);
        assert!(!graph.is_transitioning());

        // Trigger with speed above threshold — should transition
        let high_params = make_params(&[("speed", 0.8)]);
        graph.check_triggers(&high_params);
        assert!(graph.is_transitioning());
    }

    #[test]
    fn test_reachable_states() {
        let idle_params = make_params(&[("w", 0.0)]);
        let mut graph = PoseGraph::new("idle", idle_params);
        graph.add_node(PoseNode::new("walk", make_params(&[("w", 1.0)])));
        graph.add_node(PoseNode::new("run", make_params(&[("w", 2.0)])));
        graph.add_transition(PoseTransition::new("idle", "walk", 0.3));
        graph.add_transition(PoseTransition::new("walk", "run", 0.3));

        let reachable = graph.reachable_states();
        assert!(reachable.contains(&"walk"));
        assert!(reachable.contains(&"run"));
        assert!(!reachable.contains(&"idle"));
    }
}
