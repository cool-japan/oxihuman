// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Decal UV — projection and atlas UV computation for decal rendering.

/// A decal UV transform in atlas space.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalUvTransform {
    pub offset: [f32; 2],
    pub scale: [f32; 2],
    pub rotation_rad: f32,
}

/// Decal UV configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalUvConfig {
    pub atlas_size: u32,
    pub tile_size: u32,
    pub padding: u32,
}

#[allow(dead_code)]
pub fn default_decal_uv_config() -> DecalUvConfig {
    DecalUvConfig {
        atlas_size: 1024,
        tile_size: 128,
        padding: 2,
    }
}

#[allow(dead_code)]
pub fn default_decal_uv_transform() -> DecalUvTransform {
    DecalUvTransform {
        offset: [0.0, 0.0],
        scale: [1.0, 1.0],
        rotation_rad: 0.0,
    }
}

#[allow(dead_code)]
pub fn duv_tiles_per_row(cfg: &DecalUvConfig) -> u32 {
    cfg.atlas_size / (cfg.tile_size + cfg.padding)
}

#[allow(dead_code)]
pub fn duv_tile_uv_offset(cfg: &DecalUvConfig, tile_index: u32) -> [f32; 2] {
    let per_row = duv_tiles_per_row(cfg);
    if per_row == 0 {
        return [0.0, 0.0];
    }
    let row = tile_index / per_row;
    let col = tile_index % per_row;
    let step = (cfg.tile_size + cfg.padding) as f32 / cfg.atlas_size as f32;
    [col as f32 * step, row as f32 * step]
}

#[allow(dead_code)]
pub fn duv_tile_uv_scale(cfg: &DecalUvConfig) -> [f32; 2] {
    let s = cfg.tile_size as f32 / cfg.atlas_size as f32;
    [s, s]
}

#[allow(dead_code)]
pub fn duv_project_point(t: &DecalUvTransform, p: [f32; 2]) -> [f32; 2] {
    let cos_r = t.rotation_rad.cos();
    let sin_r = t.rotation_rad.sin();
    let rx = p[0] * cos_r - p[1] * sin_r;
    let ry = p[0] * sin_r + p[1] * cos_r;
    [rx * t.scale[0] + t.offset[0], ry * t.scale[1] + t.offset[1]]
}

#[allow(dead_code)]
pub fn duv_is_in_bounds(uv: [f32; 2]) -> bool {
    (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1])
}

#[allow(dead_code)]
pub fn duv_set_rotation(t: &mut DecalUvTransform, rad: f32) {
    use std::f32::consts::TAU;
    t.rotation_rad = rad % TAU;
}

#[allow(dead_code)]
pub fn duv_to_json(t: &DecalUvTransform) -> String {
    format!(
        r#"{{"offset":[{:.4},{:.4}],"scale":[{:.4},{:.4}],"rotation_rad":{:.4}}}"#,
        t.offset[0], t.offset[1], t.scale[0], t.scale[1], t.rotation_rad
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_decal_uv_config();
        assert_eq!(cfg.atlas_size, 1024);
    }

    #[test]
    fn tiles_per_row() {
        let cfg = default_decal_uv_config();
        let n = duv_tiles_per_row(&cfg);
        assert!(n > 0);
    }

    #[test]
    fn tile_offset_zero_for_first() {
        let cfg = default_decal_uv_config();
        let off = duv_tile_uv_offset(&cfg, 0);
        assert!(off[0].abs() < 1e-6 && off[1].abs() < 1e-6);
    }

    #[test]
    fn tile_scale_positive() {
        let cfg = default_decal_uv_config();
        let s = duv_tile_uv_scale(&cfg);
        assert!(s[0] > 0.0 && s[1] > 0.0);
    }

    #[test]
    fn project_identity() {
        let t = default_decal_uv_transform();
        let p = duv_project_point(&t, [0.5, 0.3]);
        assert!((p[0] - 0.5).abs() < 1e-6);
        assert!((p[1] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn in_bounds_center() {
        assert!(duv_is_in_bounds([0.5, 0.5]));
    }

    #[test]
    fn out_of_bounds() {
        assert!(!duv_is_in_bounds([1.5, 0.5]));
    }

    #[test]
    fn set_rotation() {
        let mut t = default_decal_uv_transform();
        duv_set_rotation(&mut t, 1.0);
        assert!((t.rotation_rad - 1.0).abs() < 1e-6);
    }

    #[test]
    fn to_json_fields() {
        let t = default_decal_uv_transform();
        let j = duv_to_json(&t);
        assert!(j.contains("offset"));
        assert!(j.contains("rotation_rad"));
    }
}
