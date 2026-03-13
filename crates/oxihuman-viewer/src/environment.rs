// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

// ── Enumerations ──────────────────────────────────────────────────────────────

/// Sky rendering model.
pub enum SkyModel {
    Procedural,
    Hdri,
    SolidColor,
    Gradient,
}

// ── Descriptors ───────────────────────────────────────────────────────────────

/// Physical atmosphere parameters for procedural sky.
pub struct AtmosphereParams {
    /// Normalized sun direction vector (points toward sun).
    pub sun_direction: [f32; 3],
    /// Sun disc intensity multiplier.
    pub sun_intensity: f32,
    /// Atmospheric turbidity / haze (1.0 = crystal clear, 10.0 = very hazy).
    pub turbidity: f32,
    /// Ground reflectance [0..1].
    pub ground_albedo: f32,
    /// Mie scattering coefficient.
    pub mie_coefficient: f32,
}

/// Two-stop sky gradient.
pub struct SkyGradient {
    pub zenith_color: [f32; 3],
    pub horizon_color: [f32; 3],
    pub nadir_color: [f32; 3],
}

/// Full environment descriptor controlling sky, fog, and exposure.
pub struct EnvironmentDescriptor {
    pub sky_model: SkyModel,
    pub atmosphere: Option<AtmosphereParams>,
    pub gradient: Option<SkyGradient>,
    pub hdri_path: Option<String>,
    pub solid_color: [f32; 3],
    pub exposure: f32,
    pub rotation_deg: f32,
    pub fog_density: f32,
    pub fog_color: [f32; 3],
}

// ── Preset constructors ───────────────────────────────────────────────────────

impl EnvironmentDescriptor {
    /// Outdoor daylight with procedural sky and standard sun.
    pub fn default_outdoor() -> Self {
        Self {
            sky_model: SkyModel::Procedural,
            atmosphere: Some(AtmosphereParams {
                sun_direction: [0.3, 0.8, 0.5],
                sun_intensity: 5.0,
                turbidity: 2.5,
                ground_albedo: 0.25,
                mie_coefficient: 0.005,
            }),
            gradient: None,
            hdri_path: None,
            solid_color: [0.53, 0.81, 0.98],
            exposure: 1.0,
            rotation_deg: 0.0,
            fog_density: 0.005,
            fog_color: [0.85, 0.90, 0.95],
        }
    }

    /// Studio environment: neutral gray background, no atmosphere.
    pub fn default_studio() -> Self {
        Self {
            sky_model: SkyModel::SolidColor,
            atmosphere: None,
            gradient: None,
            hdri_path: None,
            solid_color: [0.5, 0.5, 0.5],
            exposure: 1.0,
            rotation_deg: 0.0,
            fog_density: 0.0,
            fog_color: [0.5, 0.5, 0.5],
        }
    }

    /// Night sky: dark blue gradient, minimal exposure.
    pub fn default_night() -> Self {
        Self {
            sky_model: SkyModel::Gradient,
            atmosphere: None,
            gradient: Some(SkyGradient {
                zenith_color: [0.02, 0.02, 0.08],
                horizon_color: [0.05, 0.05, 0.12],
                nadir_color: [0.01, 0.01, 0.03],
            }),
            hdri_path: None,
            solid_color: [0.01, 0.01, 0.03],
            exposure: 0.05,
            rotation_deg: 0.0,
            fog_density: 0.002,
            fog_color: [0.02, 0.02, 0.05],
        }
    }

    /// Sunset: warm gradient, sun near the horizon.
    pub fn default_sunset() -> Self {
        Self {
            sky_model: SkyModel::Gradient,
            atmosphere: Some(AtmosphereParams {
                sun_direction: [0.95, 0.08, 0.30],
                sun_intensity: 3.0,
                turbidity: 5.0,
                ground_albedo: 0.15,
                mie_coefficient: 0.012,
            }),
            gradient: Some(SkyGradient {
                zenith_color: [0.20, 0.15, 0.35],
                horizon_color: [1.00, 0.45, 0.15],
                nadir_color: [0.10, 0.08, 0.05],
            }),
            hdri_path: None,
            solid_color: [1.0, 0.45, 0.15],
            exposure: 0.8,
            rotation_deg: 0.0,
            fog_density: 0.015,
            fog_color: [0.95, 0.60, 0.30],
        }
    }
}

// ── Sky sampling ──────────────────────────────────────────────────────────────

