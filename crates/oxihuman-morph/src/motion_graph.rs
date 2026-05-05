// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// TransitionCondition
// ---------------------------------------------------------------------------

/// Condition that must be satisfied for a state transition to fire.
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionCondition {
    /// Fire immediately on the next update tick.
    Always,
    /// Fire after the controller has spent at least N seconds in the current
    /// state.
    AfterSeconds(f32),
    /// Fire when the named parameter is strictly above the threshold.
    ParameterAbove(String, f32),
    /// Fire when the named parameter is strictly below the threshold.
    ParameterBelow(String, f32),
    /// Fire when the named parameter is within ±0.05 of the threshold.
    ParameterEqual(String, f32),
    /// Fire when the current animation clip reaches its end (state_time >=
    /// clip duration).  Only meaningful for non-looping states.
    AtEnd,
}

// ---------------------------------------------------------------------------
// MotionTransition
// ---------------------------------------------------------------------------

/// A directed edge in the motion graph.
#[allow(dead_code)]
pub struct MotionTransition {
    /// Source state name.
    pub from_state: String,
    /// Destination state name.
    pub to_state: String,
    /// Condition that triggers the transition.
    pub condition: TransitionCondition,
    /// Cross-fade duration in seconds.
    pub blend_duration: f32,
    /// Higher priority transitions are evaluated first.
    pub priority: i32,
}

// ---------------------------------------------------------------------------
// MotionState
// ---------------------------------------------------------------------------

/// A single animation state bound to one clip.
#[allow(dead_code)]
pub struct MotionState {
    /// Unique name of this state.
    pub name: String,
    /// Name of the animation clip (resolved externally).
    pub clip_name: String,
    /// Total clip duration in seconds.
    pub duration: f32,
    /// Whether the clip should loop.
    pub loop_state: bool,
    /// Playback speed multiplier (1.0 = normal speed).
    pub speed: f32,
    /// Base morph-target weights that apply while this state is active.
    pub morph_weights: HashMap<String, f32>,
}

impl MotionState {
    /// Create a state with sensible defaults.
    pub fn new(name: impl Into<String>, clip_name: impl Into<String>, duration: f32) -> Self {
        Self {
            name: name.into(),
            clip_name: clip_name.into(),
            duration,
            loop_state: true,
            speed: 1.0,
            morph_weights: HashMap::new(),
        }
    }

    /// Fluent setter for loop_state.
    pub fn with_loop(mut self, loop_state: bool) -> Self {
        self.loop_state = loop_state;
        self
    }

    /// Fluent setter for speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Fluent setter that adds one morph weight entry.
    pub fn with_morph(mut self, key: impl Into<String>, value: f32) -> Self {
        self.morph_weights.insert(key.into(), value);
        self
    }
}

// ---------------------------------------------------------------------------
// MotionGraph
// ---------------------------------------------------------------------------

/// Animation state machine – a collection of states connected by transitions.
pub struct MotionGraph {
    states: HashMap<String, MotionState>,
    transitions: Vec<MotionTransition>,
    /// Name of the state to enter on startup.
    pub entry_state: Option<String>,
}

impl Default for MotionGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl MotionGraph {
    /// Create an empty motion graph.
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            transitions: Vec::new(),
            entry_state: None,
        }
    }

    /// Register a state.  If this is the first state and no entry state has
    /// been set, it becomes the entry state automatically.
    pub fn add_state(&mut self, state: MotionState) {
        if self.entry_state.is_none() {
            self.entry_state = Some(state.name.clone());
        }
        self.states.insert(state.name.clone(), state);
    }

    /// Register a transition edge.
    pub fn add_transition(&mut self, transition: MotionTransition) {
        self.transitions.push(transition);
    }

    /// Total number of registered states.
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// Total number of registered transitions.
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }

    /// Look up a state by name.
    pub fn get_state(&self, name: &str) -> Option<&MotionState> {
        self.states.get(name)
    }

    /// Return all transitions whose `from_state` matches `state`, sorted by
    /// descending priority so that callers evaluate highest-priority first.
    pub fn transitions_from(&self, state: &str) -> Vec<&MotionTransition> {
        let mut ts: Vec<&MotionTransition> = self
            .transitions
            .iter()
            .filter(|t| t.from_state == state)
            .collect();
        ts.sort_by_key(|b| std::cmp::Reverse(b.priority));
        ts
    }

    /// Build the canonical idle / walk / run locomotion graph.
    pub fn default_graph() -> Self {
        build_locomotion_graph()
    }
}

