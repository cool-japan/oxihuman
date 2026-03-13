// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WorldBackground {
    pub color: [f32; 3],
    pub strength: f32,
    pub use_sky: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WorldSettings {
    pub background: WorldBackground,
    pub ambient_color: [f32; 3],
    pub mist_start: f32,
    pub mist_depth: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WorldSettingsExport {
    pub settings: WorldSettings,
}

#[allow(dead_code)]
pub fn default_world_settings() -> WorldSettings {
    WorldSettings {
        background: WorldBackground {
            color: [0.05, 0.05, 0.05],
            strength: 1.0,
            use_sky: false,
        },
        ambient_color: [0.0; 3],
        mist_start: 0.0,
        mist_depth: 25.0,
    }
}

#[allow(dead_code)]
pub fn new_world_settings_export() -> WorldSettingsExport {
    WorldSettingsExport { settings: default_world_settings() }
}

#[allow(dead_code)]
pub fn ws_set_background_color(exp: &mut WorldSettingsExport, color: [f32; 3]) {
    exp.settings.background.color = color;
}

#[allow(dead_code)]
pub fn ws_set_ambient(exp: &mut WorldSettingsExport, color: [f32; 3]) {
    exp.settings.ambient_color = color;
}

#[allow(dead_code)]
pub fn ws_set_mist(exp: &mut WorldSettingsExport, start: f32, depth: f32) {
    exp.settings.mist_start = start;
    exp.settings.mist_depth = depth;
}

#[allow(dead_code)]
pub fn ws_to_json(exp: &WorldSettingsExport) -> String {
    let bg = &exp.settings.background;
    format!(
        r#"{{"bg_strength":{},"mist_depth":{},"use_sky":{}}}"#,
        bg.strength,
        exp.settings.mist_depth,
        bg.use_sky
    )
}

#[allow(dead_code)]
pub fn ws_validate(exp: &WorldSettingsExport) -> bool {
    exp.settings.background.strength >= 0.0 && exp.settings.mist_depth >= 0.0
}

#[allow(dead_code)]
pub fn ws_background_luminance(exp: &WorldSettingsExport) -> f32 {
    let c = &exp.settings.background.color;
    let lum = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    lum * exp.settings.background.strength
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let s = default_world_settings();
        assert!((s.background.strength - 1.0).abs() < 1e-6);
        assert!(!s.background.use_sky);
    }

    #[test]
    fn test_new_export() {
        let e = new_world_settings_export();
        assert!((e.settings.mist_depth - 25.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_background_color() {
        let mut e = new_world_settings_export();
        ws_set_background_color(&mut e, [1.0, 0.0, 0.0]);
        assert!((e.settings.background.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_ambient() {
        let mut e = new_world_settings_export();
        ws_set_ambient(&mut e, [0.1, 0.1, 0.1]);
        assert!((e.settings.ambient_color[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_set_mist() {
        let mut e = new_world_settings_export();
        ws_set_mist(&mut e, 5.0, 50.0);
        assert!((e.settings.mist_start - 5.0).abs() < 1e-5);
        assert!((e.settings.mist_depth - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate() {
        let e = new_world_settings_export();
        assert!(ws_validate(&e));
    }

    #[test]
    fn test_luminance() {
        let mut e = new_world_settings_export();
        ws_set_background_color(&mut e, [1.0, 1.0, 1.0]);
        let lum = ws_background_luminance(&e);
        assert!(lum > 0.0);
    }

    #[test]
    fn test_to_json() {
        let e = new_world_settings_export();
        let j = ws_to_json(&e);
        assert!(j.contains("bg_strength"));
        assert!(j.contains("mist_depth"));
    }
}
