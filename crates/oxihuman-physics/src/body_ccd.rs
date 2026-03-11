#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Continuous collision detection state for a body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyCcd {
    enabled: bool,
    threshold: f32,
    prev_pos: [f32; 3],
    curr_pos: [f32; 3],
    radius: f32,
}

#[allow(dead_code)]
pub fn new_body_ccd(radius: f32) -> BodyCcd {
    BodyCcd {
        enabled: true,
        threshold: 0.01,
        prev_pos: [0.0; 3],
        curr_pos: [0.0; 3],
        radius,
    }
}

#[allow(dead_code)]
pub fn ccd_toi(ccd: &BodyCcd, target: [f32; 3], target_radius: f32) -> Option<f32> {
    let dx = target[0] - ccd.curr_pos[0];
    let dy = target[1] - ccd.curr_pos[1];
    let dz = target[2] - ccd.curr_pos[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let combined = ccd.radius + target_radius;
    if dist < combined {
        Some(dist / combined)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn ccd_sweep_test(ccd: &BodyCcd) -> f32 {
    let dx = ccd.curr_pos[0] - ccd.prev_pos[0];
    let dy = ccd.curr_pos[1] - ccd.prev_pos[1];
    let dz = ccd.curr_pos[2] - ccd.prev_pos[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn ccd_is_enabled(ccd: &BodyCcd) -> bool {
    ccd.enabled
}

#[allow(dead_code)]
pub fn ccd_set_enabled(ccd: &mut BodyCcd, enabled: bool) {
    ccd.enabled = enabled;
}

#[allow(dead_code)]
pub fn ccd_threshold(ccd: &BodyCcd) -> f32 {
    ccd.threshold
}

#[allow(dead_code)]
pub fn ccd_to_json(ccd: &BodyCcd) -> String {
    format!(
        "{{\"enabled\":{},\"threshold\":{:.6},\"radius\":{:.6}}}",
        ccd.enabled, ccd.threshold, ccd.radius
    )
}

#[allow(dead_code)]
pub fn ccd_reset(ccd: &mut BodyCcd) {
    ccd.prev_pos = [0.0; 3];
    ccd.curr_pos = [0.0; 3];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_body_ccd() {
        let ccd = new_body_ccd(1.0);
        assert!(ccd_is_enabled(&ccd));
    }

    #[test]
    fn test_ccd_toi_hit() {
        let mut ccd = new_body_ccd(1.0);
        ccd.curr_pos = [0.0; 3];
        let toi = ccd_toi(&ccd, [0.5, 0.0, 0.0], 1.0);
        assert!(toi.is_some());
    }

    #[test]
    fn test_ccd_toi_miss() {
        let ccd = new_body_ccd(0.1);
        let toi = ccd_toi(&ccd, [100.0, 0.0, 0.0], 0.1);
        assert!(toi.is_none());
    }

    #[test]
    fn test_ccd_sweep_test() {
        let mut ccd = new_body_ccd(1.0);
        ccd.prev_pos = [0.0; 3];
        ccd.curr_pos = [1.0, 0.0, 0.0];
        assert!((ccd_sweep_test(&ccd) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ccd_is_enabled() {
        let ccd = new_body_ccd(1.0);
        assert!(ccd_is_enabled(&ccd));
    }

    #[test]
    fn test_ccd_set_enabled() {
        let mut ccd = new_body_ccd(1.0);
        ccd_set_enabled(&mut ccd, false);
        assert!(!ccd_is_enabled(&ccd));
    }

    #[test]
    fn test_ccd_threshold() {
        let ccd = new_body_ccd(1.0);
        assert!((ccd_threshold(&ccd) - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_ccd_to_json() {
        let ccd = new_body_ccd(1.0);
        let json = ccd_to_json(&ccd);
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_ccd_reset() {
        let mut ccd = new_body_ccd(1.0);
        ccd.curr_pos = [5.0; 3];
        ccd_reset(&mut ccd);
        assert!((ccd.curr_pos[0]).abs() < 1e-6);
    }

    #[test]
    fn test_ccd_sweep_zero() {
        let ccd = new_body_ccd(1.0);
        assert!(ccd_sweep_test(&ccd).abs() < 1e-6);
    }
}
