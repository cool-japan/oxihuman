// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sprite atlas UV layout — packs sprite frames into a texture atlas grid.

/// A single sprite frame entry in the atlas.
#[derive(Debug, Clone, Copy)]
pub struct SpriteFrame {
    pub id: u32,
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

/// Configuration for a sprite atlas layout.
#[derive(Debug, Clone)]
pub struct SpriteAtlasConfig {
    pub columns: u32,
    pub rows: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
}

impl Default for SpriteAtlasConfig {
    fn default() -> Self {
        Self {
            columns: 4,
            rows: 4,
            atlas_width: 1024,
            atlas_height: 1024,
        }
    }
}

/// A packed sprite atlas with UV frames.
#[derive(Debug, Default, Clone)]
pub struct SpriteAtlas {
    pub frames: Vec<SpriteFrame>,
    pub config: SpriteAtlasConfig,
}

impl SpriteAtlas {
    /// Builds an atlas layout for the given config.
    pub fn build(config: SpriteAtlasConfig) -> Self {
        let mut frames = Vec::new();
        let col_size = 1.0 / config.columns as f32;
        let row_size = 1.0 / config.rows as f32;
        let total = config.columns * config.rows;
        for i in 0..total {
            let col = i % config.columns;
            let row = i / config.columns;
            frames.push(SpriteFrame {
                id: i,
                u_min: col as f32 * col_size,
                v_min: row as f32 * row_size,
                u_max: (col + 1) as f32 * col_size,
                v_max: (row + 1) as f32 * row_size,
            });
        }
        Self { frames, config }
    }

    /// Returns the frame for a given sprite id, if it exists.
    pub fn frame(&self, id: u32) -> Option<&SpriteFrame> {
        self.frames.iter().find(|f| f.id == id)
    }

    /// Returns the total number of frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

/// Computes the pixel size of a single cell in the atlas.
pub fn cell_pixel_size(config: &SpriteAtlasConfig) -> (u32, u32) {
    (
        config.atlas_width / config.columns.max(1),
        config.atlas_height / config.rows.max(1),
    )
}

/// Returns the UV center of a frame.
pub fn frame_uv_center(frame: &SpriteFrame) -> [f32; 2] {
    [
        (frame.u_min + frame.u_max) * 0.5,
        (frame.v_min + frame.v_max) * 0.5,
    ]
}

/// Validates that all frame UVs are within [0, 1].
pub fn validate_atlas_uvs(atlas: &SpriteAtlas) -> bool {
    atlas.frames.iter().all(|f| {
        (0.0..=1.0).contains(&f.u_min)
            && (0.0..=1.0).contains(&f.u_max)
            && (0.0..=1.0).contains(&f.v_min)
            && (0.0..=1.0).contains(&f.v_max)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_atlas() -> SpriteAtlas {
        SpriteAtlas::build(SpriteAtlasConfig::default())
    }

    #[test]
    fn test_frame_count() {
        /* 4x4 config should produce 16 frames */
        assert_eq!(default_atlas().frame_count(), 16);
    }

    #[test]
    fn test_frame_zero_exists() {
        /* Frame 0 should be present */
        assert!(default_atlas().frame(0).is_some());
    }

    #[test]
    fn test_frame_out_of_range() {
        /* Frame 99 should not exist in a 4x4 atlas */
        assert!(default_atlas().frame(99).is_none());
    }

    #[test]
    fn test_uvs_in_range() {
        /* All UVs must be within [0,1] */
        assert!(validate_atlas_uvs(&default_atlas()));
    }

    #[test]
    fn test_cell_pixel_size() {
        /* 1024 / 4 = 256 */
        let (w, h) = cell_pixel_size(&SpriteAtlasConfig::default());
        assert_eq!(w, 256);
        assert_eq!(h, 256);
    }

    #[test]
    fn test_frame_uv_center_first() {
        /* First frame center should be at (col_size/2, row_size/2) */
        let atlas = default_atlas();
        let frame = atlas.frame(0).unwrap();
        let [cu, cv] = frame_uv_center(frame);
        assert!((cu - 0.125).abs() < 0.001);
        assert!((cv - 0.125).abs() < 0.001);
    }

    #[test]
    fn test_single_cell_atlas() {
        /* 1x1 atlas should produce one full-UV frame */
        let cfg = SpriteAtlasConfig {
            columns: 1,
            rows: 1,
            atlas_width: 256,
            atlas_height: 256,
        };
        let atlas = SpriteAtlas::build(cfg);
        let f = atlas.frame(0).unwrap();
        assert_eq!(f.u_max, 1.0);
        assert_eq!(f.v_max, 1.0);
    }

    #[test]
    fn test_frame_ids_sequential() {
        /* Frame ids should match their sequential index */
        let atlas = default_atlas();
        for (i, f) in atlas.frames.iter().enumerate() {
            assert_eq!(f.id, i as u32);
        }
    }

    #[test]
    fn test_u_min_less_than_u_max() {
        /* u_min must be less than u_max for every frame */
        for f in default_atlas().frames.iter() {
            assert!(f.u_min < f.u_max);
        }
    }

    #[test]
    fn test_default_config() {
        /* Default config should be 4x4 at 1024x1024 */
        let cfg = SpriteAtlasConfig::default();
        assert_eq!(cfg.columns, 4);
    }
}
