// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Predefined lighting configurations for the 3D viewer.
//!
//! Each preset defines a complete lighting environment with positioned lights,
//! ambient contribution, and exposure control. The presets use `f64` precision
//! for compatibility with scientific / medical visualization pipelines.

use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Kind of light source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightKind {
    /// Infinitely distant light (like the sun).
    Directional,
    /// Omni-directional point emitter.
    Point,
    /// Cone-shaped emitter with inner/outer falloff angles (radians).
    Spot { inner_angle: f64, outer_angle: f64 },
}

/// A single light source in the scene.
#[derive(Debug, Clone)]
pub struct Light {
    /// What kind of light this is.
    pub kind: LightKind,
    /// World-space position (ignored for `Directional`).
    pub position: [f64; 3],
    /// Normalized direction the light points toward.
    pub direction: [f64; 3],
    /// Linear RGB colour.
    pub color: [f64; 3],
    /// Intensity multiplier (interpretation depends on `kind`).
    pub intensity: f64,
    /// Effective radius / range for falloff.
    pub radius: f64,
}

/// A complete lighting preset for the 3D viewer.
#[derive(Debug, Clone)]
pub struct LightingPreset {
    /// Human-readable name.
    pub name: String,
    /// All lights in the preset.
    pub lights: Vec<Light>,
    /// Ambient RGB contribution (linear).
    pub ambient: [f64; 3],
    /// Exposure value (EV) — higher values brighten the scene.
    pub exposure: f64,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn normalize_f64(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-15 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Approximate colour temperature (Kelvin) to linear RGB.
///
/// Uses a Planckian-locus polynomial fit. Valid range ~1000–40 000 K.
fn kelvin_to_rgb_f64(temp_k: f64) -> [f64; 3] {
    let t = temp_k.clamp(1000.0, 40_000.0) / 100.0;

    let r = if t <= 66.0 {
        1.0
    } else {
        let v = 329.698_73 * (t - 60.0).powf(-0.133_204_76);
        (v / 255.0).clamp(0.0, 1.0)
    };

    let g = if t <= 66.0 {
        let v = 99.470_8 * t.ln() - 161.119_6;
        (v / 255.0).clamp(0.0, 1.0)
    } else {
        let v = 288.122_17 * (t - 60.0).powf(-0.075_514_85);
        (v / 255.0).clamp(0.0, 1.0)
    };

    let b = if t >= 66.0 {
        1.0
    } else if t <= 19.0 {
        0.0
    } else {
        let v = 138.517_7 * (t - 10.0).ln() - 305.044_8;
        (v / 255.0).clamp(0.0, 1.0)
    };

    [r, g, b]
}

fn directional_light(direction: [f64; 3], color: [f64; 3], intensity: f64) -> Light {
    Light {
        kind: LightKind::Directional,
        position: [0.0; 3],
        direction: normalize_f64(direction),
        color,
        intensity,
        radius: f64::INFINITY,
    }
}

fn point_light(position: [f64; 3], color: [f64; 3], intensity: f64, radius: f64) -> Light {
    Light {
        kind: LightKind::Point,
        position,
        direction: [0.0, -1.0, 0.0],
        color,
        intensity,
        radius,
    }
}

fn spot_light(
    position: [f64; 3],
    direction: [f64; 3],
    color: [f64; 3],
    intensity: f64,
    radius: f64,
    inner_angle: f64,
    outer_angle: f64,
) -> Light {
    Light {
        kind: LightKind::Spot {
            inner_angle,
            outer_angle,
        },
        position,
        direction: normalize_f64(direction),
        color,
        intensity,
        radius,
    }
}

// ── LightingPreset ────────────────────────────────────────────────────────────

impl LightingPreset {
    // ── Factory presets ────────────────────────────────────────────────────

    /// Classic three-point studio lighting.
    ///
    /// * Key light — bright warm from upper-left
    /// * Fill light — cooler, dimmer from right
    /// * Back/rim light — from behind to separate subject from background
    pub fn studio() -> Self {
        let key = directional_light([-0.6, -0.5, 0.65], kelvin_to_rgb_f64(5600.0), 2.0);
        let fill = directional_light([0.7, -0.3, 0.6], kelvin_to_rgb_f64(7500.0), 0.6);
        let back = directional_light([0.0, -0.2, -1.0], [1.0, 1.0, 1.0], 0.9);
        LightingPreset {
            name: "Studio (3-Point)".to_string(),
            lights: vec![key, fill, back],
            ambient: [0.05, 0.05, 0.05],
            exposure: 1.0,
        }
    }

    /// Outdoor sun + sky ambient lighting.
    ///
    /// Single strong directional for the sun, blue-tinted ambient from sky.
    pub fn outdoor() -> Self {
        let sun = directional_light([0.15, -1.0, 0.1], kelvin_to_rgb_f64(6500.0), 3.5);
        // Subtle fill from ground bounce
        let bounce = directional_light([0.0, 1.0, 0.0], [0.4, 0.35, 0.3], 0.15);
        LightingPreset {
            name: "Outdoor".to_string(),
            lights: vec![sun, bounce],
            ambient: [0.25, 0.35, 0.45],
            exposure: 1.2,
        }
    }

    /// Warm interior lighting from overhead and accent point sources.
    pub fn indoor() -> Self {
        let overhead = point_light([0.0, 2.8, 0.0], kelvin_to_rgb_f64(3200.0), 1.8, 6.0);
        let accent_left = point_light([-2.0, 1.8, 1.5], kelvin_to_rgb_f64(2900.0), 0.8, 4.0);
        let accent_right = point_light([2.0, 1.5, -0.5], kelvin_to_rgb_f64(3400.0), 0.6, 3.5);
        LightingPreset {
            name: "Indoor".to_string(),
            lights: vec![overhead, accent_left, accent_right],
            ambient: [0.12, 0.10, 0.08],
            exposure: 0.9,
        }
    }

    /// Flat, even lighting for medical / body examination.
    ///
    /// Multiple directional lights from cardinal directions with neutral colour
    /// and strong ambient to minimise shadows.
    pub fn medical() -> Self {
        let front = directional_light([0.0, -0.3, 1.0], [1.0, 1.0, 1.0], 1.0);
        let back = directional_light([0.0, -0.3, -1.0], [1.0, 1.0, 1.0], 0.8);
        let left = directional_light([-1.0, -0.3, 0.0], [1.0, 1.0, 1.0], 0.9);
        let right = directional_light([1.0, -0.3, 0.0], [1.0, 1.0, 1.0], 0.9);
        let top = directional_light([0.0, -1.0, 0.0], [1.0, 1.0, 1.0], 0.7);
        let bottom = directional_light([0.0, 1.0, 0.0], [0.9, 0.9, 0.95], 0.4);
        LightingPreset {
            name: "Medical".to_string(),
            lights: vec![front, back, left, right, top, bottom],
            ambient: [0.35, 0.35, 0.35],
            exposure: 1.0,
        }
    }

    /// High-contrast dramatic lighting with a single strong key light.
    pub fn dramatic() -> Self {
        let key = spot_light(
            [-3.0, 4.0, 2.0],
            [0.6, -0.7, -0.4],
            kelvin_to_rgb_f64(4800.0),
            4.0,
            10.0,
            std::f64::consts::PI / 12.0, // 15 degrees
            std::f64::consts::PI / 6.0,  // 30 degrees
        );
        LightingPreset {
            name: "Dramatic".to_string(),
            lights: vec![key],
            ambient: [0.01, 0.01, 0.02],
            exposure: 1.4,
        }
    }

    /// Rim / silhouette emphasis — light from behind and sides, minimal front.
    pub fn rim_light() -> Self {
        let back_left = directional_light([-0.5, -0.2, -1.0], kelvin_to_rgb_f64(6000.0), 2.5);
        let back_right = directional_light([0.5, -0.2, -1.0], kelvin_to_rgb_f64(6000.0), 2.5);
        let subtle_front = directional_light([0.0, -0.4, 1.0], [0.5, 0.5, 0.6], 0.15);
        LightingPreset {
            name: "Rim Light".to_string(),
            lights: vec![back_left, back_right, subtle_front],
            ambient: [0.02, 0.02, 0.03],
            exposure: 1.3,
        }
    }

    /// Build a custom preset from user-provided lights.
    pub fn custom(lights: Vec<Light>, ambient: [f64; 3]) -> Self {
        LightingPreset {
            name: "Custom".to_string(),
            lights,
            ambient,
            exposure: 1.0,
        }
    }

    /// Return all built-in presets.
    pub fn all_presets() -> Vec<Self> {
        vec![
            Self::studio(),
            Self::outdoor(),
            Self::indoor(),
            Self::medical(),
            Self::dramatic(),
            Self::rim_light(),
        ]
    }

    // ── Queries ───────────────────────────────────────────────────────────

    /// Total number of lights in this preset.
    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    /// Collect lights by kind.
    pub fn lights_by_kind(&self) -> HashMap<&'static str, Vec<&Light>> {
        let mut map: HashMap<&'static str, Vec<&Light>> = HashMap::new();
        for light in &self.lights {
            let key = match light.kind {
                LightKind::Directional => "directional",
                LightKind::Point => "point",
                LightKind::Spot { .. } => "spot",
            };
            map.entry(key).or_default().push(light);
        }
        map
    }

    /// Compute the total intensity across all lights (sum of individual intensities).
    pub fn total_intensity(&self) -> f64 {
        self.lights.iter().map(|l| l.intensity).sum()
    }

    /// Serialize to a compact JSON string.
    pub fn to_json(&self) -> String {
        let lights_json: Vec<String> = self
            .lights
            .iter()
            .map(|l| {
                let kind_str = match l.kind {
                    LightKind::Directional => "\"directional\"".to_string(),
                    LightKind::Point => "\"point\"".to_string(),
                    LightKind::Spot {
                        inner_angle,
                        outer_angle,
                    } => format!(
                        r#"{{"spot":{{"inner_angle":{:.6},"outer_angle":{:.6}}}}}"#,
                        inner_angle, outer_angle
                    ),
                };
                format!(
                    r#"{{"kind":{},"position":[{:.6},{:.6},{:.6}],"direction":[{:.6},{:.6},{:.6}],"color":[{:.6},{:.6},{:.6}],"intensity":{:.6},"radius":{:.6}}}"#,
                    kind_str,
                    l.position[0], l.position[1], l.position[2],
                    l.direction[0], l.direction[1], l.direction[2],
                    l.color[0], l.color[1], l.color[2],
                    l.intensity, l.radius,
                )
            })
            .collect();

        format!(
            r#"{{"name":"{}","lights":[{}],"ambient":[{:.6},{:.6},{:.6}],"exposure":{:.6}}}"#,
            self.name,
            lights_json.join(","),
            self.ambient[0],
            self.ambient[1],
            self.ambient[2],
            self.exposure,
        )
    }
}

impl Light {
    /// Evaluate the light's contribution colour at a given surface point.
    ///
    /// Returns `(direction_to_light, attenuated_color)` where
    /// `direction_to_light` points from `surface_pos` toward the light source
    /// and `attenuated_color` is the light colour scaled by intensity and
    /// distance/angle attenuation.
    pub fn evaluate_at(&self, surface_pos: [f64; 3]) -> ([f64; 3], [f64; 3]) {
        match self.kind {
            LightKind::Directional => {
                let to_light = [-self.direction[0], -self.direction[1], -self.direction[2]];
                let col = [
                    self.color[0] * self.intensity,
                    self.color[1] * self.intensity,
                    self.color[2] * self.intensity,
                ];
                (normalize_f64(to_light), col)
            }
            LightKind::Point => {
                let delta = [
                    self.position[0] - surface_pos[0],
                    self.position[1] - surface_pos[1],
                    self.position[2] - surface_pos[2],
                ];
                let dist_sq = delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2];
                let dist = dist_sq.sqrt();
                if dist < 1e-12 {
                    return ([0.0, 1.0, 0.0], [0.0; 3]);
                }
                let to_light = normalize_f64(delta);
                // Inverse-square falloff with radius clamp
                let atten = if self.radius > 0.0 && dist > self.radius {
                    0.0
                } else {
                    self.intensity / (1.0 + dist_sq)
                };
                let col = [
                    self.color[0] * atten,
                    self.color[1] * atten,
                    self.color[2] * atten,
                ];
                (to_light, col)
            }
            LightKind::Spot {
                inner_angle,
                outer_angle,
            } => {
                let delta = [
                    self.position[0] - surface_pos[0],
                    self.position[1] - surface_pos[1],
                    self.position[2] - surface_pos[2],
                ];
                let dist_sq = delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2];
                let dist = dist_sq.sqrt();
                if dist < 1e-12 {
                    return ([0.0, 1.0, 0.0], [0.0; 3]);
                }
                let to_light = normalize_f64(delta);
                // Angle between light direction and vector from light to surface
                let neg_to_light = [-to_light[0], -to_light[1], -to_light[2]];
                let cos_angle = dot_f64(normalize_f64(self.direction), neg_to_light);
                let angle = cos_angle.clamp(-1.0, 1.0).acos();

                let spot_atten = if angle < inner_angle {
                    1.0
                } else if angle < outer_angle {
                    let t = (outer_angle - angle) / (outer_angle - inner_angle).max(1e-12);
                    t * t // smooth quadratic falloff
                } else {
                    0.0
                };

                let dist_atten = if self.radius > 0.0 && dist > self.radius {
                    0.0
                } else {
                    self.intensity / (1.0 + dist_sq)
                };

                let total = spot_atten * dist_atten;
                let col = [
                    self.color[0] * total,
                    self.color[1] * total,
                    self.color[2] * total,
                ];
                (to_light, col)
            }
        }
    }
}

