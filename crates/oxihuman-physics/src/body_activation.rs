#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// Body activation/deactivation state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyActivation {
    active: bool,
    sleep_time: f32,
    threshold: f32,
}

#[allow(dead_code)]
pub fn new_body_activation(threshold: f32) -> BodyActivation {
    BodyActivation {
        active: true,
        sleep_time: 0.0,
        threshold,
    }
}

#[allow(dead_code)]
pub fn activate_body(ba: &mut BodyActivation) {
    ba.active = true;
    ba.sleep_time = 0.0;
}

#[allow(dead_code)]
pub fn deactivate_body(ba: &mut BodyActivation) {
    ba.active = false;
}

#[allow(dead_code)]
pub fn is_active_ba(ba: &BodyActivation) -> bool {
    ba.active
}

#[allow(dead_code)]
pub fn activation_time(ba: &BodyActivation) -> f32 {
    ba.sleep_time
}

#[allow(dead_code)]
pub fn activation_threshold(ba: &BodyActivation) -> f32 {
    ba.threshold
}

#[allow(dead_code)]
pub fn activation_to_json(ba: &BodyActivation) -> String {
    format!(
        "{{\"active\":{},\"sleep_time\":{:.6},\"threshold\":{:.6}}}",
        ba.active, ba.sleep_time, ba.threshold
    )
}

#[allow(dead_code)]
pub fn activation_reset(ba: &mut BodyActivation) {
    ba.active = true;
    ba.sleep_time = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_body_activation() {
        let ba = new_body_activation(0.5);
        assert!(is_active_ba(&ba));
    }

    #[test]
    fn test_activate_body() {
        let mut ba = new_body_activation(0.5);
        deactivate_body(&mut ba);
        activate_body(&mut ba);
        assert!(is_active_ba(&ba));
    }

    #[test]
    fn test_deactivate_body() {
        let mut ba = new_body_activation(0.5);
        deactivate_body(&mut ba);
        assert!(!is_active_ba(&ba));
    }

    #[test]
    fn test_is_active_ba() {
        let ba = new_body_activation(0.5);
        assert!(is_active_ba(&ba));
    }

    #[test]
    fn test_activation_time() {
        let ba = new_body_activation(0.5);
        assert!(activation_time(&ba).abs() < 1e-6);
    }

    #[test]
    fn test_activation_threshold() {
        let ba = new_body_activation(0.5);
        assert!((activation_threshold(&ba) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_activation_to_json() {
        let ba = new_body_activation(0.5);
        let json = activation_to_json(&ba);
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn test_activation_reset() {
        let mut ba = new_body_activation(0.5);
        deactivate_body(&mut ba);
        activation_reset(&mut ba);
        assert!(is_active_ba(&ba));
    }

    #[test]
    fn test_sleep_time_reset_on_activate() {
        let mut ba = new_body_activation(0.5);
        ba.sleep_time = 5.0;
        activate_body(&mut ba);
        assert!(activation_time(&ba).abs() < 1e-6);
    }

    #[test]
    fn test_threshold_preserved() {
        let mut ba = new_body_activation(1.5);
        activation_reset(&mut ba);
        assert!((activation_threshold(&ba) - 1.5).abs() < 1e-6);
    }
}
