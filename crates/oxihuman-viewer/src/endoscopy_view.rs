// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct EndoscopyFrame {
    pub width: u32,
    pub height: u32,
    pub pixels_rgb: Vec<[u8; 3]>,
    pub frame_index: u32,
    pub timestamp_ms: u64,
}

pub fn new_endoscopy_frame(w: u32, h: u32, idx: u32) -> EndoscopyFrame {
    let n = (w as usize) * (h as usize);
    EndoscopyFrame {
        width: w,
        height: h,
        pixels_rgb: vec![[0, 0, 0]; n],
        frame_index: idx,
        timestamp_ms: idx as u64 * 33,
    }
}

pub fn endo_set_pixel(f: &mut EndoscopyFrame, x: u32, y: u32, rgb: [u8; 3]) {
    if x < f.width && y < f.height {
        f.pixels_rgb[y as usize * f.width as usize + x as usize] = rgb;
    }
}

pub fn endo_get_pixel(f: &EndoscopyFrame, x: u32, y: u32) -> [u8; 3] {
    if x < f.width && y < f.height {
        f.pixels_rgb[y as usize * f.width as usize + x as usize]
    } else {
        [0, 0, 0]
    }
}

pub fn endo_mean_brightness(f: &EndoscopyFrame) -> f32 {
    if f.pixels_rgb.is_empty() {
        return 0.0;
    }
    let sum: u64 = f
        .pixels_rgb
        .iter()
        .map(|p| (p[0] as u64 + p[1] as u64 + p[2] as u64) / 3)
        .sum();
    sum as f32 / f.pixels_rgb.len() as f32
}

pub fn endo_pixel_count(f: &EndoscopyFrame) -> usize {
    f.pixels_rgb.len()
}

pub fn endo_duration_ms(frames: &[EndoscopyFrame]) -> u64 {
    match (frames.first(), frames.last()) {
        (Some(first), Some(last)) => last.timestamp_ms.saturating_sub(first.timestamp_ms),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_frame() {
        /* frame_index is set */
        let f = new_endoscopy_frame(64, 64, 5);
        assert_eq!(f.frame_index, 5);
    }

    #[test]
    fn test_pixel_count() {
        /* pixel count = w*h */
        let f = new_endoscopy_frame(8, 6, 0);
        assert_eq!(endo_pixel_count(&f), 48);
    }

    #[test]
    fn test_set_get_pixel() {
        /* set and get */
        let mut f = new_endoscopy_frame(10, 10, 0);
        endo_set_pixel(&mut f, 5, 5, [200, 100, 50]);
        assert_eq!(endo_get_pixel(&f, 5, 5), [200, 100, 50]);
    }

    #[test]
    fn test_get_pixel_oob() {
        /* out of bounds => black */
        let f = new_endoscopy_frame(5, 5, 0);
        assert_eq!(endo_get_pixel(&f, 100, 100), [0, 0, 0]);
    }

    #[test]
    fn test_mean_brightness_black() {
        /* all black => 0 */
        let f = new_endoscopy_frame(4, 4, 0);
        assert!((endo_mean_brightness(&f) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_duration_ms() {
        /* duration = last - first timestamp */
        let f0 = new_endoscopy_frame(4, 4, 0);
        let f1 = new_endoscopy_frame(4, 4, 10);
        let dur = endo_duration_ms(&[f0, f1]);
        assert_eq!(dur, 10 * 33);
    }

    #[test]
    fn test_duration_empty() {
        /* empty slice => 0 */
        assert_eq!(endo_duration_ms(&[]), 0);
    }
}
