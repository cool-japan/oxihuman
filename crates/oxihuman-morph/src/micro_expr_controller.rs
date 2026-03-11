// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Micro-expression timing controller: FACS-based brief involuntary expressions.

#![allow(dead_code)]

use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

/// FACS-aligned micro-expression categories.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MicroExprType {
    Fear,
    Disgust,
    Contempt,
    Surprise,
    Happiness,
    Sadness,
    Anger,
}

/// Configuration for the micro-expression controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MicroExpressionConfig {
    /// Maximum simultaneously active expressions.
    pub max_active: usize,
    /// Maximum queued (pending) expressions.
    pub max_queue: usize,
    /// Default duration in milliseconds when none specified.
    pub default_duration_ms: f32,
}

/// A single micro-expression instance managed by the controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MicroExpression {
    /// Unique id.
    pub id: u32,
    /// Type of expression.
    pub expr_type: MicroExprType,
    /// Total duration in ms.
    pub duration_ms: f32,
    /// Elapsed time in ms.
    pub elapsed_ms: f32,
    /// Peak intensity 0..1.
    pub intensity: f32,
    /// Whether this has been explicitly cancelled.
    pub cancelled: bool,
    /// Morph weights at peak intensity.
    pub morph_weights: HashMap<String, f32>,
}

/// Controller managing active + pending micro-expressions.
#[allow(dead_code)]
pub struct MicroExprController {
    pub config: MicroExpressionConfig,
    pub active: Vec<MicroExpression>,
    pub pending: Vec<MicroExpression>,
    next_id: u32,
}

/// Combined morph weight snapshot.
pub type MicroMorphWeights = HashMap<String, f32>;

// ── Default weights per type ──────────────────────────────────────────────────

