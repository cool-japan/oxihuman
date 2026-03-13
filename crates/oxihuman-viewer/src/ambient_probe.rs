// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Ambient light probe that samples environment colour from 6 face directions.

#![allow(dead_code)]

/// Configuration for the ambient probe.
#[derive(Debug, Clone)]
pub struct AmbientProbeConfig {
    /// Global intensity multiplier.
    pub intensity: f32,
    /// Whether to gamma-correct output samples.
    pub gamma_correct: bool,
}

/// A single face colour sample (linear RGB).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceColor {
    /// Linear RGB colour, 0.0–1.0.
    pub rgb: [f32; 3],
}

/// Ambient light probe storing 6 face colours (±X, ±Y, ±Z).
#[derive(Debug, Clone)]
pub struct AmbientProbe {
    pub config: AmbientProbeConfig,
    /// Colours for the 6 cube faces: +X, -X, +Y, -Y, +Z, -Z.
    pub faces: [FaceColor; 6],
}

/// Returns the default [`AmbientProbeConfig`].
#[allow(dead_code)]
pub fn default_ambient_probe_config() -> AmbientProbeConfig {
    AmbientProbeConfig {
        intensity: 1.0,
        gamma_correct: false,
    }
}

/// Creates a new [`AmbientProbe`] with all faces set to black.
#[allow(dead_code)]
pub fn new_ambient_probe(cfg: AmbientProbeConfig) -> AmbientProbe {
    AmbientProbe {
        config: cfg,
        faces: [FaceColor { rgb: [0.0, 0.0, 0.0] }; 6],
    }
}

/// Sets the colour for a specific face (0..5).  Returns `false` if index is out of range.
#[allow(dead_code)]
pub fn probe_set_face_color(probe: &mut AmbientProbe, face_idx: usize, color: FaceColor) -> bool {
    if face_idx >= 6 {
        return false;
    }
    probe.faces[face_idx] = color;
    true
}

/// Gets the colour for a specific face (0..5).
#[allow(dead_code)]
pub fn probe_get_face_color(probe: &AmbientProbe, face_idx: usize) -> Option<FaceColor> {
    probe.faces.get(face_idx).copied()
}

/// Samples the probe for a direction vector [x, y, z] by returning the dominant face colour.
#[allow(dead_code)]
pub fn probe_sample_direction(probe: &AmbientProbe, dir: [f32; 3]) -> FaceColor {
    let ax = dir[0].abs();
    let ay = dir[1].abs();
    let az = dir[2].abs();
    let face_idx = if ax >= ay && ax >= az {
        if dir[0] >= 0.0 { 0 } else { 1 } // +X or -X
    } else if ay >= ax && ay >= az {
        if dir[1] >= 0.0 { 2 } else { 3 } // +Y or -Y
    } else if dir[2] >= 0.0 {
        4 // +Z
    } else {
        5 // -Z
    };
    let mut fc = probe.faces[face_idx];
    let i = probe.config.intensity;
    fc.rgb = [fc.rgb[0] * i, fc.rgb[1] * i, fc.rgb[2] * i];
    fc
}

/// Computes the average colour across all 6 faces.
#[allow(dead_code)]
pub fn probe_average_color(probe: &AmbientProbe) -> FaceColor {
    let mut sum = [0.0f32; 3];
    for f in &probe.faces {
        sum[0] += f.rgb[0];
        sum[1] += f.rgb[1];
        sum[2] += f.rgb[2];
    }
    FaceColor {
        rgb: [sum[0] / 6.0, sum[1] / 6.0, sum[2] / 6.0],
    }
}

/// Returns the configured intensity.
#[allow(dead_code)]
pub fn probe_intensity(probe: &AmbientProbe) -> f32 {
    probe.config.intensity
}

/// Serialises the probe to a JSON string.
#[allow(dead_code)]
pub fn probe_to_json(probe: &AmbientProbe) -> String {
    let faces: Vec<String> = probe
        .faces
        .iter()
        .map(|f| {
            format!(
                "[{:.4},{:.4},{:.4}]",
                f.rgb[0], f.rgb[1], f.rgb[2]
            )
        })
        .collect();
    format!(
        "{{\"intensity\":{:.4},\"faces\":[{}]}}",
        probe.config.intensity,
        faces.join(",")
    )
}

/// Resets all face colours to black and intensity to 1.0.
#[allow(dead_code)]
pub fn probe_reset(probe: &mut AmbientProbe) {
    probe.faces = [FaceColor { rgb: [0.0, 0.0, 0.0] }; 6];
    probe.config.intensity = 1.0;
}

/// Returns the number of faces (always 6).
#[allow(dead_code)]
pub fn probe_face_count(_probe: &AmbientProbe) -> usize {
    6
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_intensity_is_one() {
        let cfg = default_ambient_probe_config();
        assert!((cfg.intensity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn new_probe_has_six_black_faces() {
        let probe = new_ambient_probe(default_ambient_probe_config());
        assert_eq!(probe_face_count(&probe), 6);
        for &f in &probe.faces {
            assert_eq!(f.rgb, [0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn set_face_color_ok() {
        let mut probe = new_ambient_probe(default_ambient_probe_config());
        let ok = probe_set_face_color(&mut probe, 0, FaceColor { rgb: [1.0, 0.0, 0.0] });
        assert!(ok);
        assert_eq!(probe.faces[0].rgb, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn set_face_color_out_of_range_fails() {
        let mut probe = new_ambient_probe(default_ambient_probe_config());
        assert!(!probe_set_face_color(&mut probe, 6, FaceColor { rgb: [1.0, 0.0, 0.0] }));
    }

    #[test]
    fn get_face_color_roundtrip() {
        let mut probe = new_ambient_probe(default_ambient_probe_config());
        probe_set_face_color(&mut probe, 2, FaceColor { rgb: [0.5, 0.5, 0.5] });
        let got = probe_get_face_color(&probe, 2).expect("should succeed");
        assert_eq!(got.rgb, [0.5, 0.5, 0.5]);
    }

    #[test]
    fn sample_direction_plus_x() {
        let mut probe = new_ambient_probe(default_ambient_probe_config());
        probe_set_face_color(&mut probe, 0, FaceColor { rgb: [1.0, 0.0, 0.0] });
        let fc = probe_sample_direction(&probe, [1.0, 0.0, 0.0]);
        assert!((fc.rgb[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn average_color_is_mean_of_faces() {
        let mut probe = new_ambient_probe(default_ambient_probe_config());
        for i in 0..6 {
            probe_set_face_color(&mut probe, i, FaceColor { rgb: [0.6, 0.6, 0.6] });
        }
        let avg = probe_average_color(&probe);
        assert!((avg.rgb[0] - 0.6).abs() < 1e-4);
    }

    #[test]
    fn probe_reset_clears_state() {
        let mut probe = new_ambient_probe(default_ambient_probe_config());
        probe_set_face_color(&mut probe, 0, FaceColor { rgb: [1.0, 1.0, 1.0] });
        probe.config.intensity = 2.5;
        probe_reset(&mut probe);
        assert_eq!(probe.faces[0].rgb, [0.0, 0.0, 0.0]);
        assert!((probe_intensity(&probe) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn to_json_contains_intensity() {
        let probe = new_ambient_probe(default_ambient_probe_config());
        let json = probe_to_json(&probe);
        assert!(json.contains("\"intensity\""));
        assert!(json.contains("\"faces\""));
    }
}
