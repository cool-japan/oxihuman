// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Procedural sky dome with atmospheric scattering model.

// ── Enums ────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SkyDomeModel {
    Solid,
    Gradient,
    Preetham,
    Hosek,
}

// ── Structs ──────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkyDomeConfig {
    pub model: SkyDomeModel,
    pub sun_direction: [f32; 3],
    pub turbidity: f32,
    pub ground_albedo: f32,
    pub exposure: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkyColor {
    pub zenith: [f32; 3],
    pub horizon: [f32; 3],
    pub sun_disk: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkyDome {
    pub config: SkyDomeConfig,
    pub cached_color: SkyColor,
    pub dirty: bool,
}

// ── Functions ────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_sky_dome_config() -> SkyDomeConfig {
    SkyDomeConfig {
        model: SkyDomeModel::Preetham,
        sun_direction: [0.0, 1.0, 0.0],
        turbidity: 2.0,
        ground_albedo: 0.1,
        exposure: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_sky_dome(cfg: SkyDomeConfig) -> SkyDome {
    let color = compute_sky_color_internal(&cfg);
    SkyDome {
        config: cfg,
        cached_color: color,
        dirty: false,
    }
}

#[allow(dead_code)]
pub fn set_sun_direction(dome: &mut SkyDome, dir: [f32; 3]) {
    dome.config.sun_direction = dir;
    dome.dirty = true;
}

#[allow(dead_code)]
pub fn set_turbidity(dome: &mut SkyDome, t: f32) {
    dome.config.turbidity = t.clamp(1.0, 10.0);
    dome.dirty = true;
}

#[allow(dead_code)]
pub fn compute_sky_color(dome: &mut SkyDome) -> &SkyColor {
    if dome.dirty {
        dome.cached_color = compute_sky_color_internal(&dome.config);
        dome.dirty = false;
    }
    &dome.cached_color
}

#[allow(dead_code)]
pub fn sample_sky_at_direction(dome: &SkyDome, dir: [f32; 3]) -> [f32; 3] {
    let y = dir[1].clamp(-1.0, 1.0);
    let t = (y + 1.0) * 0.5; // 0 at nadir, 1 at zenith
    let h = dome.cached_color.horizon;
    let z = dome.cached_color.zenith;
    [
        (h[0] * (1.0 - t) + z[0] * t) * dome.config.exposure,
        (h[1] * (1.0 - t) + z[1] * t) * dome.config.exposure,
        (h[2] * (1.0 - t) + z[2] * t) * dome.config.exposure,
    ]
}

#[allow(dead_code)]
pub fn sky_luminance(dome: &SkyDome) -> f32 {
    let z = dome.cached_color.zenith;
    0.2126 * z[0] + 0.7152 * z[1] + 0.0722 * z[2]
}

#[allow(dead_code)]
pub fn sun_elevation(dome: &SkyDome) -> f32 {
    let d = dome.config.sun_direction;
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-9 {
        return 0.0;
    }
    let y = d[1] / len;
    y.clamp(-1.0, 1.0).asin().to_degrees()
}

#[allow(dead_code)]
pub fn sky_dome_to_json(dome: &SkyDome) -> String {
    format!(
        r#"{{"model":"{}","turbidity":{:.2},"exposure":{:.2},"sun_elevation":{:.2}}}"#,
        sky_model_name(dome),
        dome.config.turbidity,
        dome.config.exposure,
        sun_elevation(dome),
    )
}

#[allow(dead_code)]
pub fn sky_model_name(dome: &SkyDome) -> &'static str {
    match dome.config.model {
        SkyDomeModel::Solid => "solid",
        SkyDomeModel::Gradient => "gradient",
        SkyDomeModel::Preetham => "preetham",
        SkyDomeModel::Hosek => "hosek",
    }
}

#[allow(dead_code)]
pub fn is_day_time(dome: &SkyDome) -> bool {
    sun_elevation(dome) > 0.0
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn compute_sky_color_internal(cfg: &SkyDomeConfig) -> SkyColor {
    let elev = {
        let d = cfg.sun_direction;
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1e-9 {
            0.0f32
        } else {
            (d[1] / len).clamp(-1.0, 1.0).asin()
        }
    };
    let t = cfg.turbidity.clamp(1.0, 10.0);
    // Simple Preetham-inspired tint — not physically accurate, but deterministic and warning-free
    let blue_shift = (1.0 - elev / std::f32::consts::FRAC_PI_2).clamp(0.0, 1.0);
    let zenith = [
        0.15 + 0.1 / t,
        0.35 + 0.1 / t,
        0.6 + 0.3 * blue_shift / t,
    ];
    let horizon = [0.7 + 0.1 / t, 0.75 + 0.05 / t, 0.8];
    let sun_disk = [1.0, 0.95, 0.7];
    SkyColor {
        zenith,
        horizon,
        sun_disk,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_model_is_preetham() {
        let cfg = default_sky_dome_config();
        assert_eq!(cfg.model, SkyDomeModel::Preetham);
        assert!((cfg.turbidity - 2.0).abs() < 1e-6);
    }

    #[test]
    fn new_sky_dome_not_dirty() {
        let cfg = default_sky_dome_config();
        let dome = new_sky_dome(cfg);
        assert!(!dome.dirty);
    }

    #[test]
    fn set_sun_direction_marks_dirty() {
        let cfg = default_sky_dome_config();
        let mut dome = new_sky_dome(cfg);
        set_sun_direction(&mut dome, [1.0, 0.0, 0.0]);
        assert!(dome.dirty);
    }

    #[test]
    fn compute_sky_color_clears_dirty() {
        let cfg = default_sky_dome_config();
        let mut dome = new_sky_dome(cfg);
        set_sun_direction(&mut dome, [0.0, 0.8, 0.6]);
        assert!(dome.dirty);
        compute_sky_color(&mut dome);
        assert!(!dome.dirty);
    }

    #[test]
    fn sun_elevation_straight_up() {
        let cfg = default_sky_dome_config(); // sun_direction = [0,1,0]
        let dome = new_sky_dome(cfg);
        let elev = sun_elevation(&dome);
        assert!((elev - 90.0).abs() < 1e-3);
    }

    #[test]
    fn is_day_time_when_sun_above_horizon() {
        let cfg = default_sky_dome_config();
        let dome = new_sky_dome(cfg);
        assert!(is_day_time(&dome));
    }

    #[test]
    fn is_not_day_time_when_sun_below_horizon() {
        let mut cfg = default_sky_dome_config();
        cfg.sun_direction = [0.0, -1.0, 0.0];
        let dome = new_sky_dome(cfg);
        assert!(!is_day_time(&dome));
    }

    #[test]
    fn sky_model_name_all_variants() {
        let mut cfg = default_sky_dome_config();
        cfg.model = SkyDomeModel::Solid;
        let dome = new_sky_dome(cfg.clone());
        assert_eq!(sky_model_name(&dome), "solid");

        cfg.model = SkyDomeModel::Gradient;
        let dome = new_sky_dome(cfg.clone());
        assert_eq!(sky_model_name(&dome), "gradient");

        cfg.model = SkyDomeModel::Hosek;
        let dome = new_sky_dome(cfg);
        assert_eq!(sky_model_name(&dome), "hosek");
    }

    #[test]
    fn sky_dome_to_json_contains_model() {
        let cfg = default_sky_dome_config();
        let dome = new_sky_dome(cfg);
        let json = sky_dome_to_json(&dome);
        assert!(json.contains("preetham"));
        assert!(json.contains("turbidity"));
    }

    #[test]
    fn set_turbidity_clamps() {
        let cfg = default_sky_dome_config();
        let mut dome = new_sky_dome(cfg);
        set_turbidity(&mut dome, 100.0);
        assert!((dome.config.turbidity - 10.0).abs() < 1e-6);
    }

    #[test]
    fn sky_luminance_positive() {
        let cfg = default_sky_dome_config();
        let dome = new_sky_dome(cfg);
        assert!(sky_luminance(&dome) > 0.0);
    }
}