/// Sample the sky color at a given elevation angle (0 = horizon, 90 = zenith).
pub fn sky_color_at_angle(env: &EnvironmentDescriptor, elevation_deg: f32) -> [f32; 3] {
    let t = (elevation_deg / 90.0).clamp(0.0, 1.0);

    match &env.sky_model {
        SkyModel::SolidColor => env.solid_color,

        SkyModel::Procedural => {
            // Simplified Rayleigh: sky is bluer near zenith, paler near horizon
            let blue_zenith = [0.15, 0.40, 0.90];
            let white_horizon = [0.85, 0.88, 0.92];
            lerp_color(white_horizon, blue_zenith, t.powf(0.5))
        }

        SkyModel::Gradient => {
            if let Some(grad) = &env.gradient {
                if elevation_deg < 0.0 {
                    grad.nadir_color
                } else {
                    lerp_color(grad.horizon_color, grad.zenith_color, t)
                }
            } else {
                env.solid_color
            }
        }

        SkyModel::Hdri => {
            // Without actual HDRI data, return a neutral color
            [0.5, 0.5, 0.5]
        }
    }
}

// ── Fog ───────────────────────────────────────────────────────────────────────

/// Exponential fog factor: exp(-density * distance). Returns 1.0 at d=0.
pub fn fog_factor(distance: f32, env: &EnvironmentDescriptor) -> f32 {
    (-env.fog_density * distance).exp()
}

// ── Sun direction ─────────────────────────────────────────────────────────────

/// Compute a sun direction vector from azimuth and elevation angles (degrees).
/// Returns a unit vector pointing toward the sun. Elevation 90° = straight up.
pub fn sun_direction_from_angles(azimuth_deg: f32, elevation_deg: f32) -> [f32; 3] {
    let az = azimuth_deg.to_radians();
    let el = elevation_deg.to_radians();
    let cos_el = el.cos();
    let x = cos_el * az.sin();
    let y = el.sin();
    let z = cos_el * az.cos();
    // Normalize (already unit length by construction, but guard for precision)
    let len = (x * x + y * y + z * z).sqrt();
    if len < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [x / len, y / len, z / len]
    }
}

// ── JSON serialization ────────────────────────────────────────────────────────

