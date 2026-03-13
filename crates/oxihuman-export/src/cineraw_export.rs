// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cinema DNG RAW stub export.

/// CinemaDNG bit depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CineDngBitDepth {
    Bits10,
    Bits12,
    Bits14,
    Bits16,
}

impl CineDngBitDepth {
    pub fn bits(&self) -> u32 {
        match self {
            CineDngBitDepth::Bits10 => 10,
            CineDngBitDepth::Bits12 => 12,
            CineDngBitDepth::Bits14 => 14,
            CineDngBitDepth::Bits16 => 16,
        }
    }
}

/// A CinemaDNG frame.
#[derive(Debug, Clone)]
pub struct CineDngFrame {
    pub frame_index: u32,
    pub width: u32,
    pub height: u32,
    pub iso: u32,
    pub shutter_angle: f32,
}

/// CinemaDNG export stub.
#[derive(Debug, Clone)]
pub struct CineRawExport {
    pub bit_depth: CineDngBitDepth,
    pub fps: f32,
    pub frames: Vec<CineDngFrame>,
    pub camera_make: String,
    pub camera_model: String,
}

/// Create a new CinemaDNG export.
pub fn new_cineraw_export(bit_depth: CineDngBitDepth, fps: f32) -> CineRawExport {
    CineRawExport {
        bit_depth,
        fps,
        frames: Vec::new(),
        camera_make: "Generic".to_string(),
        camera_model: "Model1".to_string(),
    }
}

/// Add a frame.
pub fn cineraw_add_frame(
    export: &mut CineRawExport,
    frame_index: u32,
    width: u32,
    height: u32,
    iso: u32,
    shutter_angle: f32,
) {
    export.frames.push(CineDngFrame {
        frame_index,
        width,
        height,
        iso,
        shutter_angle,
    });
}

/// Frame count.
pub fn cineraw_frame_count(export: &CineRawExport) -> usize {
    export.frames.len()
}

/// Duration in seconds.
pub fn cineraw_duration(export: &CineRawExport) -> f32 {
    if export.fps <= 0.0 {
        return 0.0;
    }
    export.frames.len() as f32 / export.fps
}

/// Validate the export.
pub fn validate_cineraw(export: &CineRawExport) -> bool {
    export.fps > 0.0
}

/// Estimate file size in bytes.
pub fn cineraw_size_estimate(export: &CineRawExport) -> usize {
    let bits = export.bit_depth.bits() as usize;
    export
        .frames
        .iter()
        .map(|f| {
            let pixels = f.width as usize * f.height as usize;
            pixels * bits.div_ceil(8)
        })
        .sum()
}

/// Generate a DNG EXIF-like metadata string.
pub fn cineraw_metadata_string(export: &CineRawExport) -> String {
    format!(
        "CinemaDNG|{}bit|FPS:{}|{}|{}|Frames:{}",
        export.bit_depth.bits(),
        export.fps,
        export.camera_make,
        export.camera_model,
        export.frames.len()
    )
}

/// Average shutter angle across frames.
pub fn cineraw_avg_shutter_angle(export: &CineRawExport) -> f32 {
    if export.frames.is_empty() {
        return 0.0;
    }
    export.frames.iter().map(|f| f.shutter_angle).sum::<f32>() / export.frames.len() as f32
}

/// Resolution of the first frame.
pub fn cineraw_resolution(export: &CineRawExport) -> Option<(u32, u32)> {
    export.frames.first().map(|f| (f.width, f.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CineRawExport {
        let mut exp = new_cineraw_export(CineDngBitDepth::Bits12, 24.0);
        cineraw_add_frame(&mut exp, 0, 4096, 3072, 800, 180.0);
        cineraw_add_frame(&mut exp, 1, 4096, 3072, 400, 90.0);
        exp
    }

    #[test]
    fn test_frame_count() {
        assert_eq!(cineraw_frame_count(&sample()), 2);
    }

    #[test]
    fn test_duration() {
        assert!((cineraw_duration(&sample()) - 2.0 / 24.0).abs() < 1e-4);
    }

    #[test]
    fn test_validate() {
        assert!(validate_cineraw(&sample()));
    }

    #[test]
    fn test_size_estimate() {
        assert!(cineraw_size_estimate(&sample()) > 0);
    }

    #[test]
    fn test_metadata() {
        assert!(cineraw_metadata_string(&sample()).contains("12bit"));
    }

    #[test]
    fn test_avg_shutter() {
        assert!((cineraw_avg_shutter_angle(&sample()) - 135.0).abs() < 1.0);
    }

    #[test]
    fn test_resolution() {
        assert_eq!(cineraw_resolution(&sample()), Some((4096, 3072)));
    }

    #[test]
    fn test_bit_depths() {
        assert_eq!(CineDngBitDepth::Bits10.bits(), 10);
        assert_eq!(CineDngBitDepth::Bits16.bits(), 16);
    }

    #[test]
    fn test_empty_frame_count() {
        let exp = new_cineraw_export(CineDngBitDepth::Bits14, 30.0);
        assert_eq!(cineraw_frame_count(&exp), 0);
    }
}