fn default_weights_for(expr_type: &MicroExprType, intensity: f32) -> HashMap<String, f32> {
    let mut m = HashMap::new();
    let i = intensity;
    match expr_type {
        MicroExprType::Fear => {
            m.insert("brow_raise_inner".to_string(), 0.8 * i);
            m.insert("upper_lid_raise".to_string(), 0.9 * i);
            m.insert("lip_stretch".to_string(), 0.6 * i);
        }
        MicroExprType::Disgust => {
            m.insert("nose_wrinkle".to_string(), 0.9 * i);
            m.insert("upper_lip_raise".to_string(), 0.7 * i);
            m.insert("brow_lower".to_string(), 0.4 * i);
        }
        MicroExprType::Contempt => {
            m.insert("lip_corner_pull_r".to_string(), 0.8 * i);
            m.insert("cheek_raise_r".to_string(), 0.3 * i);
        }
        MicroExprType::Surprise => {
            m.insert("brow_raise".to_string(), 1.0 * i);
            m.insert("upper_lid_raise".to_string(), 0.8 * i);
            m.insert("jaw_drop".to_string(), 0.6 * i);
        }
        MicroExprType::Happiness => {
            m.insert("cheek_raise".to_string(), 0.7 * i);
            m.insert("lip_corner_pull".to_string(), 0.9 * i);
            m.insert("lower_lid_raise".to_string(), 0.5 * i);
        }
        MicroExprType::Sadness => {
            m.insert("brow_inner_raise".to_string(), 0.6 * i);
            m.insert("lip_corner_pull_down".to_string(), 0.7 * i);
            m.insert("lower_lip_depress".to_string(), 0.4 * i);
        }
        MicroExprType::Anger => {
            m.insert("brow_lower".to_string(), 0.9 * i);
            m.insert("nose_wrinkle".to_string(), 0.5 * i);
            m.insert("lip_tighten".to_string(), 0.8 * i);
        }
    }
    m
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Create a default config.
#[allow(dead_code)]
pub fn default_micro_config() -> MicroExpressionConfig {
    MicroExpressionConfig {
        max_active: 4,
        max_queue: 16,
        default_duration_ms: 150.0,
    }
}

/// Create a new controller.
#[allow(dead_code)]
pub fn new_micro_expression(config: MicroExpressionConfig) -> MicroExprController {
    MicroExprController {
        config,
        active: Vec::new(),
        pending: Vec::new(),
        next_id: 1,
    }
}

/// Trigger a micro-expression immediately with the given duration in ms.
/// Returns the assigned id, or `None` if the active limit is reached.
#[allow(dead_code)]
pub fn trigger_micro_expr(
    ctrl: &mut MicroExprController,
    expr_type: MicroExprType,
    duration_ms: f32,
    intensity: f32,
) -> Option<u32> {
    if ctrl.active.len() >= ctrl.config.max_active {
        return None;
    }
    let id = ctrl.next_id;
    ctrl.next_id += 1;
    let morph_weights = default_weights_for(&expr_type, intensity);
    ctrl.active.push(MicroExpression {
        id,
        expr_type,
        duration_ms,
        elapsed_ms: 0.0,
        intensity,
        cancelled: false,
        morph_weights,
    });
    Some(id)
}

/// Advance all active expressions by `dt_ms`. Finished ones are removed.
#[allow(dead_code)]
pub fn update_micro_expressions(ctrl: &mut MicroExprController, dt_ms: f32) {
    for expr in &mut ctrl.active {
        if !expr.cancelled {
            expr.elapsed_ms += dt_ms;
        }
    }
    ctrl.active.retain(|e| !e.cancelled && e.elapsed_ms < e.duration_ms);
    // Promote pending into active if there is room.
    while ctrl.active.len() < ctrl.config.max_active && !ctrl.pending.is_empty() {
        let next = ctrl.pending.remove(0);
        ctrl.active.push(next);
    }
}

/// Number of currently active (playing) expressions.
#[allow(dead_code)]
pub fn active_micro_expr_count(ctrl: &MicroExprController) -> usize {
    ctrl.active.len()
}

/// Combine all active expression morph weights (additive, clamped to 1.0).
#[allow(dead_code)]
pub fn micro_expr_weights(ctrl: &MicroExprController) -> MicroMorphWeights {
    let mut out: HashMap<String, f32> = HashMap::new();
    for expr in &ctrl.active {
        // Compute envelope: triangle (rise to peak, fall to zero)
        let t = if expr.duration_ms > 0.0 {
            expr.elapsed_ms / expr.duration_ms
        } else {
            1.0
        };
        let env = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
        for (k, v) in &expr.morph_weights {
            let entry = out.entry(k.clone()).or_insert(0.0);
            *entry = (*entry + v * env).min(1.0);
        }
    }
    out
}

/// Cancel an active expression by id. Returns true if found.
#[allow(dead_code)]
pub fn cancel_micro_expr(ctrl: &mut MicroExprController, id: u32) -> bool {
    for expr in &mut ctrl.active {
        if expr.id == id {
            expr.cancelled = true;
            return true;
        }
    }
    // Also remove from pending.
    let before = ctrl.pending.len();
    ctrl.pending.retain(|e| e.id != id);
    ctrl.pending.len() < before
}

/// Return the current envelope intensity of an active expression.
#[allow(dead_code)]
pub fn micro_expr_intensity(ctrl: &MicroExprController, id: u32) -> Option<f32> {
    ctrl.active.iter().find(|e| e.id == id).map(|e| {
        let t = if e.duration_ms > 0.0 {
            e.elapsed_ms / e.duration_ms
        } else {
            1.0
        };
        if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 }
    })
}

/// Queue an expression to be played as soon as a slot is free.
/// Returns the assigned id, or `None` if the queue is full.
#[allow(dead_code)]
pub fn queue_micro_expr(
    ctrl: &mut MicroExprController,
    expr_type: MicroExprType,
    duration_ms: f32,
    intensity: f32,
) -> Option<u32> {
    if ctrl.pending.len() >= ctrl.config.max_queue {
        return None;
    }
    let id = ctrl.next_id;
    ctrl.next_id += 1;
    let morph_weights = default_weights_for(&expr_type, intensity);
    ctrl.pending.push(MicroExpression {
        id,
        expr_type,
        duration_ms,
        elapsed_ms: 0.0,
        intensity,
        cancelled: false,
        morph_weights,
    });
    Some(id)
}

