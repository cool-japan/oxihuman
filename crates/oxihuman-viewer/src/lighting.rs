// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Lighting presets and HDR environment light descriptors.

// ── Types ──────────────────────────────────────────────────────────────────

/// A directional (infinite-distance) light, e.g. the sun.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DirectionalLight {
    /// Normalized direction the light travels *toward* (world space).
    pub direction: [f32; 3],
    /// Linear RGB color.
    pub color: [f32; 3],
    /// Illuminance in lux.
    pub intensity: f32,
}

/// A point light that radiates equally in all directions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PointLight {
    /// World-space position.
    pub position: [f32; 3],
    /// Linear RGB color.
    pub color: [f32; 3],
    /// Luminous intensity in candela.
    pub intensity: f32,
    /// Effective maximum range in world units.
    pub range: f32,
}

/// Ambient (hemispherical) fill light.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AmbientLight {
    /// Linear RGB color.
    pub color: [f32; 3],
    /// Relative intensity in [0, 1].
    pub intensity: f32,
}

/// Descriptor for an HDRI environment map.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HdriDescriptor {
    pub name: String,
    /// Path on disk; `None` means the environment is procedurally generated.
    pub path: Option<String>,
    /// Rotation around the vertical axis in degrees.
    pub rotation_deg: f32,
    /// Global intensity multiplier.
    pub intensity: f32,
}

/// Complete lighting setup for a scene.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightingSetup {
    pub directional: Vec<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
    pub ambient: AmbientLight,
    pub hdri: Option<HdriDescriptor>,
}

// ── LightingSetup presets ─────────────────────────────────────────────────

impl LightingSetup {
    /// Classic three-point studio lighting for character presentation.
    ///
    /// * Key  – bright warm front-left
    /// * Fill – cool dim front-right
    /// * Back – white rim from behind
    pub fn studio_three_point() -> Self {
        let key = DirectionalLight {
            direction: normalize_dir([-0.6, -0.4, 0.7]),
            color: kelvin_to_rgb(5600.0),
            intensity: 2000.0,
        };
        let fill = DirectionalLight {
            direction: normalize_dir([0.7, -0.3, 0.6]),
            color: kelvin_to_rgb(7500.0),
            intensity: 600.0,
        };
        let back = DirectionalLight {
            direction: normalize_dir([0.0, -0.2, -1.0]),
            color: [1.0, 1.0, 1.0],
            intensity: 900.0,
        };
        LightingSetup {
            directional: vec![key, fill, back],
            point_lights: vec![],
            ambient: AmbientLight {
                color: [1.0, 1.0, 1.0],
                intensity: 0.05,
            },
            hdri: None,
        }
    }

    /// Strong overhead sun with bright blue-sky ambient.
    pub fn outdoor_noon() -> Self {
        LightingSetup {
            directional: vec![DirectionalLight {
                direction: normalize_dir([0.15, -1.0, 0.1]),
                color: kelvin_to_rgb(6500.0),
                intensity: 100_000.0,
            }],
            point_lights: vec![],
            ambient: AmbientLight {
                color: srgb_to_linear_color([0.53, 0.81, 0.98]),
                intensity: 0.7,
            },
            hdri: Some(HdriDescriptor {
                name: "outdoor_noon".to_string(),
                path: None,
                rotation_deg: 0.0,
                intensity: 1.0,
            }),
        }
    }

    /// Low warm sun near the horizon plus orange-tinted ambient sky.
    pub fn outdoor_sunset() -> Self {
        LightingSetup {
            directional: vec![DirectionalLight {
                direction: normalize_dir([-0.9, -0.2, 0.3]),
                color: kelvin_to_rgb(2200.0),
                intensity: 8_000.0,
            }],
            point_lights: vec![],
            ambient: AmbientLight {
                color: srgb_to_linear_color([1.0, 0.55, 0.2]),
                intensity: 0.3,
            },
            hdri: Some(HdriDescriptor {
                name: "outdoor_sunset".to_string(),
                path: None,
                rotation_deg: 180.0,
                intensity: 0.8,
            }),
        }
    }

