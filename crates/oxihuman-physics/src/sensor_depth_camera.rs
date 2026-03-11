// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Depth camera point cloud sensor stub.

/// Depth camera configuration.
#[derive(Debug, Clone)]
pub struct DepthCameraConfig {
    /// Image width in pixels.
    pub width: usize,
    /// Image height in pixels.
    pub height: usize,
    /// Focal length in pixels [fx, fy].
    pub focal_length_px: [f32; 2],
    /// Principal point [cx, cy].
    pub principal_point_px: [f32; 2],
    /// Minimum depth in metres.
    pub min_depth_m: f32,
    /// Maximum depth in metres.
    pub max_depth_m: f32,
    /// Frame rate in Hz.
    pub frame_rate_hz: f32,
}

impl Default for DepthCameraConfig {
    fn default() -> Self {
        DepthCameraConfig {
            width: 640,
            height: 480,
            focal_length_px: [525.0, 525.0],
            principal_point_px: [319.5, 239.5],
            min_depth_m: 0.3,
            max_depth_m: 4.0,
            frame_rate_hz: 30.0,
        }
    }
}

/// A depth camera frame (dense depth map).
#[derive(Debug, Clone)]
pub struct DepthFrame {
    pub frame_index: u64,
    pub time: f32,
    /// Depth values in metres, length = width × height (0.0 = invalid).
    pub depth: Vec<f32>,
}

/// A 3-D point from the depth camera.
#[derive(Debug, Clone, PartialEq)]
pub struct DepthPoint {
    pub position: [f32; 3],
    pub pixel: [u32; 2],
}

/// Depth camera sensor stub.
#[derive(Debug)]
pub struct DepthCameraSensor {
    pub config: DepthCameraConfig,
    frames: Vec<DepthFrame>,
}

impl DepthCameraSensor {
    /// Create a new depth camera sensor.
    pub fn new(config: DepthCameraConfig) -> Self {
        DepthCameraSensor {
            config,
            frames: vec![],
        }
    }

    /// Record a frame.
    pub fn push_frame(&mut self, frame: DepthFrame) {
        self.frames.push(frame);
    }

    /// Return frame count.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Return the latest frame.
    pub fn latest(&self) -> Option<&DepthFrame> {
        self.frames.last()
    }

    /// Clear all frames.
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

/// Unproject a depth frame to a 3-D point cloud.
pub fn unproject_frame(frame: &DepthFrame, config: &DepthCameraConfig) -> Vec<DepthPoint> {
    let mut points = vec![];
    let [fx, fy] = config.focal_length_px;
    let [cx, cy] = config.principal_point_px;
    for row in 0..config.height {
        for col in 0..config.width {
            let idx = row * config.width + col;
            if idx >= frame.depth.len() {
                continue;
            }
            let z = frame.depth[idx];
            if z < config.min_depth_m || z > config.max_depth_m {
                continue;
            }
            let x = (col as f32 - cx) * z / fx;
            let y = (row as f32 - cy) * z / fy;
            points.push(DepthPoint {
                position: [x, y, z],
                pixel: [col as u32, row as u32],
            });
        }
    }
    points
}

/// Return the number of valid depth pixels in a frame.
pub fn valid_pixel_count(frame: &DepthFrame, config: &DepthCameraConfig) -> usize {
    frame
        .depth
        .iter()
        .filter(|&&d| d >= config.min_depth_m && d <= config.max_depth_m)
        .count()
}

/// Compute the mean depth of valid pixels.
pub fn mean_depth(frame: &DepthFrame, config: &DepthCameraConfig) -> Option<f32> {
    let valid: Vec<f32> = frame
        .depth
        .iter()
        .cloned()
        .filter(|&d| d >= config.min_depth_m && d <= config.max_depth_m)
        .collect();
    if valid.is_empty() {
        return None;
    }
    Some(valid.iter().sum::<f32>() / valid.len() as f32)
}

/// Return the expected pixel count (width × height).
pub fn expected_pixel_count(config: &DepthCameraConfig) -> usize {
    config.width * config.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_pixel_count() {
        /* expected count is width × height */
        let cfg = DepthCameraConfig::default();
        assert_eq!(expected_pixel_count(&cfg), 640 * 480);
    }

    #[test]
    fn test_push_and_count() {
        /* push increments frame count */
        let mut s = DepthCameraSensor::new(DepthCameraConfig::default());
        s.push_frame(DepthFrame {
            frame_index: 0,
            time: 0.0,
            depth: vec![1.0; 640 * 480],
        });
        assert_eq!(s.frame_count(), 1);
    }

    #[test]
    fn test_valid_pixel_count_all_valid() {
        /* all-1m frame is fully valid */
        let cfg = DepthCameraConfig::default();
        let frame = DepthFrame {
            frame_index: 0,
            time: 0.0,
            depth: vec![1.0; 640 * 480],
        };
        assert_eq!(valid_pixel_count(&frame, &cfg), 640 * 480);
    }

    #[test]
    fn test_valid_pixel_count_some_invalid() {
        /* zeros are invalid */
        let cfg = DepthCameraConfig::default();
        let mut depth = vec![1.0f32; 10];
        depth[0] = 0.0;
        let frame = DepthFrame {
            frame_index: 0,
            time: 0.0,
            depth,
        };
        assert_eq!(valid_pixel_count(&frame, &cfg), 9);
    }

    #[test]
    fn test_mean_depth_all_valid() {
        /* mean of all-1m frame is 1.0 */
        let cfg = DepthCameraConfig {
            width: 2,
            height: 2,
            ..DepthCameraConfig::default()
        };
        let frame = DepthFrame {
            frame_index: 0,
            time: 0.0,
            depth: vec![1.0; 4],
        };
        assert!((mean_depth(&frame, &cfg).unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean_depth_empty() {
        /* all-invalid frame returns None */
        let cfg = DepthCameraConfig::default();
        let frame = DepthFrame {
            frame_index: 0,
            time: 0.0,
            depth: vec![0.0; 4],
        };
        assert!(mean_depth(&frame, &cfg).is_none());
    }

    #[test]
    fn test_unproject_single_pixel() {
        /* single valid pixel unprojected gives one point */
        let cfg = DepthCameraConfig {
            width: 1,
            height: 1,
            ..DepthCameraConfig::default()
        };
        let frame = DepthFrame {
            frame_index: 0,
            time: 0.0,
            depth: vec![1.0],
        };
        let pts = unproject_frame(&frame, &cfg);
        assert_eq!(pts.len(), 1);
    }

    #[test]
    fn test_clear() {
        /* clear removes frames */
        let mut s = DepthCameraSensor::new(DepthCameraConfig::default());
        s.push_frame(DepthFrame {
            frame_index: 0,
            time: 0.0,
            depth: vec![],
        });
        s.clear();
        assert_eq!(s.frame_count(), 0);
    }

    #[test]
    fn test_latest() {
        /* latest returns last pushed */
        let mut s = DepthCameraSensor::new(DepthCameraConfig::default());
        s.push_frame(DepthFrame {
            frame_index: 3,
            time: 1.0,
            depth: vec![],
        });
        assert_eq!(s.latest().unwrap().frame_index, 3);
    }

    #[test]
    fn test_default_frame_rate() {
        /* default frame rate is 30 Hz */
        assert_eq!(DepthCameraConfig::default().frame_rate_hz, 30.0);
    }
}
