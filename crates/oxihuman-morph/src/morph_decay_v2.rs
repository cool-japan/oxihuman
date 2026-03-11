// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Morph decay with exponential and linear falloff modes.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DecayMode {
    Exponential,
    Linear,
    Step,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphDecayV2 {
    pub mode: DecayMode,
    pub rate: f32,
    pub value: f32,
}

#[allow(dead_code)]
pub fn new_morph_decay_v2(mode: DecayMode, rate: f32, initial: f32) -> MorphDecayV2 {
    MorphDecayV2 { mode, rate, value: initial }
}

#[allow(dead_code)]
pub fn mdv2_step(decay: &mut MorphDecayV2, dt: f32) {
    match decay.mode {
        DecayMode::Exponential => {
            decay.value *= (-decay.rate * dt).exp();
        }
        DecayMode::Linear => {
            decay.value -= decay.rate * dt;
            if decay.value < 0.0 {
                decay.value = 0.0;
            }
        }
        DecayMode::Step => {
            decay.value = 0.0;
        }
    }
}

#[allow(dead_code)]
pub fn mdv2_value(decay: &MorphDecayV2) -> f32 {
    decay.value
}

#[allow(dead_code)]
pub fn mdv2_is_settled(decay: &MorphDecayV2, tol: f32) -> bool {
    decay.value.abs() < tol
}

#[allow(dead_code)]
pub fn mdv2_reset(decay: &mut MorphDecayV2, v: f32) {
    decay.value = v;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_decay_reduces() {
        let mut d = new_morph_decay_v2(DecayMode::Exponential, 1.0, 1.0);
        mdv2_step(&mut d, 1.0);
        assert!(mdv2_value(&d) < 1.0);
    }

    #[test]
    fn test_exponential_decay_positive() {
        let mut d = new_morph_decay_v2(DecayMode::Exponential, 2.0, 0.5);
        mdv2_step(&mut d, 0.1);
        assert!(d.value > 0.0);
    }

    #[test]
    fn test_linear_decay_reduces() {
        let mut d = new_morph_decay_v2(DecayMode::Linear, 0.5, 1.0);
        mdv2_step(&mut d, 1.0);
        assert!((mdv2_value(&d) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_linear_decay_clamps_zero() {
        let mut d = new_morph_decay_v2(DecayMode::Linear, 10.0, 0.1);
        mdv2_step(&mut d, 1.0);
        assert_eq!(mdv2_value(&d), 0.0);
    }

    #[test]
    fn test_step_snaps_to_zero() {
        let mut d = new_morph_decay_v2(DecayMode::Step, 1.0, 99.0);
        mdv2_step(&mut d, 0.001);
        assert_eq!(mdv2_value(&d), 0.0);
    }

    #[test]
    fn test_is_settled_true() {
        let d = new_morph_decay_v2(DecayMode::Linear, 1.0, 0.0001);
        assert!(mdv2_is_settled(&d, 0.001));
    }

    #[test]
    fn test_is_settled_false() {
        let d = new_morph_decay_v2(DecayMode::Linear, 1.0, 0.5);
        assert!(!mdv2_is_settled(&d, 0.001));
    }

    #[test]
    fn test_reset() {
        let mut d = new_morph_decay_v2(DecayMode::Exponential, 1.0, 1.0);
        mdv2_step(&mut d, 1.0);
        mdv2_reset(&mut d, 2.0);
        assert!((mdv2_value(&d) - 2.0).abs() < 1e-6);
    }
}
