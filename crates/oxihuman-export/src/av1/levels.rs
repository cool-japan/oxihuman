// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AV1 sequence level and tier definitions (AV1 spec Table A.2).

/// AV1 sequence level index values (Section 7.3 / Annex A).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeqLevel {
    L20 = 0,
    L21 = 1,
    L22 = 2,
    L23 = 3,
    L30 = 4,
    L31 = 5,
    L32 = 6,
    L33 = 7,
    L40 = 8,
    L41 = 9,
    L42 = 10,
    L43 = 11,
    L50 = 12,
    L51 = 13,
    L52 = 14,
    L53 = 15,
    L60 = 16,
    L61 = 17,
    L62 = 18,
    L63 = 19,
}

// ---- Tile & coding unit constants ----

/// Maximum width of a tile in luma samples.
pub const MAX_TILE_WIDTH: u32 = 4096;
/// Maximum area of a tile in luma samples.
pub const MAX_TILE_AREA: u32 = 4096 * 2304;
/// Maximum number of tile rows.
pub const MAX_TILE_ROWS: u32 = 64;
/// Maximum number of tile columns.
pub const MAX_TILE_COLS: u32 = 64;
/// Maximum number of tiles in a frame.
pub const MAX_TILE_COUNT: u32 = 512;
/// Number of reference frame slots.
pub const NUM_REF_FRAMES: usize = 8;
/// Number of references per frame.
pub const REFS_PER_FRAME: usize = 7;
/// Maximum superblock size in luma samples.
pub const MAX_SB_SIZE: u32 = 128;
/// Minimum coding unit / MI size in luma samples.
pub const MI_SIZE: u32 = 4;
/// Maximum number of segments.
pub const MAX_SEGMENTS: usize = 8;

// ---- Table A.2 data ----

/// Level parameters from AV1 spec Table A.2.
/// Each entry: (seq_level_idx, max_picture_size_luma_samples, max_display_rate, max_decode_rate,
///              max_header_rate, max_bitrate_main_tier, max_bitrate_high_tier, ...)
///
/// We store: (max_picture_size, max_bitrate_main_tier_bps).
/// max_bitrate values are in Mbps from the spec; we store as bps (u64).
struct LevelParams {
    max_picture_size: u64,  // luma samples
    max_bitrate_mbps: u64,  // main tier, Mbps
}

/// AV1 spec Table A.2 entries ordered by SeqLevel index 0..=19.
/// Entries that lack a high tier simply repeat main tier.
static LEVEL_PARAMS: [LevelParams; 20] = [
    // L20 (2.0): 147456 = 426×345 ≈ 0.14 MP
    LevelParams { max_picture_size: 147_456,    max_bitrate_mbps: 1_500 },
    // L21 (2.1)
    LevelParams { max_picture_size: 278_784,    max_bitrate_mbps: 3_000 },
    // L22 (2.2)
    LevelParams { max_picture_size: 278_784,    max_bitrate_mbps: 6_000 },
    // L23 (2.3)
    LevelParams { max_picture_size: 278_784,    max_bitrate_mbps: 10_000 },
    // L30 (3.0): 665_856 ≈ 0.66 MP  (≥ 1280×480)
    LevelParams { max_picture_size: 665_856,    max_bitrate_mbps: 12_000 },
    // L31 (3.1): 1_065_024
    LevelParams { max_picture_size: 1_065_024,  max_bitrate_mbps: 20_000 },
    // L32 (3.2)
    LevelParams { max_picture_size: 1_065_024,  max_bitrate_mbps: 30_000 },
    // L33 (3.3)
    LevelParams { max_picture_size: 1_065_024,  max_bitrate_mbps: 40_000 },
    // L40 (4.0): 2_359_296 = 1920×1224 approx
    LevelParams { max_picture_size: 2_359_296,  max_bitrate_mbps: 60_000 },
    // L41 (4.1): 2_359_296
    LevelParams { max_picture_size: 2_359_296,  max_bitrate_mbps: 100_000 },
    // L42 (4.2)
    LevelParams { max_picture_size: 2_359_296,  max_bitrate_mbps: 160_000 },
    // L43 (4.3)
    LevelParams { max_picture_size: 2_359_296,  max_bitrate_mbps: 240_000 },
    // L50 (5.0): 8_912_896 ≈ 4K
    LevelParams { max_picture_size: 8_912_896,  max_bitrate_mbps: 240_000 },
    // L51 (5.1): 8_912_896
    LevelParams { max_picture_size: 8_912_896,  max_bitrate_mbps: 480_000 },
    // L52 (5.2)
    LevelParams { max_picture_size: 8_912_896,  max_bitrate_mbps: 800_000 },
    // L53 (5.3)
    LevelParams { max_picture_size: 8_912_896,  max_bitrate_mbps: 1_600_000 },
    // L60 (6.0): 35_651_584 ≈ 8K
    LevelParams { max_picture_size: 35_651_584, max_bitrate_mbps: 1_600_000 },
    // L61 (6.1)
    LevelParams { max_picture_size: 35_651_584, max_bitrate_mbps: 4_800_000 },
    // L62 (6.2)
    LevelParams { max_picture_size: 35_651_584, max_bitrate_mbps: 4_800_000 },
    // L63 (6.3)
    LevelParams { max_picture_size: 35_651_584, max_bitrate_mbps: 4_800_000 },
];

