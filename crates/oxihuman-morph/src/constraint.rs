// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::params::ParamState;

/// Clamp all ParamState values to [0.0, 1.0].
pub fn clamp_params(p: &mut ParamState) {
    p.height = p.height.clamp(0.0, 1.0);
    p.weight = p.weight.clamp(0.0, 1.0);
    p.muscle = p.muscle.clamp(0.0, 1.0);
    p.age = p.age.clamp(0.0, 1.0);
    for v in p.extra.values_mut() {
        *v = v.clamp(0.0, 1.0);
    }
}

/// Smooth step curve: 3t² - 2t³ (maps `[0,1]` → `[0,1]` with smooth ends).
pub fn smooth_step(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Apply smooth_step to all params (for organic response curves).
pub fn smooth_params(p: &ParamState) -> ParamState {
    let mut out = p.clone();
    out.height = smooth_step(p.height);
    out.weight = smooth_step(p.weight);
    out.muscle = smooth_step(p.muscle);
    out.age = smooth_step(p.age);
    for (k, v) in &p.extra {
        out.extra.insert(k.clone(), smooth_step(*v));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_out_of_range() {
        let mut p = ParamState::new(1.5, -0.3, 0.5, 2.0);
        clamp_params(&mut p);
        assert!((p.height - 1.0).abs() < 1e-6);
        assert!((p.weight - 0.0).abs() < 1e-6);
        assert!((p.age - 1.0).abs() < 1e-6);
    }

    #[test]
    fn smooth_step_endpoints() {
        assert!((smooth_step(0.0) - 0.0).abs() < 1e-6);
        assert!((smooth_step(1.0) - 1.0).abs() < 1e-6);
        assert!((smooth_step(0.5) - 0.5).abs() < 1e-6);
    }
}