fn dot_f64(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn studio_has_three_lights() {
        let preset = LightingPreset::studio();
        assert_eq!(preset.light_count(), 3);
    }

    #[test]
    fn outdoor_has_two_lights() {
        let preset = LightingPreset::outdoor();
        assert_eq!(preset.light_count(), 2);
    }

    #[test]
    fn indoor_has_three_point_lights() {
        let preset = LightingPreset::indoor();
        assert_eq!(preset.light_count(), 3);
        let by_kind = preset.lights_by_kind();
        assert_eq!(by_kind.get("point").map(|v| v.len()).unwrap_or(0), 3);
    }

    #[test]
    fn medical_has_six_directional_lights() {
        let preset = LightingPreset::medical();
        assert_eq!(preset.light_count(), 6);
        let by_kind = preset.lights_by_kind();
        assert_eq!(by_kind.get("directional").map(|v| v.len()).unwrap_or(0), 6);
    }

    #[test]
    fn dramatic_has_one_spot() {
        let preset = LightingPreset::dramatic();
        assert_eq!(preset.light_count(), 1);
        let by_kind = preset.lights_by_kind();
        assert_eq!(by_kind.get("spot").map(|v| v.len()).unwrap_or(0), 1);
    }

    #[test]
    fn rim_light_has_three_lights() {
        let preset = LightingPreset::rim_light();
        assert_eq!(preset.light_count(), 3);
    }

    #[test]
    fn custom_preset_accepts_empty_lights() {
        let preset = LightingPreset::custom(vec![], [0.1, 0.1, 0.1]);
        assert_eq!(preset.light_count(), 0);
        assert_eq!(preset.name, "Custom");
    }

    #[test]
    fn all_presets_returns_six() {
        let presets = LightingPreset::all_presets();
        assert_eq!(presets.len(), 6);
    }

    #[test]
    fn all_presets_have_positive_exposure() {
        for p in LightingPreset::all_presets() {
            assert!(
                p.exposure > 0.0,
                "preset '{}' must have positive exposure",
                p.name
            );
        }
    }

    #[test]
    fn all_presets_ambient_non_negative() {
        for p in LightingPreset::all_presets() {
            for ch in &p.ambient {
                assert!(*ch >= 0.0, "ambient channel must be >= 0 in '{}'", p.name);
            }
        }
    }

    #[test]
    fn total_intensity_positive_for_non_custom() {
        for p in LightingPreset::all_presets() {
            assert!(
                p.total_intensity() > 0.0,
                "total intensity must be > 0 for '{}'",
                p.name
            );
        }
    }

    #[test]
    fn to_json_contains_name() {
        let preset = LightingPreset::studio();
        let json = preset.to_json();
        assert!(json.contains("Studio"));
        assert!(json.contains("lights"));
        assert!(json.contains("ambient"));
        assert!(json.contains("exposure"));
    }

    #[test]
    fn directional_light_evaluate_at_origin() {
        let light = directional_light([0.0, -1.0, 0.0], [1.0, 1.0, 1.0], 2.0);
        let (to_light, col) = light.evaluate_at([0.0, 0.0, 0.0]);
        // Direction to light should be opposite of light direction => [0, 1, 0]
        assert!((to_light[1] - 1.0).abs() < 1e-6);
        // Colour should be intensity * color
        assert!((col[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn point_light_evaluate_distance_falloff() {
        let light = point_light([0.0, 5.0, 0.0], [1.0, 1.0, 1.0], 10.0, 100.0);
        let (_, col_near) = light.evaluate_at([0.0, 4.0, 0.0]);
        let (_, col_far) = light.evaluate_at([0.0, 0.0, 0.0]);
        // Closer point should receive more light
        assert!(
            col_near[0] > col_far[0],
            "near: {}, far: {}",
            col_near[0],
            col_far[0]
        );
    }

    #[test]
    fn point_light_outside_radius_is_zero() {
        let light = point_light([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 10.0, 2.0);
        let (_, col) = light.evaluate_at([0.0, 0.0, 100.0]);
        assert!(col[0].abs() < 1e-12, "should be zero outside radius");
    }

    #[test]
    fn spot_light_outside_cone_is_zero() {
        let light = spot_light(
            [0.0, 5.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, 1.0, 1.0],
            10.0,
            100.0,
            0.1,
            0.2,
        );
        // Point far to the side — well outside the cone
        let (_, col) = light.evaluate_at([100.0, 5.0, 0.0]);
        assert!(col[0] < 1e-6, "should be ~zero outside the spot cone");
    }

    #[test]
    fn spot_light_inside_inner_cone() {
        let light = spot_light(
            [0.0, 5.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, 1.0, 1.0],
            10.0,
            100.0,
            0.5,
            1.0,
        );
        // Point directly below the light
        let (_, col) = light.evaluate_at([0.0, 0.0, 0.0]);
        assert!(col[0] > 0.0, "should receive light inside the inner cone");
    }

    #[test]
    fn kelvin_6500_near_white() {
        let c = kelvin_to_rgb_f64(6500.0);
        assert!(c[0] > 0.9);
        assert!(c[1] > 0.9);
        assert!(c[2] > 0.9);
    }

    #[test]
    fn kelvin_2700_is_warm() {
        let c = kelvin_to_rgb_f64(2700.0);
        assert!(c[0] > c[2], "2700K should have more red than blue");
    }

    #[test]
    fn normalize_f64_unit_length() {
        let v = normalize_f64([3.0, 4.0, 0.0]);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-12);
    }

    #[test]
    fn normalize_f64_zero_vector_fallback() {
        let v = normalize_f64([0.0, 0.0, 0.0]);
        // Should return a valid unit vector (default [0,0,1])
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-12);
    }
}
