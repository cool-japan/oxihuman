// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! GPU readback helpers: staging buffer management and pixel-format conversion.

/// Format of the readback data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ReadbackFormat {
    Rgba8Unorm,
    Rgba16Float,
    R32Float,
    Depth32Float,
}

/// A pending readback request.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReadbackRequest {
    pub format: ReadbackFormat,
    pub width: u32,
    pub height: u32,
    /// Byte offset into the staging buffer.
    pub offset: u64,
}

/// A completed readback with raw byte payload.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReadbackResult {
    pub format: ReadbackFormat,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Return the bytes-per-pixel for a `ReadbackFormat`.
#[allow(dead_code)]
pub fn bytes_per_pixel(fmt: ReadbackFormat) -> usize {
    match fmt {
        ReadbackFormat::Rgba8Unorm => 4,
        ReadbackFormat::Rgba16Float => 8,
        ReadbackFormat::R32Float => 4,
        ReadbackFormat::Depth32Float => 4,
    }
}

/// Compute the required staging buffer size in bytes.
#[allow(dead_code)]
pub fn staging_buffer_size(req: &ReadbackRequest) -> u64 {
    let bpp = bytes_per_pixel(req.format) as u64;
    bpp * req.width as u64 * req.height as u64
}

/// Create a blank (zeroed) `ReadbackResult`.
#[allow(dead_code)]
pub fn blank_result(fmt: ReadbackFormat, width: u32, height: u32) -> ReadbackResult {
    let size = bytes_per_pixel(fmt) * width as usize * height as usize;
    ReadbackResult {
        format: fmt,
        width,
        height,
        data: vec![0u8; size],
    }
}

/// Convert an Rgba8Unorm pixel at index `i` to linear [f32; 4].
#[allow(dead_code)]
pub fn rgba8_to_f32(data: &[u8], pixel_index: usize) -> [f32; 4] {
    let base = pixel_index * 4;
    [
        data[base] as f32 / 255.0,
        data[base + 1] as f32 / 255.0,
        data[base + 2] as f32 / 255.0,
        data[base + 3] as f32 / 255.0,
    ]
}

/// Read an R32Float value at `pixel_index`.
#[allow(dead_code)]
pub fn r32_to_f32(data: &[u8], pixel_index: usize) -> f32 {
    let base = pixel_index * 4;
    let bytes: [u8; 4] = data[base..base + 4].try_into().unwrap_or([0; 4]);
    f32::from_le_bytes(bytes)
}

/// Validate that a `ReadbackResult` data length matches width × height × bpp.
#[allow(dead_code)]
pub fn validate_result(res: &ReadbackResult) -> bool {
    let expected = bytes_per_pixel(res.format) * res.width as usize * res.height as usize;
    res.data.len() == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_per_pixel_rgba8() {
        assert_eq!(bytes_per_pixel(ReadbackFormat::Rgba8Unorm), 4);
    }

    #[test]
    fn bytes_per_pixel_rgba16() {
        assert_eq!(bytes_per_pixel(ReadbackFormat::Rgba16Float), 8);
    }

    #[test]
    fn staging_buffer_size_correct() {
        let req = ReadbackRequest {
            format: ReadbackFormat::Rgba8Unorm,
            width: 4,
            height: 4,
            offset: 0,
        };
        assert_eq!(staging_buffer_size(&req), 64);
    }

    #[test]
    fn blank_result_correct_size() {
        let r = blank_result(ReadbackFormat::Rgba8Unorm, 2, 2);
        assert!(validate_result(&r));
    }

    #[test]
    fn rgba8_to_f32_white() {
        let data = vec![255u8; 4];
        let px = rgba8_to_f32(&data, 0);
        assert!((px[0] - 1.0).abs() < 1e-3);
    }

    #[test]
    fn rgba8_to_f32_black() {
        let data = vec![0u8; 8];
        let px = rgba8_to_f32(&data, 0);
        assert_eq!(px[0], 0.0);
    }

    #[test]
    fn r32_to_f32_roundtrip() {
        let val = std::f32::consts::PI;
        let bytes = val.to_le_bytes();
        let data: Vec<u8> = bytes.to_vec();
        assert!((r32_to_f32(&data, 0) - val).abs() < 1e-6);
    }

    #[test]
    fn validate_result_correct() {
        let r = blank_result(ReadbackFormat::R32Float, 3, 3);
        assert!(validate_result(&r));
    }

    #[test]
    fn validate_result_wrong_size() {
        let mut r = blank_result(ReadbackFormat::Rgba8Unorm, 2, 2);
        r.data.pop();
        assert!(!validate_result(&r));
    }
}
