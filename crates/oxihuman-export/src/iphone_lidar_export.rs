// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! iPhone LiDAR scan export (R3D metadata stub).

/// iPhone LiDAR frame metadata.
#[allow(dead_code)]
pub struct IphoneLidarFrame {
    pub timestamp_ns: u64,
    pub depth_width: u32,
    pub depth_height: u32,
    pub confidence_map: Vec<u8>,
    pub depth_map: Vec<f32>,
}

/// iPhone LiDAR export.
#[allow(dead_code)]
pub struct IphoneLidarExport {
    pub frames: Vec<IphoneLidarFrame>,
    pub device_model: String,
    pub capture_date: String,
}

/// Create a new iPhone LiDAR export.
#[allow(dead_code)]
pub fn new_iphone_lidar_export(device_model: &str) -> IphoneLidarExport {
    IphoneLidarExport {
        frames: Vec::new(),
        device_model: device_model.to_string(),
        capture_date: "2026-01-01".to_string(),
    }
}

/// Create a new LiDAR frame.
#[allow(dead_code)]
pub fn new_lidar_frame(width: u32, height: u32, timestamp_ns: u64) -> IphoneLidarFrame {
    let n = (width * height) as usize;
    IphoneLidarFrame {
        timestamp_ns,
        depth_width: width,
        depth_height: height,
        confidence_map: vec![0u8; n],
        depth_map: vec![0.0f32; n],
    }
}

/// Add a frame.
#[allow(dead_code)]
pub fn add_lidar_frame(export: &mut IphoneLidarExport, frame: IphoneLidarFrame) {
    export.frames.push(frame);
}

/// Frame count.
#[allow(dead_code)]
pub fn lidar_frame_count(export: &IphoneLidarExport) -> usize {
    export.frames.len()
}

/// Total pixel count across all frames.
#[allow(dead_code)]
pub fn total_lidar_pixels(export: &IphoneLidarExport) -> usize {
    export.frames.iter().map(|f| (f.depth_width * f.depth_height) as usize).sum()
}

/// Average depth in a frame.
#[allow(dead_code)]
pub fn avg_depth(frame: &IphoneLidarFrame) -> f32 {
    if frame.depth_map.is_empty() { return 0.0; }
    frame.depth_map.iter().sum::<f32>() / frame.depth_map.len() as f32
}

/// Set depth value at pixel.
#[allow(dead_code)]
pub fn set_depth_pixel(frame: &mut IphoneLidarFrame, x: u32, y: u32, depth: f32) {
    let idx = (y * frame.depth_width + x) as usize;
    if idx < frame.depth_map.len() {
        frame.depth_map[idx] = depth;
    }
}

/// Build R3D metadata JSON stub.
#[allow(dead_code)]
pub fn build_r3d_metadata_json(export: &IphoneLidarExport) -> String {
    format!(
        "{{\"device\":\"{}\",\"date\":\"{}\",\"frames\":{}}}",
        export.device_model, export.capture_date, export.frames.len()
    )
}

/// Validate.
#[allow(dead_code)]
pub fn validate_iphone_lidar(export: &IphoneLidarExport) -> bool {
    !export.frames.is_empty() && !export.device_model.is_empty()
}

/// Estimate export size in bytes.
#[allow(dead_code)]
pub fn lidar_export_size_bytes(export: &IphoneLidarExport) -> usize {
    export.frames.iter().map(|f| (f.depth_width * f.depth_height) as usize * 5).sum::<usize>() + 256
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_iphone_lidar_export("iPhone14Pro");
        assert_eq!(lidar_frame_count(&e), 0);
    }

    #[test]
    fn add_frame_count() {
        let mut e = new_iphone_lidar_export("iPhone14Pro");
        let f = new_lidar_frame(32, 24, 1000000);
        add_lidar_frame(&mut e, f);
        assert_eq!(lidar_frame_count(&e), 1);
    }

    #[test]
    fn frame_pixel_count() {
        let f = new_lidar_frame(32, 24, 0);
        assert_eq!(f.depth_map.len(), 768);
    }

    #[test]
    fn avg_depth_zero() {
        let f = new_lidar_frame(4, 4, 0);
        assert_eq!(avg_depth(&f), 0.0);
    }

    #[test]
    fn set_depth_pixel_works() {
        let mut f = new_lidar_frame(4, 4, 0);
        set_depth_pixel(&mut f, 1, 0, 2.5);
        assert!((f.depth_map[1] - 2.5).abs() < 1e-5);
    }

    #[test]
    fn total_pixels_sum() {
        let mut e = new_iphone_lidar_export("test");
        let f1 = new_lidar_frame(4, 4, 0);
        let f2 = new_lidar_frame(4, 4, 1);
        add_lidar_frame(&mut e, f1);
        add_lidar_frame(&mut e, f2);
        assert_eq!(total_lidar_pixels(&e), 32);
    }

    #[test]
    fn r3d_json_contains_device() {
        let e = new_iphone_lidar_export("iPhone15Ultra");
        let j = build_r3d_metadata_json(&e);
        assert!(j.contains("iPhone15Ultra"));
    }

    #[test]
    fn validate_fails_empty() {
        let e = new_iphone_lidar_export("test");
        assert!(!validate_iphone_lidar(&e));
    }

    #[test]
    fn validate_passes() {
        let mut e = new_iphone_lidar_export("test");
        let f = new_lidar_frame(8, 6, 0);
        add_lidar_frame(&mut e, f);
        assert!(validate_iphone_lidar(&e));
    }
}
