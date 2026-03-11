// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Screen flash effect for impact/transition feedback.

/// Flash effect state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlashEffect {
    pub color: [f32; 3],
    pub intensity: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub active: bool,
}

#[allow(dead_code)]
pub fn new_flash_effect() -> FlashEffect {
    FlashEffect {
        color: [1.0, 1.0, 1.0],
        intensity: 1.0,
        duration: 0.3,
        elapsed: 0.0,
        active: false,
    }
}

#[allow(dead_code)]
pub fn trigger_flash(effect: &mut FlashEffect, r: f32, g: f32, b: f32, intensity: f32, duration: f32) {
    effect.color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
    effect.intensity = intensity.clamp(0.0, 5.0);
    effect.duration = duration.max(0.01);
    effect.elapsed = 0.0;
    effect.active = true;
}

#[allow(dead_code)]
pub fn update_flash(effect: &mut FlashEffect, dt: f32) {
    if !effect.active { return; }
    effect.elapsed += dt;
    if effect.elapsed >= effect.duration {
        effect.active = false;
        effect.elapsed = effect.duration;
    }
}

#[allow(dead_code)]
pub fn flash_alpha(effect: &FlashEffect) -> f32 {
    if !effect.active { return 0.0; }
    let t = (effect.elapsed / effect.duration).clamp(0.0, 1.0);
    effect.intensity * (1.0 - t)
}

#[allow(dead_code)]
pub fn is_flash_active(effect: &FlashEffect) -> bool {
    effect.active
}

#[allow(dead_code)]
pub fn reset_flash(effect: &mut FlashEffect) {
    effect.active = false;
    effect.elapsed = 0.0;
}

#[allow(dead_code)]
pub fn flash_progress(effect: &FlashEffect) -> f32 {
    if effect.duration <= 0.0 { return 1.0; }
    (effect.elapsed / effect.duration).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn flash_remaining(effect: &FlashEffect) -> f32 {
    (effect.duration - effect.elapsed).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flash() {
        let f = new_flash_effect();
        assert!(!f.active);
    }

    #[test]
    fn test_trigger_flash() {
        let mut f = new_flash_effect();
        trigger_flash(&mut f, 1.0, 0.0, 0.0, 2.0, 0.5);
        assert!(f.active);
        assert!((f.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_update_flash() {
        let mut f = new_flash_effect();
        trigger_flash(&mut f, 1.0, 1.0, 1.0, 1.0, 0.3);
        update_flash(&mut f, 0.1);
        assert!(f.active);
        update_flash(&mut f, 0.3);
        assert!(!f.active);
    }

    #[test]
    fn test_flash_alpha() {
        let mut f = new_flash_effect();
        trigger_flash(&mut f, 1.0, 1.0, 1.0, 1.0, 1.0);
        let a = flash_alpha(&f);
        assert!((a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_flash_alpha_inactive() {
        let f = new_flash_effect();
        assert!(flash_alpha(&f).abs() < 1e-6);
    }

    #[test]
    fn test_is_active() {
        let mut f = new_flash_effect();
        assert!(!is_flash_active(&f));
        trigger_flash(&mut f, 1.0, 1.0, 1.0, 1.0, 0.5);
        assert!(is_flash_active(&f));
    }

    #[test]
    fn test_reset_flash() {
        let mut f = new_flash_effect();
        trigger_flash(&mut f, 1.0, 1.0, 1.0, 1.0, 0.5);
        reset_flash(&mut f);
        assert!(!f.active);
    }

    #[test]
    fn test_flash_progress() {
        let mut f = new_flash_effect();
        trigger_flash(&mut f, 1.0, 1.0, 1.0, 1.0, 1.0);
        update_flash(&mut f, 0.5);
        let p = flash_progress(&f);
        assert!((p - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_flash_remaining() {
        let mut f = new_flash_effect();
        trigger_flash(&mut f, 1.0, 1.0, 1.0, 1.0, 1.0);
        update_flash(&mut f, 0.3);
        let r = flash_remaining(&f);
        assert!((r - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_intensity_clamp() {
        let mut f = new_flash_effect();
        trigger_flash(&mut f, 1.0, 1.0, 1.0, 10.0, 0.5);
        assert!((f.intensity - 5.0).abs() < 1e-6);
    }
}