// ---------------------------------------------------------------------------
// MotionController
// ---------------------------------------------------------------------------

/// Runtime controller that drives a `MotionGraph`.
pub struct MotionController {
    /// The graph being controlled.
    pub graph: MotionGraph,
    /// Name of the currently active state.
    pub current_state: String,
    /// Seconds spent in the current state (resets on transition).
    pub state_time: f32,
    /// Name of the state being blended *into* (if any).
    pub blend_state: Option<String>,
    /// Seconds spent in the current blend.
    pub blend_time: f32,
    /// Total duration of the current blend.
    pub blend_duration: f32,
    /// Named runtime float parameters used by transition conditions.
    pub parameters: HashMap<String, f32>,
    /// Total time since the controller was created / reset.
    pub total_time: f32,
}

impl MotionController {
    /// Create a controller and start in the graph's entry state (or `"idle"`
    /// as a fallback).
    pub fn new(graph: MotionGraph) -> Self {
        let current_state = graph
            .entry_state
            .clone()
            .unwrap_or_else(|| "idle".to_string());
        Self {
            graph,
            current_state,
            state_time: 0.0,
            blend_state: None,
            blend_time: 0.0,
            blend_duration: 0.0,
            parameters: HashMap::new(),
            total_time: 0.0,
        }
    }

    /// Set (or overwrite) a named runtime parameter.
    pub fn set_parameter(&mut self, name: &str, value: f32) {
        self.parameters.insert(name.to_string(), value);
    }

    /// Read a named runtime parameter (0.0 if not set).
    pub fn get_parameter(&self, name: &str) -> f32 {
        self.parameters.get(name).copied().unwrap_or(0.0)
    }

    /// Advance the controller by `dt` seconds.
    ///
    /// 1. Advances `state_time` and `total_time`.
    /// 2. Checks all transitions out of the current state in priority order.
    /// 3. If a condition fires, starts a blend to the target state.
    /// 4. Advances an in-progress blend; finalises it when complete.
    pub fn update(&mut self, dt: f32) {
        self.state_time += dt;
        self.total_time += dt;

        // Advance an in-progress blend.
        if self.blend_state.is_some() {
            self.blend_time += dt;
            if self.blend_time >= self.blend_duration {
                // Snap to destination.
                let Some(dest) = self.blend_state.take() else {
                    return;
                };
                self.current_state = dest;
                self.state_time = 0.0;
                self.blend_time = 0.0;
                self.blend_duration = 0.0;
            }
            // While blending, do not evaluate new transitions.
            return;
        }

        // Evaluate outgoing transitions.
        let transitions: Vec<(String, f32)> = self
            .graph
            .transitions_from(&self.current_state.clone())
            .iter()
            .filter_map(|t| {
                if self.check_condition(&t.condition) {
                    Some((t.to_state.clone(), t.blend_duration))
                } else {
                    None
                }
            })
            .collect();

        if let Some((to_state, blend_dur)) = transitions.into_iter().next() {
            self.transition_to(&to_state, blend_dur);
        }
    }

    /// Force-start a transition to `state` with the given cross-fade duration.
    /// If `blend_duration` is 0.0, the transition is instantaneous.
    pub fn transition_to(&mut self, state: &str, blend_duration: f32) {
        if blend_duration <= 0.0 {
            self.current_state = state.to_string();
            self.state_time = 0.0;
            self.blend_state = None;
            self.blend_time = 0.0;
            self.blend_duration = 0.0;
        } else {
            self.blend_state = Some(state.to_string());
            self.blend_time = 0.0;
            self.blend_duration = blend_duration;
        }
    }

    /// Current blend weight in [0, 1].
    ///
    /// * 0.0 → 100 % current state.
    /// * 1.0 → 100 % destination state.
    ///
    /// Returns 0.0 when not blending.
    pub fn blend_weight(&self) -> f32 {
        if self.blend_duration <= 0.0 {
            return 0.0;
        }
        (self.blend_time / self.blend_duration).clamp(0.0, 1.0)
    }

    /// Evaluate blended morph weights at the current instant.
    ///
    /// When not blending, returns the current state's morph weights unchanged.
    /// When blending, returns a linear interpolation between the current and
    /// destination state morph weights.
    pub fn evaluate_morphs(&self) -> HashMap<String, f32> {
        let current_morphs = self
            .graph
            .get_state(&self.current_state)
            .map(|s| s.morph_weights.clone())
            .unwrap_or_default();

        match &self.blend_state {
            None => current_morphs,
            Some(dest_name) => {
                let dest_morphs = self
                    .graph
                    .get_state(dest_name)
                    .map(|s| s.morph_weights.clone())
                    .unwrap_or_default();
                blend_morph_maps(&current_morphs, &dest_morphs, self.blend_weight())
            }
        }
    }

