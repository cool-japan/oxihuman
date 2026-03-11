// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! RED RAW R3D stub export.

/// R3D codec variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R3dCodec {
    Redcode28,
    Redcode36,
    Redcode42,
}

impl R3dCodec {
    pub fn compression_ratio(&self) -> u32 {
        match self {
            R3dCodec::Redcode28 => 28,
            R3dCodec::Redcode36 => 36,
            R3dCodec::Redcode42 => 42,
        }
    }
}

/// A RED RAW R3D stub frame.
#[derive(Debug, Clone)]
pub struct R3dFrame {
    pub frame_index: u32,
    pub width: u32,
    pub height: u32,
    pub iso: u32,
    pub kelvin: u32,
}

/// R3D export stub.
#[derive(Debug, Clone)]
pub struct R3dExport {
    pub codec: R3dCodec,
    pub fps: f32,
    pub frames: Vec<R3dFrame>,
    pub camera_serial: String,
}

/// Create a new R3D export.
pub fn new_r3d_export(codec: R3dCodec, fps: f32) -> R3dExport {
    R3dExport {
        codec,
        fps,
        frames: Vec::new(),
        camera_serial: "RED-000001".to_string(),
    }
}

/// Add a frame to the export.
pub fn r3d_add_frame(
    export: &mut R3dExport,
    frame_index: u32,
    width: u32,
    height: u32,
    iso: u32,
    kelvin: u32,
) {
    export.frames.push(R3dFrame {
        frame_index,
        width,
        height,
        iso,
        kelvin,
    });
}

/// Frame count.
pub fn r3d_frame_count(export: &R3dExport) -> usize {
    export.frames.len()
}

/// Duration in seconds.
pub fn r3d_duration_seconds(export: &R3dExport) -> f32 {
    if export.fps <= 0.0 {
        return 0.0;
    }
    export.frames.len() as f32 / export.fps
}

/// Estimate the R3D file size in bytes (stub).
pub fn r3d_size_estimate(export: &R3dExport) -> usize {
    export
        .frames
        .iter()
        .map(|f| {
            let pixels = f.width as usize * f.height as usize;
            /* 16-bit RAW / compression ratio */
            pixels * 2 / export.codec.compression_ratio() as usize
        })
        .sum()
}

/// Validate the export.
pub fn validate_r3d(export: &R3dExport) -> bool {
    export.fps > 0.0 && !export.frames.is_empty()
}

/// Generate a stub R3D metadata string.
pub fn r3d_metadata_string(export: &R3dExport) -> String {
    format!(
        "R3D|Codec:REDCODE{}|FPS:{}|Frames:{}|Serial:{}",
        export.codec.compression_ratio(),
        export.fps,
        export.frames.len(),
        export.camera_serial
    )
}

/// Return the resolution of the first frame.
pub fn r3d_resolution(export: &R3dExport) -> Option<(u32, u32)> {
    export.frames.first().map(|f| (f.width, f.height))
}

/// Average ISO across frames.
pub fn r3d_average_iso(export: &R3dExport) -> f32 {
    if export.frames.is_empty() {
        return 0.0;
    }
    export.frames.iter().map(|f| f.iso as f32).sum::<f32>() / export.frames.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> R3dExport {
        let mut exp = new_r3d_export(R3dCodec::Redcode28, 24.0);
        r3d_add_frame(&mut exp, 0, 8192, 4320, 800, 5600);
        r3d_add_frame(&mut exp, 1, 8192, 4320, 1600, 5600);
        exp
    }

    #[test]
    fn test_frame_count() {
        assert_eq!(r3d_frame_count(&sample()), 2);
    }

    #[test]
    fn test_duration() {
        let exp = sample();
        assert!((r3d_duration_seconds(&exp) - 2.0 / 24.0).abs() < 1e-4);
    }

    #[test]
    fn test_size_estimate() {
        assert!(r3d_size_estimate(&sample()) > 0);
    }

    #[test]
    fn test_validate_valid() {
        assert!(validate_r3d(&sample()));
    }

    #[test]
    fn test_validate_no_frames() {
        let exp = new_r3d_export(R3dCodec::Redcode36, 24.0);
        assert!(!validate_r3d(&exp));
    }

    #[test]
    fn test_metadata_string() {
        let s = r3d_metadata_string(&sample());
        assert!(s.contains("28"));
    }

    #[test]
    fn test_resolution() {
        let exp = sample();
        assert_eq!(r3d_resolution(&exp), Some((8192, 4320)));
    }

    #[test]
    fn test_average_iso() {
        let exp = sample();
        assert!((r3d_average_iso(&exp) - 1200.0).abs() < 1e-3);
    }

    #[test]
    fn test_codec_ratios() {
        assert_eq!(R3dCodec::Redcode28.compression_ratio(), 28);
        assert_eq!(R3dCodec::Redcode42.compression_ratio(), 42);
    }
}
