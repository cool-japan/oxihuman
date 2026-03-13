// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! APNG animation stub export.

/// A single APNG frame.
#[derive(Debug, Clone)]
pub struct ApngFrame {
    pub width: u32,
    pub height: u32,
    pub delay_num: u16,
    pub delay_den: u16,
    pub pixels: Vec<[u8; 4]>,
}

impl ApngFrame {
    /// Create a filled APNG frame.
    pub fn new_solid(
        width: u32,
        height: u32,
        delay_num: u16,
        delay_den: u16,
        color: [u8; 4],
    ) -> Self {
        let pixels = vec![color; (width * height) as usize];
        Self {
            width,
            height,
            delay_num,
            delay_den,
            pixels,
        }
    }

    /// Frame delay in seconds.
    pub fn delay_secs(&self) -> f32 {
        if self.delay_den == 0 {
            return 0.0;
        }
        self.delay_num as f32 / self.delay_den as f32
    }

    /// Pixel count.
    pub fn pixel_count(&self) -> usize {
        self.pixels.len()
    }
}

/// APNG export stub.
#[derive(Debug, Clone)]
pub struct ApngExport {
    pub width: u32,
    pub height: u32,
    pub loop_count: u32,
    pub frames: Vec<ApngFrame>,
}

impl ApngExport {
    /// Create a new APNG export.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            loop_count: 0,
            frames: Vec::new(),
        }
    }

    /// Add a frame.
    pub fn add_frame(&mut self, frame: ApngFrame) {
        self.frames.push(frame);
    }

    /// Return frame count.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Total animation duration in seconds.
    pub fn total_duration_secs(&self) -> f32 {
        self.frames.iter().map(|f| f.delay_secs()).sum()
    }
}

/// Validate that all frames match animation dimensions.
pub fn validate_apng(apng: &ApngExport) -> bool {
    apng.frames
        .iter()
        .all(|f| f.width == apng.width && f.height == apng.height)
}

/// Estimate raw uncompressed size.
pub fn estimate_raw_bytes(apng: &ApngExport) -> usize {
    apng.frames
        .iter()
        .map(|f| f.pixel_count() * 4)
        .sum::<usize>()
        + 8
}

/// Serialize metadata to JSON (stub).
pub fn apng_metadata_json(apng: &ApngExport) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"frames\":{},\"loop\":{}}}",
        apng.width,
        apng.height,
        apng.frame_count(),
        apng.loop_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_apng() -> ApngExport {
        let mut apng = ApngExport::new(32, 32);
        apng.add_frame(ApngFrame::new_solid(32, 32, 1, 24, [255, 0, 0, 255]));
        apng.add_frame(ApngFrame::new_solid(32, 32, 1, 24, [0, 0, 255, 255]));
        apng
    }

    #[test]
    fn test_frame_count() {
        /* frame count is correct */
        assert_eq!(sample_apng().frame_count(), 2);
    }

    #[test]
    fn test_delay_secs() {
        /* delay in seconds computed from num/den */
        let f = ApngFrame::new_solid(4, 4, 1, 10, [0; 4]);
        assert!((f.delay_secs() - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_total_duration_secs() {
        /* total duration sums frame delays */
        let apng = sample_apng();
        let expected = 2.0 / 24.0;
        assert!((apng.total_duration_secs() - expected).abs() < 1e-5);
    }

    #[test]
    fn test_validate_apng_valid() {
        /* matching frames pass validation */
        assert!(validate_apng(&sample_apng()));
    }

    #[test]
    fn test_validate_apng_invalid() {
        /* mismatched frames fail validation */
        let mut apng = ApngExport::new(32, 32);
        apng.add_frame(ApngFrame::new_solid(16, 16, 1, 10, [0; 4]));
        assert!(!validate_apng(&apng));
    }

    #[test]
    fn test_estimate_raw_bytes_positive() {
        /* raw byte estimate is positive */
        assert!(estimate_raw_bytes(&sample_apng()) > 0);
    }

    #[test]
    fn test_metadata_json() {
        /* metadata JSON contains expected keys */
        let json = apng_metadata_json(&sample_apng());
        assert!(json.contains("32"));
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count matches dimensions */
        let f = ApngFrame::new_solid(8, 8, 1, 10, [0; 4]);
        assert_eq!(f.pixel_count(), 64);
    }
}