    /// Check whether a single `TransitionCondition` is currently satisfied.
    pub fn check_condition(&self, cond: &TransitionCondition) -> bool {
        match cond {
            TransitionCondition::Always => true,
            TransitionCondition::AfterSeconds(secs) => self.state_time >= *secs,
            TransitionCondition::ParameterAbove(name, threshold) => {
                self.get_parameter(name) > *threshold
            }
            TransitionCondition::ParameterBelow(name, threshold) => {
                self.get_parameter(name) < *threshold
            }
            TransitionCondition::ParameterEqual(name, threshold) => {
                (self.get_parameter(name) - threshold).abs() <= 0.05
            }
            TransitionCondition::AtEnd => {
                if let Some(state) = self.graph.get_state(&self.current_state) {
                    let effective_dur = if state.speed > 0.0 {
                        state.duration / state.speed
                    } else {
                        f32::MAX
                    };
                    self.state_time >= effective_dur
                } else {
                    false
                }
            }
        }
    }

    /// Returns `true` while a cross-fade blend is in progress.
    pub fn is_blending(&self) -> bool {
        self.blend_state.is_some()
    }

    /// The name of the currently active state.
    pub fn current_state_name(&self) -> &str {
        &self.current_state
    }
}

// ---------------------------------------------------------------------------
// Convenience graph builders
// ---------------------------------------------------------------------------

/// Build the default idle → walk → run locomotion graph.
///
/// Parameters used:
/// * `"speed"` (f32) – character movement speed in m/s.
pub fn build_locomotion_graph() -> MotionGraph {
    let mut graph = MotionGraph::new();

    // States ----------------------------------------------------------------
    let idle = MotionState::new("idle", "anim_idle", 2.0)
        .with_loop(true)
        .with_morph("body_relaxed", 1.0)
        .with_morph("arms_down", 1.0);

    let walk = MotionState::new("walk", "anim_walk", 1.2)
        .with_loop(true)
        .with_morph("body_relaxed", 0.5)
        .with_morph("arms_swing", 0.8);

    let run = MotionState::new("run", "anim_run", 0.8)
        .with_loop(true)
        .with_morph("body_tense", 0.7)
        .with_morph("arms_swing", 1.0);

    let land = MotionState::new("land", "anim_land", 0.5)
        .with_loop(false)
        .with_morph("legs_bent", 1.0);

    graph.add_state(idle);
    graph.add_state(walk);
    graph.add_state(run);
    graph.add_state(land);

    // Transitions -----------------------------------------------------------
    // idle → walk when speed > 0.5 m/s
    graph.add_transition(MotionTransition {
        from_state: "idle".into(),
        to_state: "walk".into(),
        condition: TransitionCondition::ParameterAbove("speed".into(), 0.5),
        blend_duration: 0.3,
        priority: 0,
    });

    // walk → idle when speed < 0.3 m/s
    graph.add_transition(MotionTransition {
        from_state: "walk".into(),
        to_state: "idle".into(),
        condition: TransitionCondition::ParameterBelow("speed".into(), 0.3),
        blend_duration: 0.4,
        priority: 0,
    });

    // walk → run when speed > 3.0 m/s
    graph.add_transition(MotionTransition {
        from_state: "walk".into(),
        to_state: "run".into(),
        condition: TransitionCondition::ParameterAbove("speed".into(), 3.0),
        blend_duration: 0.3,
        priority: 1,
    });

    // run → walk when speed < 2.5 m/s
    graph.add_transition(MotionTransition {
        from_state: "run".into(),
        to_state: "walk".into(),
        condition: TransitionCondition::ParameterBelow("speed".into(), 2.5),
        blend_duration: 0.4,
        priority: 0,
    });

    // land → idle when the landing clip ends
    graph.add_transition(MotionTransition {
        from_state: "land".into(),
        to_state: "idle".into(),
        condition: TransitionCondition::AtEnd,
        blend_duration: 0.2,
        priority: 0,
    });

    graph
}

