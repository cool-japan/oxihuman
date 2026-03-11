// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! RGB-D frame export (color + depth aligned).

/// A single RGB-D frame.
#[allow(dead_code)]
pub struct RgbdFrame {
    pub width: u32,
    pub height: u32,
    pub color: Vec<[u8; 4]>,
    pub depth: Vec<f32>,
    pub timestamp_us: u64,
}

/// RGB-D export sequence.
#[allow(dead_code)]
pub struct RgbdExport {
    pub frames: Vec<RgbdFrame>,
    pub camera_fx: f32,
    pub camera_fy: f32,
    pub camera_cx: f32,
    pub camera_cy: f32,
}

/// Create a new RGB-D export.
#[allow(dead_code)]
pub fn new_rgbd_export(fx: f32, fy: f32, cx: f32, cy: f32) -> RgbdExport {
    RgbdExport { frames: Vec::new(), camera_fx: fx, camera_fy: fy, camera_cx: cx, camera_cy: cy }
}

/// Create a new frame.
#[allow(dead_code)]
pub fn new_rgbd_frame(width: u32, height: u32, timestamp_us: u64) -> RgbdFrame {
    let n = (width * height) as usize;
    RgbdFrame {
        width,
        height,
        color: vec![[0u8, 0, 0, 255]; n],
        depth: vec![0.0f32; n],
        timestamp_us,
    }
}

/// Add a frame to the export.
#[allow(dead_code)]
pub fn add_rgbd_frame(export: &mut RgbdExport, frame: RgbdFrame) {
    export.frames.push(frame);
}

/// Frame count.
#[allow(dead_code)]
pub fn rgbd_frame_count(export: &RgbdExport) -> usize {
    export.frames.len()
}

/// Set a pixel in a frame.
#[allow(dead_code)]
pub fn set_rgbd_pixel(frame: &mut RgbdFrame, x: u32, y: u32, rgba: [u8; 4], depth: f32) {
    let idx = (y * frame.width + x) as usize;
    if idx < frame.color.len() {
        frame.color[idx] = rgba;
        frame.depth[idx] = depth;
    }
}

/// Compute average depth in a frame.
#[allow(dead_code)]
pub fn rgbd_avg_depth(frame: &RgbdFrame) -> f32 {
    if frame.depth.is_empty() { return 0.0; }
    frame.depth.iter().sum::<f32>() / frame.depth.len() as f32
}

/// Backproject a pixel to 3D point.
#[allow(dead_code)]
pub fn backproject_pixel(frame: &RgbdFrame, export: &RgbdExport, x: u32, y: u32) -> [f32; 3] {
    let idx = (y * frame.width + x) as usize;
    let d = if idx < frame.depth.len() { frame.depth[idx] } else { 0.0 };
    let px = (x as f32 - export.camera_cx) * d / export.camera_fx;
    let py = (y as f32 - export.camera_cy) * d / export.camera_fy;
    [px, py, d]
}

/// Validate frame dimensions.
#[allow(dead_code)]
pub fn validate_rgbd_frame(frame: &RgbdFrame) -> bool {
    let n = (frame.width * frame.height) as usize;
    frame.color.len() == n && frame.depth.len() == n
}

/// Estimate export size.
#[allow(dead_code)]
pub fn rgbd_export_size_bytes(export: &RgbdExport) -> usize {
    export.frames.iter().map(|f| (f.width * f.height) as usize * 8).sum()
}

/// Total pixel count.
#[allow(dead_code)]
pub fn rgbd_total_pixels(export: &RgbdExport) -> usize {
    export.frames.iter().map(|f| (f.width * f.height) as usize).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let e = new_rgbd_export(525.0, 525.0, 320.0, 240.0);
        assert_eq!(rgbd_frame_count(&e), 0);
    }

    #[test]
    fn add_frame_count() {
        let mut e = new_rgbd_export(525.0, 525.0, 320.0, 240.0);
        let f = new_rgbd_frame(8, 6, 1000);
        add_rgbd_frame(&mut e, f);
        assert_eq!(rgbd_frame_count(&e), 1);
    }

    #[test]
    fn frame_pixel_count() {
        let f = new_rgbd_frame(8, 6, 0);
        assert_eq!(f.color.len(), 48);
        assert_eq!(f.depth.len(), 48);
    }

    #[test]
    fn set_pixel_works() {
        let mut f = new_rgbd_frame(4, 4, 0);
        set_rgbd_pixel(&mut f, 1, 0, [255, 0, 0, 255], 1.5);
        assert_eq!(f.color[1], [255, 0, 0, 255]);
        assert!((f.depth[1] - 1.5).abs() < 1e-5);
    }

    #[test]
    fn avg_depth_zero_init() {
        let f = new_rgbd_frame(4, 4, 0);
        assert_eq!(rgbd_avg_depth(&f), 0.0);
    }

    #[test]
    fn backproject_z_equals_depth() {
        let mut e = new_rgbd_export(500.0, 500.0, 0.0, 0.0);
        let mut f = new_rgbd_frame(4, 4, 0);
        set_rgbd_pixel(&mut f, 0, 0, [255,255,255,255], 3.0);
        add_rgbd_frame(&mut e, f);
        let p = backproject_pixel(&e.frames[0], &e, 0, 0);
        assert!((p[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn validate_frame() {
        let f = new_rgbd_frame(4, 4, 0);
        assert!(validate_rgbd_frame(&f));
    }

    #[test]
    fn total_pixels() {
        let mut e = new_rgbd_export(500.0, 500.0, 0.0, 0.0);
        add_rgbd_frame(&mut e, new_rgbd_frame(4, 4, 0));
        add_rgbd_frame(&mut e, new_rgbd_frame(4, 4, 1));
        assert_eq!(rgbd_total_pixels(&e), 32);
    }

    #[test]
    fn export_size_bytes() {
        let mut e = new_rgbd_export(500.0, 500.0, 0.0, 0.0);
        add_rgbd_frame(&mut e, new_rgbd_frame(4, 4, 0));
        let sz = rgbd_export_size_bytes(&e);
        assert!(sz > 0);
    }
}
