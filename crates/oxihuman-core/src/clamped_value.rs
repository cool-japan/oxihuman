// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Value clamped to [min, max] range.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClampedConfig {
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClampedValue {
    pub value: f32,
    pub config: ClampedConfig,
    pub changed: bool,
}

#[allow(dead_code)]
pub fn default_clamped_config() -> ClampedConfig {
    ClampedConfig { min: 0.0, max: 1.0, default: 0.0 }
}

#[allow(dead_code)]
pub fn new_clamped_value(config: ClampedConfig) -> ClampedValue {
    let v = config.default.clamp(config.min, config.max);
    ClampedValue { value: v, config, changed: false }
}

#[allow(dead_code)]
pub fn cv_set(cv: &mut ClampedValue, v: f32) {
    let clamped = v.clamp(cv.config.min, cv.config.max);
    if (clamped - cv.value).abs() > f32::EPSILON {
        cv.changed = true;
    }
    cv.value = clamped;
}

#[allow(dead_code)]
pub fn cv_get(cv: &ClampedValue) -> f32 {
    cv.value
}

#[allow(dead_code)]
pub fn cv_nudge(cv: &mut ClampedValue, delta: f32) {
    cv_set(cv, cv.value + delta);
}

#[allow(dead_code)]
pub fn cv_reset(cv: &mut ClampedValue) {
    let default = cv.config.default;
    cv_set(cv, default);
}

#[allow(dead_code)]
pub fn cv_at_min(cv: &ClampedValue) -> bool {
    (cv.value - cv.config.min).abs() < f32::EPSILON
}

#[allow(dead_code)]
pub fn cv_at_max(cv: &ClampedValue) -> bool {
    (cv.value - cv.config.max).abs() < f32::EPSILON
}

#[allow(dead_code)]
pub fn cv_normalized(cv: &ClampedValue) -> f32 {
    let range = cv.config.max - cv.config.min;
    if range < f32::EPSILON {
        return 0.0;
    }
    (cv.value - cv.config.min) / range
}

#[allow(dead_code)]
pub fn cv_was_changed(cv: &ClampedValue) -> bool {
    cv.changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_clamped_config();
        assert_eq!(cfg.min, 0.0);
        assert_eq!(cfg.max, 1.0);
    }

    #[test]
    fn test_new_clamped_value() {
        let cfg = ClampedConfig { min: 0.0, max: 10.0, default: 5.0 };
        let cv = new_clamped_value(cfg);
        assert_eq!(cv_get(&cv), 5.0);
    }

    #[test]
    fn test_cv_set_clamps() {
        let cfg = ClampedConfig { min: 0.0, max: 1.0, default: 0.0 };
        let mut cv = new_clamped_value(cfg);
        cv_set(&mut cv, 5.0);
        assert_eq!(cv_get(&cv), 1.0);
        cv_set(&mut cv, -5.0);
        assert_eq!(cv_get(&cv), 0.0);
    }

    #[test]
    fn test_cv_nudge() {
        let cfg = ClampedConfig { min: 0.0, max: 10.0, default: 5.0 };
        let mut cv = new_clamped_value(cfg);
        cv_nudge(&mut cv, 2.0);
        assert!((cv_get(&cv) - 7.0).abs() < 1e-5);
    }

    #[test]
    fn test_cv_reset() {
        let cfg = ClampedConfig { min: 0.0, max: 10.0, default: 3.0 };
        let mut cv = new_clamped_value(cfg);
        cv_set(&mut cv, 9.0);
        cv_reset(&mut cv);
        assert!((cv_get(&cv) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_cv_at_min_max() {
        let cfg = ClampedConfig { min: 0.0, max: 1.0, default: 0.0 };
        let mut cv = new_clamped_value(cfg);
        assert!(cv_at_min(&cv));
        cv_set(&mut cv, 1.0);
        assert!(cv_at_max(&cv));
    }

    #[test]
    fn test_cv_normalized() {
        let cfg = ClampedConfig { min: 0.0, max: 4.0, default: 2.0 };
        let cv = new_clamped_value(cfg);
        assert!((cv_normalized(&cv) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_cv_was_changed() {
        let cfg = ClampedConfig { min: 0.0, max: 1.0, default: 0.0 };
        let mut cv = new_clamped_value(cfg);
        assert!(!cv_was_changed(&cv));
        cv_set(&mut cv, 0.5);
        assert!(cv_was_changed(&cv));
    }

    #[test]
    fn test_cv_normalized_zero_range() {
        let cfg = ClampedConfig { min: 5.0, max: 5.0, default: 5.0 };
        let cv = new_clamped_value(cfg);
        assert_eq!(cv_normalized(&cv), 0.0);
    }
}