/// Build a facial-expression motion graph.
///
/// Parameters used:
/// * `"emotion"` (f32) – 0=neutral, 1=happy, 2=sad, 3=angry, 4=surprised.
pub fn build_expression_graph() -> MotionGraph {
    let mut graph = MotionGraph::new();

    let neutral = MotionState::new("neutral", "expr_neutral", 1.0)
        .with_loop(true)
        .with_morph("mouth_closed", 1.0)
        .with_morph("brow_neutral", 1.0);

    let happy = MotionState::new("happy", "expr_happy", 1.0)
        .with_loop(true)
        .with_morph("mouth_smile", 1.0)
        .with_morph("cheeks_raised", 0.7)
        .with_morph("brow_raised", 0.2);

    let sad = MotionState::new("sad", "expr_sad", 1.0)
        .with_loop(true)
        .with_morph("mouth_frown", 0.8)
        .with_morph("brow_sad", 1.0)
        .with_morph("eyes_half_closed", 0.5);

    let angry = MotionState::new("angry", "expr_angry", 1.0)
        .with_loop(true)
        .with_morph("brow_furrow", 1.0)
        .with_morph("mouth_tense", 0.6)
        .with_morph("nostrils_flare", 0.4);

    let surprised = MotionState::new("surprised", "expr_surprised", 1.0)
        .with_loop(true)
        .with_morph("mouth_open", 0.9)
        .with_morph("brow_raised", 1.0)
        .with_morph("eyes_wide", 1.0);

    graph.add_state(neutral);
    graph.add_state(happy);
    graph.add_state(sad);
    graph.add_state(angry);
    graph.add_state(surprised);

    // Transitions from neutral to each expression --------------------------
    for (target, lo, hi) in [
        ("happy", 0.5_f32, 1.5_f32),
        ("sad", 1.5, 2.5),
        ("angry", 2.5, 3.5),
        ("surprised", 3.5, 4.5),
    ] {
        let mid = (lo + hi) * 0.5;
        graph.add_transition(MotionTransition {
            from_state: "neutral".into(),
            to_state: target.into(),
            condition: TransitionCondition::ParameterEqual("emotion".into(), mid),
            blend_duration: 0.25,
            priority: 0,
        });
    }

    // Transitions back to neutral -------------------------------------------
    for from in ["happy", "sad", "angry", "surprised"] {
        graph.add_transition(MotionTransition {
            from_state: from.into(),
            to_state: "neutral".into(),
            condition: TransitionCondition::ParameterEqual("emotion".into(), 0.0),
            blend_duration: 0.35,
            priority: 0,
        });
    }

    graph
}

// ---------------------------------------------------------------------------
// Utility: blend_morph_maps
// ---------------------------------------------------------------------------

