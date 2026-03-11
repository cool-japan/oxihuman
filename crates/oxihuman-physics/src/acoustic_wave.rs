// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::TAU;

/// Acoustic wave model.
#[derive(Debug, Clone)]
pub struct AcousticWaveModel {
    pub speed_of_sound: f32,
    pub frequency: f32,
    pub amplitude: f32,
}

/// Create a new AcousticWaveModel.
pub fn new_acoustic_wave(c: f32, freq: f32, amp: f32) -> AcousticWaveModel {
    AcousticWaveModel {
        speed_of_sound: c,
        frequency: freq,
        amplitude: amp,
    }
}

/// Wavelength: lambda = c / f.
pub fn wavelength(m: &AcousticWaveModel) -> f32 {
    if m.frequency.abs() < 1e-12 {
        return f32::INFINITY;
    }
    m.speed_of_sound / m.frequency
}

/// Instantaneous acoustic pressure: p = A * sin(2π*f*(t - x/c)).
pub fn wave_pressure(m: &AcousticWaveModel, x: f32, t: f32) -> f32 {
    if m.speed_of_sound.abs() < 1e-12 {
        return 0.0;
    }
    m.amplitude * (TAU * m.frequency * (t - x / m.speed_of_sound)).sin()
}

/// Sound intensity: I = A² / (2 * rho * c).
pub fn sound_intensity(m: &AcousticWaveModel, rho: f32) -> f32 {
    let denom = 2.0 * rho * m.speed_of_sound;
    if denom.abs() < 1e-12 {
        return 0.0;
    }
    m.amplitude * m.amplitude / denom
}

/// Sound pressure level (SPL) in decibels: SPL = 20 * log10(p_rms / p_ref).
/// p_ref = 20e-6 Pa (standard).
pub fn decibel_spl(m: &AcousticWaveModel, rho: f32) -> f32 {
    let intensity = sound_intensity(m, rho);
    let p_rms = (intensity * rho * m.speed_of_sound).sqrt();
    let p_ref = 20e-6_f32;
    if p_rms < 1e-30 {
        return -f32::INFINITY;
    }
    20.0 * (p_rms / p_ref).log10()
}

/// Acoustic impedance: Z = rho * c.
pub fn acoustic_impedance(rho: f32, speed: f32) -> f32 {
    rho * speed
}

/// Period: T = 1 / f.
pub fn wave_period(m: &AcousticWaveModel) -> f32 {
    if m.frequency.abs() < 1e-12 {
        return f32::INFINITY;
    }
    1.0 / m.frequency
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_acoustic_wave() {
        /* constructor */
        let m = new_acoustic_wave(343.0, 440.0, 0.5);
        assert!((m.speed_of_sound - 343.0).abs() < 1e-6);
        assert!((m.frequency - 440.0).abs() < 1e-6);
        assert!((m.amplitude - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_wavelength_a440() {
        /* A4 = 440 Hz in air: lambda ≈ 0.78 m */
        let m = new_acoustic_wave(343.0, 440.0, 1.0);
        let lam = wavelength(&m);
        assert!((lam - 343.0 / 440.0).abs() < 1e-4);
    }

    #[test]
    fn test_wavelength_zero_freq() {
        let m = new_acoustic_wave(343.0, 0.0, 1.0);
        assert!(wavelength(&m).is_infinite());
    }

    #[test]
    fn test_wave_pressure_at_origin() {
        /* at x=0, t=0 -> sin(0) = 0 */
        let m = new_acoustic_wave(343.0, 440.0, 1.0);
        let p = wave_pressure(&m, 0.0, 0.0);
        assert!(p.abs() < 1e-6);
    }

    #[test]
    fn test_wave_pressure_bounded() {
        /* pressure amplitude should not exceed amp */
        let m = new_acoustic_wave(343.0, 440.0, 2.0);
        for i in 0..20 {
            let p = wave_pressure(&m, i as f32 * 0.01, i as f32 * 0.001);
            assert!(p.abs() <= 2.001);
        }
    }

    #[test]
    fn test_sound_intensity_positive() {
        let m = new_acoustic_wave(343.0, 440.0, 1.0);
        let i = sound_intensity(&m, 1.2);
        assert!(i > 0.0);
    }

    #[test]
    fn test_decibel_spl_positive_amplitude() {
        /* reasonable SPL for 1 Pa in air */
        let m = new_acoustic_wave(343.0, 440.0, 1.0);
        let spl = decibel_spl(&m, 1.2);
        assert!(spl > 0.0);
    }

    #[test]
    fn test_acoustic_impedance() {
        /* Z = rho * c */
        let z = acoustic_impedance(1.2, 343.0);
        assert!((z - 411.6).abs() < 0.1);
    }

    #[test]
    fn test_wave_period() {
        let m = new_acoustic_wave(343.0, 1000.0, 1.0);
        let p = wave_period(&m);
        assert!((p - 0.001).abs() < 1e-7);
    }
}
