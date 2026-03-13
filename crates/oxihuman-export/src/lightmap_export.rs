// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Lightmap UV channel and pixel data export.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightmapConfig {
    pub resolution: u32,
    pub channel: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightmapExport {
    pub config: LightmapConfig,
    pub uv_coords: Vec<[f32; 2]>,
    pub pixel_data: Vec<u8>,
}

#[allow(dead_code)]
pub fn default_lightmap_config() -> LightmapConfig {
    LightmapConfig { resolution: 512, channel: 1 }
}

#[allow(dead_code)]
pub fn new_lightmap_export(config: LightmapConfig) -> LightmapExport {
    LightmapExport {
        config,
        uv_coords: Vec::new(),
        pixel_data: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn lm_set_uv(export: &mut LightmapExport, index: usize, uv: [f32; 2]) {
    if index >= export.uv_coords.len() {
        export.uv_coords.resize(index + 1, [0.0, 0.0]);
    }
    export.uv_coords[index] = uv;
}

#[allow(dead_code)]
pub fn lm_get_uv(export: &LightmapExport, index: usize) -> Option<[f32; 2]> {
    export.uv_coords.get(index).copied()
}

#[allow(dead_code)]
pub fn lm_uv_count(export: &LightmapExport) -> usize {
    export.uv_coords.len()
}

#[allow(dead_code)]
pub fn lm_set_pixel(export: &mut LightmapExport, index: usize, value: u8) {
    if index >= export.pixel_data.len() {
        export.pixel_data.resize(index + 1, 0);
    }
    export.pixel_data[index] = value;
}

#[allow(dead_code)]
pub fn lm_pixel_count(export: &LightmapExport) -> usize {
    export.pixel_data.len()
}

#[allow(dead_code)]
pub fn lm_validate(export: &LightmapExport) -> bool {
    export.config.resolution > 0
}

#[allow(dead_code)]
pub fn lm_to_json(export: &LightmapExport) -> String {
    format!(
        "{{\"resolution\":{},\"channel\":{},\"uv_count\":{},\"pixel_count\":{}}}",
        export.config.resolution,
        export.config.channel,
        export.uv_coords.len(),
        export.pixel_data.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lightmap_config();
        assert_eq!(cfg.resolution, 512);
    }

    #[test]
    fn test_new_export() {
        let exp = new_lightmap_export(default_lightmap_config());
        assert_eq!(lm_uv_count(&exp), 0);
    }

    #[test]
    fn test_set_get_uv() {
        let mut exp = new_lightmap_export(default_lightmap_config());
        lm_set_uv(&mut exp, 0, [0.5, 0.25]);
        assert_eq!(lm_get_uv(&exp, 0), Some([0.5, 0.25]));
    }

    #[test]
    fn test_uv_count() {
        let mut exp = new_lightmap_export(default_lightmap_config());
        lm_set_uv(&mut exp, 0, [0.0, 0.0]);
        lm_set_uv(&mut exp, 1, [1.0, 1.0]);
        assert_eq!(lm_uv_count(&exp), 2);
    }

    #[test]
    fn test_set_pixel() {
        let mut exp = new_lightmap_export(default_lightmap_config());
        lm_set_pixel(&mut exp, 0, 128);
        assert_eq!(exp.pixel_data[0], 128);
    }

    #[test]
    fn test_pixel_count() {
        let mut exp = new_lightmap_export(default_lightmap_config());
        lm_set_pixel(&mut exp, 3, 255);
        assert_eq!(lm_pixel_count(&exp), 4);
    }

    #[test]
    fn test_validate() {
        let exp = new_lightmap_export(default_lightmap_config());
        assert!(lm_validate(&exp));
        let bad = new_lightmap_export(LightmapConfig { resolution: 0, channel: 0 });
        assert!(!lm_validate(&bad));
    }

    #[test]
    fn test_to_json() {
        let exp = new_lightmap_export(default_lightmap_config());
        let j = lm_to_json(&exp);
        assert!(j.contains("resolution"));
    }
}