/// Linear blend between two morph-weight maps.
///
/// * `t = 0.0` → result equals `a`.
/// * `t = 1.0` → result equals `b`.
///
/// Keys present in only one map are treated as 0.0 in the other.
pub fn blend_morph_maps(
    a: &HashMap<String, f32>,
    b: &HashMap<String, f32>,
    t: f32,
) -> HashMap<String, f32> {
    let t = t.clamp(0.0, 1.0);
    let mut result: HashMap<String, f32> = HashMap::new();

    for (k, &va) in a {
        let vb = b.get(k).copied().unwrap_or(0.0);
        result.insert(k.clone(), va * (1.0 - t) + vb * t);
    }
    for (k, &vb) in b {
        if !result.contains_key(k) {
            result.insert(k.clone(), vb * t);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_tmp(name: &str, content: &str) {
        fs::write(format!("/tmp/{}", name), content).expect("should succeed");
    }

    // 1. build locomotion graph – state count
    #[test]
    fn test_locomotion_state_count() {
        let g = build_locomotion_graph();
        assert_eq!(g.state_count(), 4);
        write_tmp(
            "mg_locomotion_state_count.txt",
            &g.state_count().to_string(),
        );
    }

    // 2. build locomotion graph – transition count
    #[test]
    fn test_locomotion_transition_count() {
        let g = build_locomotion_graph();
        assert_eq!(g.transition_count(), 5);
        write_tmp(
            "mg_locomotion_transition_count.txt",
            &g.transition_count().to_string(),
        );
    }

    // 3. entry state defaults to first added state
    #[test]
    fn test_entry_state() {
        let g = build_locomotion_graph();
        assert_eq!(g.entry_state.as_deref(), Some("idle"));
        write_tmp("mg_entry_state.txt", "ok");
    }

    // 4. get_state returns correct clip name
    #[test]
    fn test_get_state_clip_name() {
        let g = build_locomotion_graph();
        let state = g.get_state("walk").expect("should succeed");
        assert_eq!(state.clip_name, "anim_walk");
        write_tmp("mg_get_state_clip.txt", &state.clip_name);
    }

    // 5. transitions_from sorted by descending priority
    #[test]
    fn test_transitions_from_priority_order() {
        let g = build_locomotion_graph();
        let ts = g.transitions_from("walk");
        assert!(ts.len() >= 2);
        // Highest priority first.
        assert!(ts[0].priority >= ts[ts.len() - 1].priority);
        write_tmp("mg_transition_priority.txt", "ok");
    }

    // 6. MotionController starts in entry state
    #[test]
    fn test_controller_entry_state() {
        let g = build_locomotion_graph();
        let ctrl = MotionController::new(g);
        assert_eq!(ctrl.current_state_name(), "idle");
        write_tmp("mg_ctrl_entry.txt", "ok");
    }

    // 7. set/get parameter round-trip
    #[test]
    fn test_parameter_round_trip() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        ctrl.set_parameter("speed", 1.5);
        assert!((ctrl.get_parameter("speed") - 1.5).abs() < 1e-5);
        write_tmp("mg_param_round_trip.txt", "ok");
    }

    // 8. missing parameter returns 0.0
    #[test]
    fn test_missing_parameter_default() {
        let g = build_locomotion_graph();
        let ctrl = MotionController::new(g);
        assert_eq!(ctrl.get_parameter("nonexistent"), 0.0);
        write_tmp("mg_missing_param.txt", "ok");
    }

    // 9. transition_to instantly (blend_duration = 0.0)
    #[test]
    fn test_instant_transition() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        ctrl.transition_to("run", 0.0);
        assert_eq!(ctrl.current_state_name(), "run");
        assert!(!ctrl.is_blending());
        write_tmp("mg_instant_transition.txt", "ok");
    }

    // 10. transition_to with blend starts a blend
    #[test]
    fn test_blend_transition() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        ctrl.transition_to("walk", 0.5);
        assert!(ctrl.is_blending());
        assert_eq!(ctrl.blend_state.as_deref(), Some("walk"));
        assert!((ctrl.blend_duration - 0.5).abs() < 1e-6);
        write_tmp("mg_blend_transition.txt", "ok");
    }

    // 11. blend finalises after enough updates
    #[test]
    fn test_blend_finalises() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        ctrl.transition_to("walk", 0.3);
        ctrl.update(0.1);
        assert!(ctrl.is_blending());
        ctrl.update(0.25); // total 0.35 > 0.3
        assert!(!ctrl.is_blending());
        assert_eq!(ctrl.current_state_name(), "walk");
        write_tmp("mg_blend_finalises.txt", "ok");
    }

    // 12. automatic idle → walk transition via parameter
    #[test]
    fn test_auto_idle_to_walk() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        ctrl.set_parameter("speed", 1.0); // > 0.5
        ctrl.update(0.01);
        // Should have started blending to walk.
        assert_eq!(ctrl.blend_state.as_deref(), Some("walk"));
        write_tmp("mg_auto_idle_walk.txt", "ok");
    }

    // 13. blend_morph_maps at t=0 returns a unchanged
    #[test]
    fn test_blend_morphs_t0() {
        let mut a = HashMap::new();
        a.insert("smile".to_string(), 0.8_f32);
        a.insert("brow".to_string(), 0.3_f32);
        let b: HashMap<String, f32> = HashMap::new();
        let result = blend_morph_maps(&a, &b, 0.0);
        assert!((result["smile"] - 0.8).abs() < 1e-6);
        assert!((result["brow"] - 0.3).abs() < 1e-6);
        write_tmp("mg_blend_t0.txt", "ok");
    }

    // 14. blend_morph_maps at t=1 returns b
    #[test]
    fn test_blend_morphs_t1() {
        let a: HashMap<String, f32> = HashMap::new();
        let mut b = HashMap::new();
        b.insert("frown".to_string(), 0.9_f32);
        let result = blend_morph_maps(&a, &b, 1.0);
        assert!((result["frown"] - 0.9).abs() < 1e-6);
        write_tmp("mg_blend_t1.txt", "ok");
    }

    // 15. blend_morph_maps at t=0.5 is midpoint
    #[test]
    fn test_blend_morphs_midpoint() {
        let mut a = HashMap::new();
        a.insert("key".to_string(), 0.0_f32);
        let mut b = HashMap::new();
        b.insert("key".to_string(), 1.0_f32);
        let result = blend_morph_maps(&a, &b, 0.5);
        assert!((result["key"] - 0.5).abs() < 1e-6);
        write_tmp("mg_blend_midpoint.txt", "ok");
    }

    // 16. evaluate_morphs while blending is interpolated
    #[test]
    fn test_evaluate_morphs_blending() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        // Force to walk state and start blending to run
        ctrl.transition_to("walk", 0.0);
        ctrl.transition_to("run", 1.0);
        ctrl.blend_time = 0.5; // halfway
        let morphs = ctrl.evaluate_morphs();
        // walk has arms_swing=0.8, run has arms_swing=1.0 → expect ~0.9
        let w = morphs.get("arms_swing").copied().unwrap_or(0.0);
        assert!((w - 0.9).abs() < 0.05, "arms_swing blend = {w}");
        write_tmp("mg_evaluate_morphs_blend.txt", &w.to_string());
    }

    // 17. check_condition: Always is always true
    #[test]
    fn test_condition_always() {
        let g = build_locomotion_graph();
        let ctrl = MotionController::new(g);
        assert!(ctrl.check_condition(&TransitionCondition::Always));
        write_tmp("mg_cond_always.txt", "ok");
    }

    // 18. check_condition: AfterSeconds fires only when state_time is enough
    #[test]
    fn test_condition_after_seconds() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        ctrl.state_time = 0.5;
        assert!(!ctrl.check_condition(&TransitionCondition::AfterSeconds(1.0)));
        ctrl.state_time = 1.5;
        assert!(ctrl.check_condition(&TransitionCondition::AfterSeconds(1.0)));
        write_tmp("mg_cond_after_seconds.txt", "ok");
    }

    // 19. check_condition: ParameterEqual uses ±0.05 tolerance
    #[test]
    fn test_condition_parameter_equal_tolerance() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        ctrl.set_parameter("x", 1.03);
        assert!(ctrl.check_condition(&TransitionCondition::ParameterEqual("x".into(), 1.0)));
        ctrl.set_parameter("x", 1.1);
        assert!(!ctrl.check_condition(&TransitionCondition::ParameterEqual("x".into(), 1.0)));
        write_tmp("mg_cond_param_equal.txt", "ok");
    }

    // 20. check_condition: AtEnd fires when state_time >= effective duration
    #[test]
    fn test_condition_at_end() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        ctrl.transition_to("land", 0.0);
        // land duration = 0.5, speed = 1.0
        ctrl.state_time = 0.4;
        assert!(!ctrl.check_condition(&TransitionCondition::AtEnd));
        ctrl.state_time = 0.6;
        assert!(ctrl.check_condition(&TransitionCondition::AtEnd));
        write_tmp("mg_cond_at_end.txt", "ok");
    }

    // 21. expression graph – neutral is entry state
    #[test]
    fn test_expression_graph_entry() {
        let g = build_expression_graph();
        assert_eq!(g.entry_state.as_deref(), Some("neutral"));
        write_tmp("mg_expr_entry.txt", "ok");
    }

    // 22. expression graph – morph weights present
    #[test]
    fn test_expression_graph_morph_weights() {
        let g = build_expression_graph();
        let happy = g.get_state("happy").expect("should succeed");
        assert!(happy.morph_weights.contains_key("mouth_smile"));
        write_tmp(
            "mg_expr_morph_weights.txt",
            &format!("{:?}", happy.morph_weights),
        );
    }

    // 23. blend_weight returns 0 when not blending
    #[test]
    fn test_blend_weight_no_blend() {
        let g = build_locomotion_graph();
        let ctrl = MotionController::new(g);
        assert_eq!(ctrl.blend_weight(), 0.0);
        write_tmp("mg_blend_weight_none.txt", "0.0");
    }

    // 24. default_graph alias returns same as build_locomotion_graph
    #[test]
    fn test_default_graph_alias() {
        let g1 = MotionGraph::default_graph();
        let g2 = build_locomotion_graph();
        assert_eq!(g1.state_count(), g2.state_count());
        assert_eq!(g1.transition_count(), g2.transition_count());
        write_tmp("mg_default_graph_alias.txt", "ok");
    }

    // 25. total_time accumulates across updates
    #[test]
    fn test_total_time_accumulates() {
        let g = build_locomotion_graph();
        let mut ctrl = MotionController::new(g);
        ctrl.update(0.1);
        ctrl.update(0.2);
        assert!((ctrl.total_time - 0.3).abs() < 1e-6);
        write_tmp("mg_total_time.txt", &ctrl.total_time.to_string());
    }
}
