// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Body-segment proportioning control (torso, limbs, head scale).

use std::f32::consts::PI;

/// Which body segment to address.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodySegment {
    Head,
    Torso,
    UpperArm,
    LowerArm,
    UpperLeg,
    LowerLeg,
}

/// Per-segment scale override.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SegmentScale {
    pub segment: BodySegment,
    /// Relative scale factor (1.0 = neutral).
    pub scale: f32,
    /// Blend weight toward this override (0..1).
    pub weight: f32,
}

/// Collection of all segment overrides.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct BodySegmentState {
    pub overrides: Vec<SegmentScale>,
}

/// Configuration bounds for segment scaling.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BodySegmentConfig {
    pub min_scale: f32,
    pub max_scale: f32,
}

impl Default for BodySegmentConfig {
    fn default() -> Self {
        Self {
            min_scale: 0.5,
            max_scale: 2.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_body_segment_state() -> BodySegmentState {
    BodySegmentState::default()
}

#[allow(dead_code)]
pub fn default_body_segment_config() -> BodySegmentConfig {
    BodySegmentConfig::default()
}

#[allow(dead_code)]
pub fn set_segment_scale(
    state: &mut BodySegmentState,
    cfg: &BodySegmentConfig,
    segment: BodySegment,
    scale: f32,
    weight: f32,
) {
    let scale = scale.clamp(cfg.min_scale, cfg.max_scale);
    let weight = weight.clamp(0.0, 1.0);
    if let Some(entry) = state.overrides.iter_mut().find(|e| e.segment == segment) {
        entry.scale = scale;
        entry.weight = weight;
    } else {
        state.overrides.push(SegmentScale {
            segment,
            scale,
            weight,
        });
    }
}

#[allow(dead_code)]
pub fn get_segment_scale(state: &BodySegmentState, segment: BodySegment) -> f32 {
    state
        .overrides
        .iter()
        .find(|e| e.segment == segment)
        .map(|e| e.scale)
        .unwrap_or(1.0)
}

#[allow(dead_code)]
pub fn reset_segment(state: &mut BodySegmentState, segment: BodySegment) {
    state.overrides.retain(|e| e.segment != segment);
}

#[allow(dead_code)]
pub fn reset_all_segments(state: &mut BodySegmentState) {
    state.overrides.clear();
}

#[allow(dead_code)]
pub fn blend_segment_states(
    a: &BodySegmentState,
    b: &BodySegmentState,
    t: f32,
) -> BodySegmentState {
    let t = t.clamp(0.0, 1.0);
    let all_segs = [
        BodySegment::Head,
        BodySegment::Torso,
        BodySegment::UpperArm,
        BodySegment::LowerArm,
        BodySegment::UpperLeg,
        BodySegment::LowerLeg,
    ];
    let overrides = all_segs
        .iter()
        .map(|&seg| {
            let sa = a
                .overrides
                .iter()
                .find(|e| e.segment == seg)
                .map(|e| e.scale)
                .unwrap_or(1.0);
            let sb = b
                .overrides
                .iter()
                .find(|e| e.segment == seg)
                .map(|e| e.scale)
                .unwrap_or(1.0);
            let wa = a
                .overrides
                .iter()
                .find(|e| e.segment == seg)
                .map(|e| e.weight)
                .unwrap_or(0.0);
            let wb = b
                .overrides
                .iter()
                .find(|e| e.segment == seg)
                .map(|e| e.weight)
                .unwrap_or(0.0);
            SegmentScale {
                segment: seg,
                scale: sa + (sb - sa) * t,
                weight: wa + (wb - wa) * t,
            }
        })
        .collect();
    BodySegmentState { overrides }
}

#[allow(dead_code)]
pub fn segment_name(seg: BodySegment) -> &'static str {
    match seg {
        BodySegment::Head => "head",
        BodySegment::Torso => "torso",
        BodySegment::UpperArm => "upper_arm",
        BodySegment::LowerArm => "lower_arm",
        BodySegment::UpperLeg => "upper_leg",
        BodySegment::LowerLeg => "lower_leg",
    }
}

#[allow(dead_code)]
pub fn total_limb_scale(state: &BodySegmentState) -> f32 {
    let ua = get_segment_scale(state, BodySegment::UpperArm);
    let la = get_segment_scale(state, BodySegment::LowerArm);
    let ul = get_segment_scale(state, BodySegment::UpperLeg);
    let ll = get_segment_scale(state, BodySegment::LowerLeg);
    (ua + la + ul + ll) / 4.0
}

/// Sinusoidal rhythm factor useful for breath-driven scale animation.
#[allow(dead_code)]
pub fn rhythm_scale(base: f32, amplitude: f32, phase_rad: f32) -> f32 {
    base + amplitude * phase_rad.sin()
}

/// Estimate limb length given scale, using a reference length in metres.
#[allow(dead_code)]
pub fn limb_length_m(reference_m: f32, scale: f32) -> f32 {
    reference_m * scale
}

/// Angular contribution of a segment scale (heuristic, uses PI internally).
#[allow(dead_code)]
pub fn segment_angle_contribution(scale: f32) -> f32 {
    (scale - 1.0) * PI * 0.1
}

#[allow(dead_code)]
pub fn state_to_json(state: &BodySegmentState) -> String {
    let entries: Vec<String> = state
        .overrides
        .iter()
        .map(|e| {
            format!(
                "{{\"segment\":\"{}\",\"scale\":{:.4},\"weight\":{:.4}}}",
                segment_name(e.segment),
                e.scale,
                e.weight
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_empty() {
        let s = new_body_segment_state();
        assert!(s.overrides.is_empty());
    }

    #[test]
    fn set_and_get_scale() {
        let mut s = new_body_segment_state();
        let cfg = default_body_segment_config();
        set_segment_scale(&mut s, &cfg, BodySegment::Torso, 1.2, 1.0);
        assert!((get_segment_scale(&s, BodySegment::Torso) - 1.2).abs() < 1e-5);
    }

    #[test]
    fn unknown_segment_returns_neutral() {
        let s = new_body_segment_state();
        assert!((get_segment_scale(&s, BodySegment::Head) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn clamps_min_scale() {
        let mut s = new_body_segment_state();
        let cfg = default_body_segment_config();
        set_segment_scale(&mut s, &cfg, BodySegment::Head, 0.1, 1.0);
        assert!(get_segment_scale(&s, BodySegment::Head) >= cfg.min_scale);
    }

    #[test]
    fn clamps_max_scale() {
        let mut s = new_body_segment_state();
        let cfg = default_body_segment_config();
        set_segment_scale(&mut s, &cfg, BodySegment::LowerLeg, 5.0, 1.0);
        assert!(get_segment_scale(&s, BodySegment::LowerLeg) <= cfg.max_scale);
    }

    #[test]
    fn reset_segment_removes_entry() {
        let mut s = new_body_segment_state();
        let cfg = default_body_segment_config();
        set_segment_scale(&mut s, &cfg, BodySegment::UpperArm, 1.5, 1.0);
        reset_segment(&mut s, BodySegment::UpperArm);
        assert!(s.overrides.is_empty());
    }

    #[test]
    fn blend_midpoint() {
        let mut a = new_body_segment_state();
        let mut b = new_body_segment_state();
        let cfg = default_body_segment_config();
        set_segment_scale(&mut a, &cfg, BodySegment::Torso, 1.0, 1.0);
        set_segment_scale(&mut b, &cfg, BodySegment::Torso, 2.0, 1.0);
        let mid = blend_segment_states(&a, &b, 0.5);
        let s = get_segment_scale(&mid, BodySegment::Torso);
        assert!((s - 1.5).abs() < 1e-4);
    }

    #[test]
    fn segment_name_all_variants() {
        assert_eq!(segment_name(BodySegment::Head), "head");
        assert_eq!(segment_name(BodySegment::LowerLeg), "lower_leg");
    }

    #[test]
    fn rhythm_scale_at_zero_phase() {
        let v = rhythm_scale(1.0, 0.1, 0.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn json_contains_torso() {
        let mut s = new_body_segment_state();
        let cfg = default_body_segment_config();
        set_segment_scale(&mut s, &cfg, BodySegment::Torso, 1.1, 0.8);
        let j = state_to_json(&s);
        assert!(j.contains("torso"));
    }
}