    /// Soft interior lighting with dim fill points and no harsh directional.
    pub fn indoor_soft() -> Self {
        LightingSetup {
            directional: vec![],
            point_lights: vec![
                PointLight {
                    position: [0.0, 2.5, 0.0],
                    color: kelvin_to_rgb(3200.0),
                    intensity: 800.0,
                    range: 6.0,
                },
                PointLight {
                    position: [1.5, 2.0, 1.5],
                    color: kelvin_to_rgb(3000.0),
                    intensity: 400.0,
                    range: 4.0,
                },
            ],
            ambient: AmbientLight {
                color: [1.0, 0.95, 0.9],
                intensity: 0.25,
            },
            hdri: None,
        }
    }

    /// High-contrast dramatic lighting with a single strong key and near-zero fill.
    pub fn dark_dramatic() -> Self {
        LightingSetup {
            directional: vec![DirectionalLight {
                direction: normalize_dir([-0.5, -0.6, 0.6]),
                color: kelvin_to_rgb(5800.0),
                intensity: 4_000.0,
            }],
            point_lights: vec![],
            ambient: AmbientLight {
                color: [1.0, 1.0, 1.0],
                intensity: 0.01,
            },
            hdri: None,
        }
    }

    /// Total number of lights in this setup (directional + point).
    pub fn total_light_count(&self) -> usize {
        self.directional.len() + self.point_lights.len()
    }

    /// Serialize to a compact JSON string (no external crate required).
    pub fn to_json(&self) -> String {
        let dir_json: Vec<String> = self
            .directional
            .iter()
            .map(|l| {
                format!(
                    r#"{{"dir":[{:.4},{:.4},{:.4}],"color":[{:.4},{:.4},{:.4}],"intensity":{:.2}}}"#,
                    l.direction[0],
                    l.direction[1],
                    l.direction[2],
                    l.color[0],
                    l.color[1],
                    l.color[2],
                    l.intensity,
                )
            })
            .collect();

        let pt_json: Vec<String> = self
            .point_lights
            .iter()
            .map(|l| {
                format!(
                    r#"{{"pos":[{:.4},{:.4},{:.4}],"color":[{:.4},{:.4},{:.4}],"intensity":{:.2},"range":{:.4}}}"#,
                    l.position[0],
                    l.position[1],
                    l.position[2],
                    l.color[0],
                    l.color[1],
                    l.color[2],
                    l.intensity,
                    l.range,
                )
            })
            .collect();

        let hdri_json = match &self.hdri {
            None => "null".to_string(),
            Some(h) => {
                let path_str = match &h.path {
                    None => "null".to_string(),
                    Some(p) => format!("\"{}\"", p),
                };
                format!(
                    r#"{{"name":"{}","path":{},"rotation_deg":{:.2},"intensity":{:.4}}}"#,
                    h.name, path_str, h.rotation_deg, h.intensity,
                )
            }
        };

        format!(
            r#"{{"directional":[{}],"point_lights":[{}],"ambient":{{"color":[{:.4},{:.4},{:.4}],"intensity":{:.4}}},"hdri":{}}}"#,
            dir_json.join(","),
            pt_json.join(","),
            self.ambient.color[0],
            self.ambient.color[1],
            self.ambient.color[2],
            self.ambient.intensity,
            hdri_json,
        )
    }
}

// ── Utility functions ──────────────────────────────────────────────────────

/// Normalize a direction vector to unit length.
///
/// Returns `[0, 0, 0]` if the input has negligible magnitude.
pub fn normalize_dir(d: [f32; 3]) -> [f32; 3] {
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 0.0]
    } else {
        [d[0] / len, d[1] / len, d[2] / len]
    }
}

