// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Procedural morph: compute weights from analytical function.

use std::f32::consts::PI;

#[allow(dead_code)]
pub enum ProceduralMorphKind {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

#[allow(dead_code)]
pub struct ProceduralMorph {
    pub kind: ProceduralMorphKind,
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
}

#[allow(dead_code)]
pub fn new_procedural_morph(kind: ProceduralMorphKind, frequency: f32, amplitude: f32) -> ProceduralMorph {
    ProceduralMorph { kind, frequency, amplitude, phase: 0.0 }
}

#[allow(dead_code)]
pub fn pm_evaluate(m: &ProceduralMorph, t: f32) -> f32 {
    let x = m.frequency * t + m.phase;
    let raw = match m.kind {
        ProceduralMorphKind::Sine => (2.0 * PI * x).sin(),
        ProceduralMorphKind::Square => if x.fract() < 0.5 { 1.0 } else { -1.0 },
        ProceduralMorphKind::Triangle => {
            let frac = x.fract();
            if frac < 0.5 { 4.0 * frac - 1.0 } else { 3.0 - 4.0 * frac }
        }
        ProceduralMorphKind::Sawtooth => 2.0 * x.fract() - 1.0,
    };
    raw * m.amplitude
}

#[allow(dead_code)]
pub fn pm_set_phase(m: &mut ProceduralMorph, phase: f32) {
    m.phase = phase;
}

#[allow(dead_code)]
pub fn pm_frequency(m: &ProceduralMorph) -> f32 {
    m.frequency
}

#[allow(dead_code)]
pub fn pm_period(m: &ProceduralMorph) -> f32 {
    if m.frequency.abs() < 1e-7 { f32::INFINITY } else { 1.0 / m.frequency }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_at_zero() {
        let m = new_procedural_morph(ProceduralMorphKind::Sine, 1.0, 1.0);
        let v = pm_evaluate(&m, 0.0);
        assert!(v.abs() < 1e-5);
    }

    #[test]
    fn test_square_positive() {
        let m = new_procedural_morph(ProceduralMorphKind::Square, 1.0, 1.0);
        let v = pm_evaluate(&m, 0.1);
        assert!(v > 0.0);
    }

    #[test]
    fn test_square_negative() {
        let m = new_procedural_morph(ProceduralMorphKind::Square, 1.0, 1.0);
        let v = pm_evaluate(&m, 0.6);
        assert!(v < 0.0);
    }

    #[test]
    fn test_triangle_midpoint() {
        let m = new_procedural_morph(ProceduralMorphKind::Triangle, 1.0, 1.0);
        let v = pm_evaluate(&m, 0.25);
        assert!((v - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_sawtooth_at_zero() {
        let m = new_procedural_morph(ProceduralMorphKind::Sawtooth, 1.0, 1.0);
        let v = pm_evaluate(&m, 0.0);
        assert!((v - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_frequency_getter() {
        let m = new_procedural_morph(ProceduralMorphKind::Sine, 2.0, 1.0);
        assert!((pm_frequency(&m) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_period() {
        let m = new_procedural_morph(ProceduralMorphKind::Sine, 2.0, 1.0);
        assert!((pm_period(&m) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_phase() {
        let mut m = new_procedural_morph(ProceduralMorphKind::Sine, 1.0, 1.0);
        pm_set_phase(&mut m, 0.25);
        assert!((m.phase - 0.25).abs() < 1e-5);
    }
}