/// Serialize an environment descriptor to a JSON string.
pub fn environment_to_json(env: &EnvironmentDescriptor) -> String {
    let sky_model_str = match &env.sky_model {
        SkyModel::Procedural => "Procedural",
        SkyModel::Hdri => "Hdri",
        SkyModel::SolidColor => "SolidColor",
        SkyModel::Gradient => "Gradient",
    };

    let has_atmosphere = env.atmosphere.is_some();
    let has_gradient = env.gradient.is_some();
    let hdri = env
        .hdri_path
        .as_deref()
        .map(|p| format!("\"{}\"", p))
        .unwrap_or_else(|| "null".to_string());

    let sc = env.solid_color;
    let fc = env.fog_color;

    let atm_json = if let Some(atm) = &env.atmosphere {
        let sd = atm.sun_direction;
        format!(
            r#"{{ "sun_direction": [{:.4}, {:.4}, {:.4}], "sun_intensity": {:.4}, "turbidity": {:.4}, "ground_albedo": {:.4}, "mie_coefficient": {:.6} }}"#,
            sd[0],
            sd[1],
            sd[2],
            atm.sun_intensity,
            atm.turbidity,
            atm.ground_albedo,
            atm.mie_coefficient,
        )
    } else {
        "null".to_string()
    };

    let grad_json = if let Some(g) = &env.gradient {
        let zc = g.zenith_color;
        let hc = g.horizon_color;
        let nc = g.nadir_color;
        format!(
            r#"{{ "zenith": [{:.4}, {:.4}, {:.4}], "horizon": [{:.4}, {:.4}, {:.4}], "nadir": [{:.4}, {:.4}, {:.4}] }}"#,
            zc[0], zc[1], zc[2], hc[0], hc[1], hc[2], nc[0], nc[1], nc[2],
        )
    } else {
        "null".to_string()
    };

    format!(
        r#"{{
  "sky_model": "{sky_model}",
  "has_atmosphere": {has_atm},
  "has_gradient": {has_grad},
  "atmosphere": {atm},
  "gradient": {grad},
  "hdri_path": {hdri},
  "solid_color": [{sc0:.4}, {sc1:.4}, {sc2:.4}],
  "exposure": {exposure:.4},
  "rotation_deg": {rot:.4},
  "fog_density": {fog_d:.6},
  "fog_color": [{fc0:.4}, {fc1:.4}, {fc2:.4}]
}}"#,
        sky_model = sky_model_str,
        has_atm = has_atmosphere,
        has_grad = has_gradient,
        atm = atm_json,
        grad = grad_json,
        hdri = hdri,
        sc0 = sc[0],
        sc1 = sc[1],
        sc2 = sc[2],
        exposure = env.exposure,
        rot = env.rotation_deg,
        fog_d = env.fog_density,
        fc0 = fc[0],
        fc1 = fc[1],
        fc2 = fc[2],
    )
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[inline]
fn lerp_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_outdoor_is_procedural() {
        let env = EnvironmentDescriptor::default_outdoor();
        assert!(matches!(env.sky_model, SkyModel::Procedural));
    }

    #[test]
    fn default_studio_solid_color_gray() {
        let env = EnvironmentDescriptor::default_studio();
        assert!(matches!(env.sky_model, SkyModel::SolidColor));
        let sc = env.solid_color;
        // All channels equal and ~0.5
        assert!((sc[0] - sc[1]).abs() < 1e-6);
        assert!((sc[1] - sc[2]).abs() < 1e-6);
        assert!((sc[0] - 0.5).abs() < 1e-3);
    }

    #[test]
    fn default_night_low_exposure() {
        let env = EnvironmentDescriptor::default_night();
        assert!(env.exposure < 0.2, "night exposure should be low");
    }

    #[test]
    fn default_night_has_gradient() {
        let env = EnvironmentDescriptor::default_night();
        assert!(env.gradient.is_some());
    }

    #[test]
    fn default_sunset_warm_gradient() {
        let env = EnvironmentDescriptor::default_sunset();
        if let Some(g) = &env.gradient {
            // Horizon should be warm (high red, low blue)
            assert!(g.horizon_color[0] > 0.5, "horizon should have high red");
            assert!(g.horizon_color[2] < 0.5, "horizon should have low blue");
        } else {
            panic!("sunset should have a gradient");
        }
    }

    #[test]
    fn fog_factor_at_zero_is_one() {
        let env = EnvironmentDescriptor::default_outdoor();
        let f = fog_factor(0.0, &env);
        assert!((f - 1.0).abs() < 1e-6, "fog at d=0 must be 1.0");
    }

    #[test]
    fn fog_factor_decreases_with_distance() {
        let env = EnvironmentDescriptor::default_outdoor();
        let f0 = fog_factor(10.0, &env);
        let f1 = fog_factor(100.0, &env);
        assert!(f1 < f0, "fog should decrease with distance");
    }

    #[test]
    fn fog_factor_zero_density_is_always_one() {
        let env = EnvironmentDescriptor::default_studio(); // fog_density = 0.0
        assert!((fog_factor(1000.0, &env) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sun_direction_elevation_90_is_up() {
        let d = sun_direction_from_angles(0.0, 90.0);
        assert!((d[1] - 1.0).abs() < 1e-5, "elevation=90 must point up (Y)");
        assert!(d[0].abs() < 1e-5);
        assert!(d[2].abs() < 1e-5);
    }

    #[test]
    fn sun_direction_elevation_0_is_horizon() {
        let d = sun_direction_from_angles(0.0, 0.0);
        assert!(d[1].abs() < 1e-5, "elevation=0 must be on horizon (Y≈0)");
    }

    #[test]
    fn sun_direction_is_unit_length() {
        let d = sun_direction_from_angles(45.0, 30.0);
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 1e-5,
            "sun direction must be unit length"
        );
    }

    #[test]
    fn sky_color_at_angle_zenith_vs_horizon_differ_procedural() {
        let env = EnvironmentDescriptor::default_outdoor();
        let zenith = sky_color_at_angle(&env, 90.0);
        let horizon = sky_color_at_angle(&env, 0.0);
        let diff = (zenith[0] - horizon[0]).abs()
            + (zenith[1] - horizon[1]).abs()
            + (zenith[2] - horizon[2]).abs();
        assert!(
            diff > 0.01,
            "zenith and horizon colors should differ for procedural sky"
        );
    }

    #[test]
    fn sky_color_solid_always_same() {
        let env = EnvironmentDescriptor::default_studio();
        let c0 = sky_color_at_angle(&env, 0.0);
        let c1 = sky_color_at_angle(&env, 90.0);
        assert_eq!(
            c0, c1,
            "solid color sky should return same color at any angle"
        );
    }

    #[test]
    fn environment_to_json_non_empty() {
        let env = EnvironmentDescriptor::default_outdoor();
        let json = environment_to_json(&env);
        assert!(!json.is_empty());
        assert!(json.contains("sky_model"));
    }

    #[test]
    fn environment_to_json_all_presets() {
        for env in [
            EnvironmentDescriptor::default_outdoor(),
            EnvironmentDescriptor::default_studio(),
            EnvironmentDescriptor::default_night(),
            EnvironmentDescriptor::default_sunset(),
        ] {
            let json = environment_to_json(&env);
            assert!(json.contains("exposure"), "JSON must include exposure");
            assert!(
                json.contains("fog_density"),
                "JSON must include fog_density"
            );
        }
    }

    #[test]
    fn all_presets_have_valid_exposure() {
        let presets = [
            EnvironmentDescriptor::default_outdoor(),
            EnvironmentDescriptor::default_studio(),
            EnvironmentDescriptor::default_night(),
            EnvironmentDescriptor::default_sunset(),
        ];
        for env in &presets {
            assert!(env.exposure > 0.0, "exposure must be positive");
            assert!(env.exposure <= 10.0, "exposure should be reasonable");
        }
    }
}