/// Number of queued (pending) expressions.
#[allow(dead_code)]
pub fn pending_count(ctrl: &MicroExprController) -> usize {
    ctrl.pending.len()
}

/// Serialize a single expression to a simple JSON string.
#[allow(dead_code)]
pub fn micro_expr_to_json(expr: &MicroExpression) -> String {
    let etype = expression_type_name(&expr.expr_type);
    format!(
        r#"{{"id":{},"type":"{}","duration_ms":{:.1},"elapsed_ms":{:.1},"intensity":{:.3},"cancelled":{}}}"#,
        expr.id, etype, expr.duration_ms, expr.elapsed_ms, expr.intensity, expr.cancelled
    )
}

/// Human-readable name for a `MicroExprType`.
#[allow(dead_code)]
pub fn expression_type_name(expr_type: &MicroExprType) -> &'static str {
    match expr_type {
        MicroExprType::Fear => "fear",
        MicroExprType::Disgust => "disgust",
        MicroExprType::Contempt => "contempt",
        MicroExprType::Surprise => "surprise",
        MicroExprType::Happiness => "happiness",
        MicroExprType::Sadness => "sadness",
        MicroExprType::Anger => "anger",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctrl() -> MicroExprController {
        new_micro_expression(default_micro_config())
    }

    #[test]
    fn test_default_config() {
        let cfg = default_micro_config();
        assert!(cfg.max_active > 0);
        assert!(cfg.max_queue > 0);
        assert!(cfg.default_duration_ms > 0.0);
    }

    #[test]
    fn test_new_controller_empty() {
        let ctrl = make_ctrl();
        assert_eq!(active_micro_expr_count(&ctrl), 0);
        assert_eq!(pending_count(&ctrl), 0);
    }

    #[test]
    fn test_trigger_returns_id() {
        let mut ctrl = make_ctrl();
        let id = trigger_micro_expr(&mut ctrl, MicroExprType::Fear, 150.0, 0.8);
        assert!(id.is_some());
        assert_eq!(active_micro_expr_count(&ctrl), 1);
    }

    #[test]
    fn test_trigger_fills_active() {
        let mut ctrl = make_ctrl();
        for _ in 0..ctrl.config.max_active {
            trigger_micro_expr(&mut ctrl, MicroExprType::Anger, 100.0, 1.0);
        }
        // Should be full now — one extra attempt should return None.
        let over = trigger_micro_expr(&mut ctrl, MicroExprType::Happiness, 100.0, 1.0);
        assert!(over.is_none());
    }

    #[test]
    fn test_update_advances_time() {
        let mut ctrl = make_ctrl();
        trigger_micro_expr(&mut ctrl, MicroExprType::Surprise, 200.0, 1.0);
        update_micro_expressions(&mut ctrl, 50.0);
        let expr = &ctrl.active[0];
        assert!((expr.elapsed_ms - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_update_removes_finished() {
        let mut ctrl = make_ctrl();
        trigger_micro_expr(&mut ctrl, MicroExprType::Happiness, 100.0, 1.0);
        update_micro_expressions(&mut ctrl, 200.0);
        assert_eq!(active_micro_expr_count(&ctrl), 0);
    }

    #[test]
    fn test_micro_expr_weights_nonempty() {
        let mut ctrl = make_ctrl();
        trigger_micro_expr(&mut ctrl, MicroExprType::Fear, 200.0, 1.0);
        update_micro_expressions(&mut ctrl, 50.0);
        let w = micro_expr_weights(&ctrl);
        assert!(!w.is_empty());
    }

    #[test]
    fn test_cancel_micro_expr() {
        let mut ctrl = make_ctrl();
        let id = trigger_micro_expr(&mut ctrl, MicroExprType::Disgust, 300.0, 0.5).unwrap();
        assert!(cancel_micro_expr(&mut ctrl, id));
        update_micro_expressions(&mut ctrl, 1.0);
        assert_eq!(active_micro_expr_count(&ctrl), 0);
    }

    #[test]
    fn test_cancel_nonexistent() {
        let mut ctrl = make_ctrl();
        assert!(!cancel_micro_expr(&mut ctrl, 9999));
    }

    #[test]
    fn test_micro_expr_intensity_some() {
        let mut ctrl = make_ctrl();
        let id = trigger_micro_expr(&mut ctrl, MicroExprType::Contempt, 200.0, 1.0).unwrap();
        update_micro_expressions(&mut ctrl, 50.0);
        assert!(micro_expr_intensity(&ctrl, id).is_some());
    }

    #[test]
    fn test_micro_expr_intensity_none() {
        let ctrl = make_ctrl();
        assert!(micro_expr_intensity(&ctrl, 42).is_none());
    }

    #[test]
    fn test_queue_and_promote() {
        let mut ctrl = make_ctrl();
        // Fill active.
        for _ in 0..ctrl.config.max_active {
            trigger_micro_expr(&mut ctrl, MicroExprType::Anger, 100.0, 1.0);
        }
        // Queue one.
        let id = queue_micro_expr(&mut ctrl, MicroExprType::Sadness, 100.0, 0.5);
        assert!(id.is_some());
        assert_eq!(pending_count(&ctrl), 1);
        // Expire all active.
        update_micro_expressions(&mut ctrl, 200.0);
        // Pending should have been promoted.
        assert_eq!(active_micro_expr_count(&ctrl), 1);
        assert_eq!(pending_count(&ctrl), 0);
    }

    #[test]
    fn test_pending_count() {
        let mut ctrl = make_ctrl();
        queue_micro_expr(&mut ctrl, MicroExprType::Fear, 100.0, 1.0);
        queue_micro_expr(&mut ctrl, MicroExprType::Anger, 100.0, 0.5);
        assert_eq!(pending_count(&ctrl), 2);
    }

    #[test]
    fn test_micro_expr_to_json() {
        let mut ctrl = make_ctrl();
        let id = trigger_micro_expr(&mut ctrl, MicroExprType::Happiness, 100.0, 0.9).unwrap();
        let expr = ctrl.active.iter().find(|e| e.id == id).unwrap();
        let json = micro_expr_to_json(expr);
        assert!(json.contains("happiness"));
    }

    #[test]
    fn test_expression_type_name_all() {
        assert_eq!(expression_type_name(&MicroExprType::Fear), "fear");
        assert_eq!(expression_type_name(&MicroExprType::Disgust), "disgust");
        assert_eq!(expression_type_name(&MicroExprType::Contempt), "contempt");
        assert_eq!(expression_type_name(&MicroExprType::Surprise), "surprise");
        assert_eq!(expression_type_name(&MicroExprType::Happiness), "happiness");
        assert_eq!(expression_type_name(&MicroExprType::Sadness), "sadness");
        assert_eq!(expression_type_name(&MicroExprType::Anger), "anger");
    }

    #[test]
    fn test_weights_clamped_to_one() {
        let mut ctrl = make_ctrl();
        // Add max active with Fear (many overlapping weights).
        for _ in 0..ctrl.config.max_active {
            trigger_micro_expr(&mut ctrl, MicroExprType::Fear, 200.0, 1.0);
        }
        update_micro_expressions(&mut ctrl, 50.0);
        let w = micro_expr_weights(&ctrl);
        for v in w.values() {
            assert!(*v <= 1.0 + 1e-6);
        }
    }

    #[test]
    fn test_queue_overflow_returns_none() {
        let cfg = MicroExpressionConfig {
            max_active: 1,
            max_queue: 2,
            default_duration_ms: 100.0,
        };
        let mut ctrl = new_micro_expression(cfg);
        queue_micro_expr(&mut ctrl, MicroExprType::Anger, 100.0, 1.0);
        queue_micro_expr(&mut ctrl, MicroExprType::Fear, 100.0, 1.0);
        // queue is full
        let r = queue_micro_expr(&mut ctrl, MicroExprType::Sadness, 100.0, 1.0);
        assert!(r.is_none());
    }
}