/// Ordered list of all SeqLevel values from lowest to highest.
const ALL_LEVELS: [SeqLevel; 20] = [
    SeqLevel::L20,
    SeqLevel::L21,
    SeqLevel::L22,
    SeqLevel::L23,
    SeqLevel::L30,
    SeqLevel::L31,
    SeqLevel::L32,
    SeqLevel::L33,
    SeqLevel::L40,
    SeqLevel::L41,
    SeqLevel::L42,
    SeqLevel::L43,
    SeqLevel::L50,
    SeqLevel::L51,
    SeqLevel::L52,
    SeqLevel::L53,
    SeqLevel::L60,
    SeqLevel::L61,
    SeqLevel::L62,
    SeqLevel::L63,
];

impl SeqLevel {
    /// Maximum display resolution area in luma samples for this level.
    pub fn max_picture_size(&self) -> u64 {
        LEVEL_PARAMS[*self as usize].max_picture_size
    }

    /// Maximum bitrate in bits per second (main tier).
    pub fn max_bitrate(&self) -> u64 {
        // Spec gives Mbps; convert to bps (multiply by 1_000_000).
        // Note: some sources use 1_000_000 bps = 1 Mbps; others 1_048_576.
        // AV1 spec uses SI Mbps (10^6). We follow the spec.
        LEVEL_PARAMS[*self as usize].max_bitrate_mbps * 1_000_000
    }

    /// Return the minimum `SeqLevel` whose `max_picture_size()` accommodates a frame of
    /// `width × height` luma samples.  Falls back to `L63` if the frame exceeds all levels.
    pub fn for_frame_size(width: u32, height: u32) -> SeqLevel {
        let frame_area = (width as u64) * (height as u64);
        for level in &ALL_LEVELS {
            if level.max_picture_size() >= frame_area {
                return *level;
            }
        }
        SeqLevel::L63
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_for_small_frame() {
        // 426 × 240 = 102_240 < 147_456 (L20).
        let level = SeqLevel::for_frame_size(426, 240);
        assert_eq!(level, SeqLevel::L20);
    }

    #[test]
    fn test_level_for_1080p() {
        // 1920 × 1080 = 2_073_600 < 2_359_296 (L40).
        let level = SeqLevel::for_frame_size(1920, 1080);
        assert_eq!(level, SeqLevel::L40);
    }

    #[test]
    fn test_level_for_4k() {
        // 3840 × 2160 = 8_294_400 < 8_912_896 (L50).
        let level = SeqLevel::for_frame_size(3840, 2160);
        assert_eq!(level, SeqLevel::L50);
    }

    #[test]
    fn test_level_for_8k() {
        // 7680 × 4320 = 33_177_600 < 35_651_584 (L60).
        let level = SeqLevel::for_frame_size(7680, 4320);
        assert_eq!(level, SeqLevel::L60);
    }

    #[test]
    fn test_max_picture_size_monotonic() {
        let sizes: Vec<u64> = ALL_LEVELS.iter().map(|l| l.max_picture_size()).collect();
        // Each level's picture size must be >= the previous (non-decreasing by spec).
        for i in 1..sizes.len() {
            assert!(
                sizes[i] >= sizes[i - 1],
                "level sizes not monotone at index {i}: {} < {}",
                sizes[i],
                sizes[i - 1]
            );
        }
    }

    #[test]
    fn test_bitrate_positive() {
        for level in &ALL_LEVELS {
            assert!(level.max_bitrate() > 0, "bitrate should be positive for {:?}", level);
        }
    }

    #[test]
    fn test_l63_is_fallback_for_huge_frame() {
        // An absurdly large resolution beyond all levels.
        let level = SeqLevel::for_frame_size(100_000, 100_000);
        assert_eq!(level, SeqLevel::L63);
    }

    #[test]
    fn test_level_indices() {
        assert_eq!(SeqLevel::L20 as u8, 0);
        assert_eq!(SeqLevel::L63 as u8, 19);
        assert_eq!(SeqLevel::L50 as u8, 12);
    }
}
