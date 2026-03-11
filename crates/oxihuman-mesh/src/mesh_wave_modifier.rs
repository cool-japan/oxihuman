// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Wave deform modifier.

use std::f32::consts::TAU;

/// Configuration for the wave modifier.
#[derive(Debug, Clone)]
pub struct WaveConfig {
    pub amplitude: f32,
    pub wavelength: f32,
    pub phase_offset: f32,
    pub speed: f32,
    pub time: f32,
    pub axis: u8, /* 0=X, 1=Y, 2=Z propagation */
    pub narrow: f32,
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self {
            amplitude: 0.5,
            wavelength: 1.0,
            phase_offset: 0.0,
            speed: 1.0,
            time: 0.0,
            axis: 0,
            narrow: 0.0,
        }
    }
}

impl WaveConfig {
    pub fn new(amplitude: f32, wavelength: f32) -> Self {
        Self { amplitude, wavelength, ..Self::default() }
    }

    pub fn with_time(mut self, t: f32) -> Self {
        self.time = t;
        self
    }
}

/// Apply wave displacement to a single vertex.
pub fn wave_displace(pos: [f32; 3], cfg: &WaveConfig) -> [f32; 3] {
    let driven = match cfg.axis {
        0 => pos[0],
        1 => pos[1],
        _ => pos[2],
    };
    let phase = TAU / cfg.wavelength.max(1e-6) * driven + cfg.phase_offset - cfg.speed * cfg.time;
    let disp = cfg.amplitude * phase.sin();
    /* narrow makes the wave decay away from origin */
    let narrow_factor = if cfg.narrow > 0.0 {
        (-cfg.narrow * driven * driven).exp()
    } else {
        1.0
    };
    let d = disp * narrow_factor;
    let up = (cfg.axis + 2) % 3;
    let mut out = pos;
    out[up as usize] += d;
    out
}

/// Apply wave modifier to all vertices.
pub fn apply_wave(positions: &mut [[f32; 3]], cfg: &WaveConfig) {
    for p in positions.iter_mut() {
        *p = wave_displace(*p, cfg);
    }
}

/// Compute wave phase at a position along the driven axis.
pub fn wave_phase(driven: f32, cfg: &WaveConfig) -> f32 {
    TAU / cfg.wavelength.max(1e-6) * driven + cfg.phase_offset - cfg.speed * cfg.time
}

/// Validate wave config.
pub fn validate_wave_config(cfg: &WaveConfig) -> bool {
    cfg.wavelength > 0.0 && cfg.axis <= 2
}

/// Compute peak displacement for a given amplitude.
pub fn wave_peak_displacement(cfg: &WaveConfig) -> f32 {
    cfg.amplitude.abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_config_default() {
        let cfg = WaveConfig::default();
        assert!((cfg.amplitude - 0.5).abs() < 1e-6);
        assert!((cfg.wavelength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_wave_config_with_time() {
        let cfg = WaveConfig::new(1.0, 2.0).with_time(3.0);
        assert!((cfg.time - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_wave_displace_zero_amplitude() {
        let cfg = WaveConfig { amplitude: 0.0, ..WaveConfig::default() };
        let p = wave_displace([1.0, 2.0, 3.0], &cfg);
        assert!((p[0] - 1.0).abs() < 1e-5);
        assert!((p[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_wave_displace_changes_position() {
        let cfg = WaveConfig::new(1.0, 1.0);
        let p = wave_displace([0.25, 0.0, 0.0], &cfg);
        assert!(p[2].abs() > 0.0 || p[1].abs() > 0.0); /* some axis is displaced */
    }

    #[test]
    fn test_apply_wave_count_preserved() {
        let mut pos = vec![[0.0_f32, 0.0, 0.0]; 5];
        let cfg = WaveConfig::new(0.5, 2.0);
        apply_wave(&mut pos, &cfg);
        assert_eq!(pos.len(), 5);
    }

    #[test]
    fn test_wave_phase_at_origin_no_speed() {
        let cfg = WaveConfig::default();
        let phase = wave_phase(0.0, &cfg);
        assert!((phase - cfg.phase_offset).abs() < 1e-5);
    }

    #[test]
    fn test_validate_wave_config_valid() {
        let cfg = WaveConfig::default();
        assert!(validate_wave_config(&cfg));
    }

    #[test]
    fn test_validate_wave_config_zero_wavelength() {
        let cfg = WaveConfig { wavelength: 0.0, ..WaveConfig::default() };
        assert!(!validate_wave_config(&cfg));
    }

    #[test]
    fn test_wave_peak_displacement() {
        let cfg = WaveConfig { amplitude: -2.0, ..WaveConfig::default() };
        assert!((wave_peak_displacement(&cfg) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_wave_narrow_reduces_amplitude() {
        let cfg_narrow = WaveConfig { narrow: 10.0, amplitude: 1.0, ..WaveConfig::default() };
        let cfg_flat = WaveConfig { narrow: 0.0, amplitude: 1.0, ..WaveConfig::default() };
        let p = [5.0_f32, 0.0, 0.0];
        let dn = wave_displace(p, &cfg_narrow);
        let df = wave_displace(p, &cfg_flat);
        /* narrow version should displace less far from origin */
        let narrow_disp = (dn[2] - p[2]).abs();
        let flat_disp = (df[2] - p[2]).abs();
        assert!(narrow_disp <= flat_disp + 1e-5);
    }
}
