// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Light-map atlas management (baked GI atlas).

/// Texel encoding.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightMapEncoding {
    Linear,
    Rgbm,
    LogLuv,
}

/// Atlas entry for one mesh.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LightMapEntry {
    pub mesh_id: u32,
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
    pub atlas_page: u32,
}

/// Light-map atlas.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LightMapAtlas {
    pub page_size: u32,
    pub page_count: u32,
    pub encoding: LightMapEncoding,
    pub entries: Vec<LightMapEntry>,
}

impl Default for LightMapAtlas {
    fn default() -> Self {
        Self {
            page_size: 1024,
            page_count: 1,
            encoding: LightMapEncoding::Linear,
            entries: Vec::new(),
        }
    }
}

#[allow(dead_code)]
pub fn new_lightmap_atlas() -> LightMapAtlas {
    LightMapAtlas::default()
}

#[allow(dead_code)]
pub fn lm_add_entry(
    atlas: &mut LightMapAtlas,
    mesh_id: u32,
    uv_offset: [f32; 2],
    uv_scale: [f32; 2],
    page: u32,
) {
    atlas.entries.push(LightMapEntry {
        mesh_id,
        uv_offset,
        uv_scale,
        atlas_page: page,
    });
}

#[allow(dead_code)]
pub fn lm_remove_entry(atlas: &mut LightMapAtlas, mesh_id: u32) {
    atlas.entries.retain(|e| e.mesh_id != mesh_id);
}

#[allow(dead_code)]
pub fn lm_entry_count(atlas: &LightMapAtlas) -> usize {
    atlas.entries.len()
}

#[allow(dead_code)]
pub fn lm_get_entry(atlas: &LightMapAtlas, mesh_id: u32) -> Option<&LightMapEntry> {
    atlas.entries.iter().find(|e| e.mesh_id == mesh_id)
}

#[allow(dead_code)]
pub fn lm_encoding_name(enc: LightMapEncoding) -> &'static str {
    match enc {
        LightMapEncoding::Linear => "linear",
        LightMapEncoding::Rgbm => "rgbm",
        LightMapEncoding::LogLuv => "logluv",
    }
}

#[allow(dead_code)]
pub fn lm_memory_bytes(atlas: &LightMapAtlas) -> u64 {
    let bytes_per_texel: u64 = match atlas.encoding {
        LightMapEncoding::Linear => 8,
        LightMapEncoding::Rgbm => 4,
        LightMapEncoding::LogLuv => 4,
    };
    atlas.page_count as u64 * atlas.page_size as u64 * atlas.page_size as u64 * bytes_per_texel
}

#[allow(dead_code)]
pub fn lm_clear(atlas: &mut LightMapAtlas) {
    atlas.entries.clear();
}

#[allow(dead_code)]
pub fn lm_to_json(atlas: &LightMapAtlas) -> String {
    format!(
        "{{\"page_size\":{},\"pages\":{},\"encoding\":\"{}\",\"entries\":{}}}",
        atlas.page_size,
        atlas.page_count,
        lm_encoding_name(atlas.encoding),
        atlas.entries.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty_entries() {
        assert_eq!(lm_entry_count(&new_lightmap_atlas()), 0);
    }

    #[test]
    fn add_entry() {
        let mut a = new_lightmap_atlas();
        lm_add_entry(&mut a, 1, [0.0, 0.0], [0.5, 0.5], 0);
        assert_eq!(lm_entry_count(&a), 1);
    }

    #[test]
    fn get_entry() {
        let mut a = new_lightmap_atlas();
        lm_add_entry(&mut a, 42, [0.25, 0.25], [0.25, 0.25], 0);
        assert!(lm_get_entry(&a, 42).is_some());
    }

    #[test]
    fn remove_entry() {
        let mut a = new_lightmap_atlas();
        lm_add_entry(&mut a, 1, [0.0, 0.0], [1.0, 1.0], 0);
        lm_remove_entry(&mut a, 1);
        assert_eq!(lm_entry_count(&a), 0);
    }

    #[test]
    fn encoding_name_rgbm() {
        assert_eq!(lm_encoding_name(LightMapEncoding::Rgbm), "rgbm");
    }

    #[test]
    fn memory_bytes_positive() {
        assert!(lm_memory_bytes(&new_lightmap_atlas()) > 0);
    }

    #[test]
    fn clear_empty() {
        let mut a = new_lightmap_atlas();
        lm_add_entry(&mut a, 1, [0.0, 0.0], [1.0, 1.0], 0);
        lm_clear(&mut a);
        assert_eq!(lm_entry_count(&a), 0);
    }

    #[test]
    fn json_has_page_size() {
        assert!(lm_to_json(&new_lightmap_atlas()).contains("page_size"));
    }

    #[test]
    fn get_missing_is_none() {
        assert!(lm_get_entry(&new_lightmap_atlas(), 99).is_none());
    }

    #[test]
    fn uv_scale_stored() {
        let mut a = new_lightmap_atlas();
        lm_add_entry(&mut a, 5, [0.1, 0.2], [0.3, 0.4], 0);
        let e = lm_get_entry(&a, 5).expect("should succeed");
        assert!((e.uv_scale[0] - 0.3).abs() < 1e-5);
    }
}
