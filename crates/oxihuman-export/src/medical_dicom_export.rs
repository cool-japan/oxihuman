// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// DICOM stub slice.
pub struct DicomSlice {
    pub rows: u16,
    pub cols: u16,
    pub pixel_spacing_mm: [f32; 2],
    pub slice_thickness_mm: f32,
    pub pixels: Vec<i16>,
}

pub fn new_dicom_slice(rows: u16, cols: u16) -> DicomSlice {
    DicomSlice {
        rows,
        cols,
        pixel_spacing_mm: [1.0, 1.0],
        slice_thickness_mm: 1.0,
        pixels: vec![0i16; rows as usize * cols as usize],
    }
}

pub fn dicom_set_pixel(s: &mut DicomSlice, r: u16, c: u16, v: i16) {
    if r < s.rows && c < s.cols {
        s.pixels[r as usize * s.cols as usize + c as usize] = v;
    }
}

pub fn dicom_get_pixel(s: &DicomSlice, r: u16, c: u16) -> i16 {
    if r < s.rows && c < s.cols {
        s.pixels[r as usize * s.cols as usize + c as usize]
    } else {
        0
    }
}

/// Apply window/level (Hounsfield unit to display 0-255).
pub fn dicom_hu_to_display(hu: i16, window_center: i16, window_width: i16) -> u8 {
    let half = window_width as f32 / 2.0;
    let center = window_center as f32;
    let v = (hu as f32 - center) / half.max(1.0);
    let v = (v + 1.0) * 0.5;
    (v.clamp(0.0, 1.0) * 255.0) as u8
}

pub fn dicom_pixel_count(s: &DicomSlice) -> usize {
    s.rows as usize * s.cols as usize
}

pub fn dicom_to_bytes(s: &DicomSlice) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.pixels.len() * 2);
    for &p in &s.pixels {
        out.extend_from_slice(&p.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dicom_slice_size() {
        let s = new_dicom_slice(256, 256);
        assert_eq!(s.pixels.len(), 65536);
    }

    #[test]
    fn test_dicom_set_get_pixel() {
        let mut s = new_dicom_slice(8, 8);
        dicom_set_pixel(&mut s, 2, 3, -500);
        assert_eq!(dicom_get_pixel(&s, 2, 3), -500);
    }

    #[test]
    fn test_dicom_get_pixel_oob() {
        let s = new_dicom_slice(8, 8);
        assert_eq!(dicom_get_pixel(&s, 100, 100), 0);
    }

    #[test]
    fn test_dicom_hu_to_display_center() {
        /* at window center, result should be ~128 */
        let v = dicom_hu_to_display(0, 0, 400);
        assert!((v as i32 - 128).abs() < 2);
    }

    #[test]
    fn test_dicom_hu_to_display_clamp_low() {
        let v = dicom_hu_to_display(-2000, 0, 400);
        assert_eq!(v, 0);
    }

    #[test]
    fn test_dicom_pixel_count() {
        let s = new_dicom_slice(4, 8);
        assert_eq!(dicom_pixel_count(&s), 32);
    }

    #[test]
    fn test_dicom_to_bytes_len() {
        let s = new_dicom_slice(4, 4);
        assert_eq!(dicom_to_bytes(&s).len(), 32);
    }
}