/// Convert sRGB (gamma-encoded, 0..1) to linear RGB.
pub fn srgb_to_linear_color(c: [f32; 3]) -> [f32; 3] {
    fn channel(v: f32) -> f32 {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    [channel(c[0]), channel(c[1]), channel(c[2])]
}

/// Approximate color temperature (Kelvin) → linear RGB using the
/// Planckian-locus polynomial fit by Kévin Beason / Charity.
///
/// Valid range: roughly 1000 K – 40 000 K.
/// Output channels are in [0, 1] linear.
pub fn kelvin_to_rgb(temp_k: f32) -> [f32; 3] {
    let t = temp_k.clamp(1000.0, 40_000.0) / 100.0;

    // Red channel
    let r = if t <= 66.0 {
        1.0f32
    } else {
        let v = 329.698_73 * (t - 60.0).powf(-0.133_204_76);
        (v / 255.0).clamp(0.0, 1.0)
    };

    // Green channel
    let g = if t <= 66.0 {
        let v = 99.470_8 * t.ln() - 161.119_6;
        (v / 255.0).clamp(0.0, 1.0)
    } else {
        let v = 288.122_17 * (t - 60.0).powf(-0.075_514_85);
        (v / 255.0).clamp(0.0, 1.0)
    };

    // Blue channel
    let b = if t >= 66.0 {
        1.0f32
    } else if t <= 19.0 {
        0.0f32
    } else {
        let v = 138.517_7 * (t - 10.0).ln() - 305.044_8;
        (v / 255.0).clamp(0.0, 1.0)
    };

    // Values are already approximately linear from the Planckian locus fit
    [r, g, b]
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // helper: dot product
    fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    // ── normalize_dir ────────────────────────────────────────────────────

    #[test]
    fn normalize_dir_unit_length() {
        let d = normalize_dir([1.0, 2.0, 3.0]);
        let len = (dot3(d, d)).sqrt();
        assert!((len - 1.0).abs() < 1e-5, "length should be 1.0, got {len}");
    }

    #[test]
    fn normalize_dir_zero_vector() {
        let d = normalize_dir([0.0, 0.0, 0.0]);
        assert_eq!(d, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn normalize_dir_already_unit() {
        let d = normalize_dir([1.0, 0.0, 0.0]);
        assert!((d[0] - 1.0).abs() < 1e-6);
        assert!(d[1].abs() < 1e-6);
        assert!(d[2].abs() < 1e-6);
    }

    // ── srgb_to_linear_color ──────────────────────────────────────────────

    #[test]
    fn srgb_to_linear_white_stays_white() {
        let lin = srgb_to_linear_color([1.0, 1.0, 1.0]);
        for ch in lin {
            assert!((ch - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn srgb_to_linear_black_stays_black() {
        let lin = srgb_to_linear_color([0.0, 0.0, 0.0]);
        for ch in lin {
            assert!(ch.abs() < 1e-6);
        }
    }

    #[test]
    fn srgb_to_linear_direction_monotone() {
        // Converting a brighter sRGB value should yield a brighter linear value
        let dim = srgb_to_linear_color([0.2, 0.2, 0.2]);
        let bright = srgb_to_linear_color([0.8, 0.8, 0.8]);
        assert!(bright[0] > dim[0], "linear should be monotone with sRGB");
    }

    // ── kelvin_to_rgb ─────────────────────────────────────────────────────

    #[test]
    fn kelvin_6500k_near_white() {
        let c = kelvin_to_rgb(6500.0);
        // D65 should be close to neutral white
        assert!(c[0] > 0.9, "R should be near 1 at 6500K, got {}", c[0]);
        assert!(c[1] > 0.9, "G should be near 1 at 6500K, got {}", c[1]);
        assert!(c[2] > 0.9, "B should be near 1 at 6500K, got {}", c[2]);
    }

    #[test]
    fn kelvin_2700k_is_warm() {
        // Warm (incandescent-like) colors: more red/green than blue
        let c = kelvin_to_rgb(2700.0);
        assert!(
            c[0] > c[2],
            "2700K should have more red than blue, got {:?}",
            c
        );
    }

    #[test]
    fn kelvin_10000k_is_cool_blue() {
        // Cool (overcast sky-like) colors: more blue influence
        let c = kelvin_to_rgb(10_000.0);
        // Red should be noticeably less than at 6500 K
        let c_neutral = kelvin_to_rgb(6500.0);
        assert!(
            c[0] <= c_neutral[0] + 0.05,
            "10000K should have less/equal red than 6500K, got {:?} vs {:?}",
            c,
            c_neutral
        );
    }

    #[test]
    fn kelvin_clamps_low_end() {
        let c = kelvin_to_rgb(500.0); // below 1000 K → clamped to 1000 K
        let c_1k = kelvin_to_rgb(1000.0);
        assert_eq!(c, c_1k);
    }

    // ── studio_three_point preset ─────────────────────────────────────────

    #[test]
    fn studio_three_point_has_three_directional_lights() {
        let setup = LightingSetup::studio_three_point();
        assert_eq!(
            setup.directional.len(),
            3,
            "studio_three_point must have exactly 3 directional lights"
        );
    }

    #[test]
    fn studio_three_point_key_brighter_than_fill() {
        let setup = LightingSetup::studio_three_point();
        let key_intensity = setup.directional[0].intensity;
        let fill_intensity = setup.directional[1].intensity;
        assert!(
            key_intensity > fill_intensity,
            "key light should be brighter than fill"
        );
    }

    #[test]
    fn studio_three_point_no_point_lights() {
        let setup = LightingSetup::studio_three_point();
        assert!(setup.point_lights.is_empty());
    }

    // ── outdoor_noon preset ───────────────────────────────────────────────

    #[test]
    fn outdoor_noon_has_strong_directional() {
        let setup = LightingSetup::outdoor_noon();
        assert_eq!(setup.directional.len(), 1);
        assert!(
            setup.directional[0].intensity > 10_000.0,
            "noon sun should be very bright (>10,000 lux)"
        );
    }

    // ── total_light_count ─────────────────────────────────────────────────

    #[test]
    fn total_light_count_studio() {
        assert_eq!(LightingSetup::studio_three_point().total_light_count(), 3);
    }

    #[test]
    fn total_light_count_indoor_soft() {
        let setup = LightingSetup::indoor_soft();
        assert_eq!(setup.total_light_count(), 2); // 0 directional + 2 point
    }

    // ── to_json ───────────────────────────────────────────────────────────

    #[test]
    fn to_json_is_non_empty_string() {
        let json = LightingSetup::studio_three_point().to_json();
        assert!(!json.is_empty());
    }

    #[test]
    fn to_json_contains_expected_keys() {
        let json = LightingSetup::outdoor_noon().to_json();
        assert!(
            json.contains("directional"),
            "JSON should contain 'directional'"
        );
        assert!(json.contains("ambient"), "JSON should contain 'ambient'");
        assert!(
            json.contains("intensity"),
            "JSON should contain 'intensity'"
        );
    }

    // ── all presets are non-empty and have positive total_light_count ─────

    #[test]
    fn all_presets_total_light_count_positive() {
        let presets = [
            LightingSetup::studio_three_point(),
            LightingSetup::outdoor_noon(),
            LightingSetup::outdoor_sunset(),
            LightingSetup::indoor_soft(),
            LightingSetup::dark_dramatic(),
        ];
        for p in &presets {
            assert!(
                p.total_light_count() > 0,
                "every preset must have at least one light"
            );
        }
    }

    #[test]
    fn all_presets_have_positive_ambient_intensity() {
        let presets = [
            LightingSetup::studio_three_point(),
            LightingSetup::outdoor_noon(),
            LightingSetup::outdoor_sunset(),
            LightingSetup::indoor_soft(),
            LightingSetup::dark_dramatic(),
        ];
        for p in &presets {
            assert!(
                p.ambient.intensity >= 0.0,
                "ambient intensity must be non-negative"
            );
        }
    }
}
