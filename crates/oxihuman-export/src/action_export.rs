// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Action (animation action) export utilities.

/* ── legacy API (keep for existing lib.rs exports) ── */

#[derive(Debug, Clone)]
pub struct ActionKeyframe {
    pub time: f32,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct ActionExport {
    pub name: String,
    pub keyframes: Vec<ActionKeyframe>,
}

pub fn new_action_export(name: &str) -> ActionExport {
    ActionExport {
        name: name.to_string(),
        keyframes: vec![],
    }
}

pub fn add_keyframe(action: &mut ActionExport, time: f32, value: f32) {
    action.keyframes.push(ActionKeyframe { time, value });
}

pub fn keyframe_count(action: &ActionExport) -> usize {
    action.keyframes.len()
}

pub fn action_duration(action: &ActionExport) -> f32 {
    if action.keyframes.is_empty() {
        return 0.0;
    }
    let min_t = action
        .keyframes
        .iter()
        .map(|k| k.time)
        .fold(f32::MAX, f32::min);
    let max_t = action
        .keyframes
        .iter()
        .map(|k| k.time)
        .fold(f32::MIN, f32::max);
    max_t - min_t
}

pub fn sample_action(action: &ActionExport, time: f32) -> f32 {
    if action.keyframes.is_empty() {
        return 0.0;
    }
    if action.keyframes.len() == 1 {
        return action.keyframes[0].value;
    }
    let mut sorted: Vec<&ActionKeyframe> = action.keyframes.iter().collect();
    sorted.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if time <= sorted[0].time {
        return sorted[0].value;
    }
    if time >= sorted[sorted.len() - 1].time {
        return sorted[sorted.len() - 1].value;
    }
    for i in 0..sorted.len() - 1 {
        if time >= sorted[i].time && time <= sorted[i + 1].time {
            let dt = sorted[i + 1].time - sorted[i].time;
            if dt < 1e-12 {
                return sorted[i].value;
            }
            let t = (time - sorted[i].time) / dt;
            return sorted[i].value + (sorted[i + 1].value - sorted[i].value) * t;
        }
    }
    0.0
}

pub fn validate_action(action: &ActionExport) -> bool {
    action.keyframes.iter().all(|k| k.time >= 0.0)
}

pub fn action_to_json(action: &ActionExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"keyframes\":{},\"duration\":{:.6}}}",
        action.name,
        keyframe_count(action),
        action_duration(action)
    )
}

pub fn clear_keyframes(action: &mut ActionExport) {
    action.keyframes.clear();
}

/* ── spec functions (wave 150B) ── */

/// Spec-style action export (multi-fcurve).
#[derive(Debug, Clone)]
pub struct ActionExportSpec {
    pub name: String,
    pub fcurves: Vec<String>,
    pub fps: f32,
}

/// Create a new `ActionExportSpec`.
pub fn new_action_export_spec(name: &str, fps: f32) -> ActionExportSpec {
    ActionExportSpec {
        name: name.to_string(),
        fcurves: Vec::new(),
        fps,
    }
}

/// Push an fcurve data path string.
pub fn action_push_fcurve(a: &mut ActionExportSpec, path: &str) {
    a.fcurves.push(path.to_string());
}

/// Duration in frames (stub: returns 0).
pub fn action_duration_frames(a: &ActionExportSpec) -> usize {
    let _ = a;
    0
}

/// Serialize to JSON.
pub fn action_spec_to_json(a: &ActionExportSpec) -> String {
    format!(
        "{{\"name\":\"{}\",\"fps\":{},\"fcurves\":{}}}",
        a.name,
        a.fps,
        a.fcurves.len()
    )
}

/// Number of fcurves.
pub fn action_fcurve_count(a: &ActionExportSpec) -> usize {
    a.fcurves.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_action_export() {
        let a = new_action_export("walk");
        assert_eq!(a.name, "walk");
        assert_eq!(keyframe_count(&a), 0);
    }

    #[test]
    fn test_add_keyframe() {
        let mut a = new_action_export("test");
        add_keyframe(&mut a, 0.0, 1.0);
        assert_eq!(keyframe_count(&a), 1);
    }

    #[test]
    fn test_duration() {
        let mut a = new_action_export("test");
        add_keyframe(&mut a, 1.0, 0.0);
        add_keyframe(&mut a, 3.0, 1.0);
        assert!((action_duration(&a) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_action_push_fcurve() {
        let mut a = new_action_export_spec("run", 24.0);
        action_push_fcurve(&mut a, "loc.x");
        assert_eq!(action_fcurve_count(&a), 1);
    }

    #[test]
    fn test_action_spec_to_json() {
        let a = new_action_export_spec("idle", 30.0);
        let j = action_spec_to_json(&a);
        assert!(j.contains("idle"));
    }
}
