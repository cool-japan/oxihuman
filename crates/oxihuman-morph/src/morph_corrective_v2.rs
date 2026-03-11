// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Corrective shape driven by angle between two joints.

#[allow(dead_code)]
pub struct CorrectiveV2 {
    pub joint_angle: f32,
    pub trigger_angle: f32,
    pub range: f32,
    pub shape_weight: f32,
}

#[allow(dead_code)]
pub fn new_corrective_v2(trigger_angle: f32, range: f32) -> CorrectiveV2 {
    CorrectiveV2 { joint_angle: 0.0, trigger_angle, range, shape_weight: 0.0 }
}

#[allow(dead_code)]
pub fn cv2_update(c: &mut CorrectiveV2, joint_angle: f32) {
    c.joint_angle = joint_angle;
    let diff = (c.trigger_angle - joint_angle).abs();
    if c.range > 1e-7 {
        c.shape_weight = (1.0 - diff / c.range).clamp(0.0, 1.0);
    } else {
        c.shape_weight = if diff < 1e-7 { 1.0 } else { 0.0 };
    }
}

#[allow(dead_code)]
pub fn cv2_weight(c: &CorrectiveV2) -> f32 {
    c.shape_weight
}

#[allow(dead_code)]
pub fn cv2_is_active(c: &CorrectiveV2) -> bool {
    c.shape_weight > 0.01
}

#[allow(dead_code)]
pub fn cv2_trigger_angle(c: &CorrectiveV2) -> f32 {
    c.trigger_angle
}

#[allow(dead_code)]
pub fn cv2_blend_weight(c: &CorrectiveV2, blend_factor: f32) -> f32 {
    c.shape_weight * blend_factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_update_at_trigger() {
        let mut c = new_corrective_v2(PI * 0.5, PI * 0.5);
        cv2_update(&mut c, PI * 0.5);
        assert!((cv2_weight(&c) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_update_outside_range() {
        let mut c = new_corrective_v2(1.0, 0.5);
        cv2_update(&mut c, 2.0);
        assert_eq!(cv2_weight(&c), 0.0);
    }

    #[test]
    fn test_is_active_true() {
        let mut c = new_corrective_v2(1.0, 1.0);
        cv2_update(&mut c, 1.0);
        assert!(cv2_is_active(&c));
    }

    #[test]
    fn test_is_active_false() {
        let mut c = new_corrective_v2(1.0, 0.1);
        cv2_update(&mut c, 5.0);
        assert!(!cv2_is_active(&c));
    }

    #[test]
    fn test_weight_range() {
        let mut c = new_corrective_v2(1.0, 1.0);
        cv2_update(&mut c, 0.5);
        let w = cv2_weight(&c);
        assert!((0.0..=1.0).contains(&w));
    }

    #[test]
    fn test_trigger_angle() {
        let c = new_corrective_v2(1.57, 0.5);
        assert!((cv2_trigger_angle(&c) - 1.57).abs() < 1e-5);
    }

    #[test]
    fn test_blend_weight() {
        let mut c = new_corrective_v2(1.0, 1.0);
        cv2_update(&mut c, 1.0);
        let bw = cv2_blend_weight(&c, 0.5);
        assert!((bw - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_partial_activation() {
        let mut c = new_corrective_v2(1.0, 1.0);
        cv2_update(&mut c, 0.5);
        let w = cv2_weight(&c);
        assert!((w - 0.5).abs() < 1e-5);
    }
}
