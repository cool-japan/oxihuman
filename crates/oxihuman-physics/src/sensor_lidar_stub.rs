// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! LiDAR point cloud sensor stub.

/// A single LiDAR point.
#[derive(Debug, Clone, PartialEq)]
pub struct LidarPoint {
    /// Position in metres [x, y, z].
    pub position: [f32; 3],
    /// Intensity (0.0–1.0).
    pub intensity: f32,
    /// Return index (0 = first return).
    pub return_index: u8,
}

/// A LiDAR point cloud frame.
#[derive(Debug, Clone)]
pub struct LidarFrame {
    pub frame_index: u64,
    pub time: f32,
    pub points: Vec<LidarPoint>,
}

/// LiDAR sensor configuration.
#[derive(Debug, Clone)]
pub struct LidarConfig {
    /// Number of laser channels (lines).
    pub channel_count: usize,
    /// Horizontal field of view in degrees.
    pub h_fov_deg: f32,
    /// Vertical field of view in degrees.
    pub v_fov_deg: f32,
    /// Maximum range in metres.
    pub max_range_m: f32,
    /// Rotation rate in Hz.
    pub rotation_rate_hz: f32,
}

impl Default for LidarConfig {
    fn default() -> Self {
        LidarConfig {
            channel_count: 64,
            h_fov_deg: 360.0,
            v_fov_deg: 40.0,
            max_range_m: 100.0,
            rotation_rate_hz: 10.0,
        }
    }
}

/// LiDAR sensor stub.
#[derive(Debug)]
pub struct LidarSensor {
    pub config: LidarConfig,
    frames: Vec<LidarFrame>,
}

impl LidarSensor {
    /// Create a new LiDAR sensor.
    pub fn new(config: LidarConfig) -> Self {
        LidarSensor {
            config,
            frames: vec![],
        }
    }

    /// Record a frame.
    pub fn push_frame(&mut self, frame: LidarFrame) {
        self.frames.push(frame);
    }

    /// Return frame count.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Return the latest frame.
    pub fn latest(&self) -> Option<&LidarFrame> {
        self.frames.last()
    }

    /// Clear all frames.
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

/// Compute the range (distance from origin) of a LiDAR point.
pub fn point_range(p: &LidarPoint) -> f32 {
    let [x, y, z] = p.position;
    (x * x + y * y + z * z).sqrt()
}

/// Filter points to those within a given range.
pub fn filter_by_range(frame: &LidarFrame, max_m: f32) -> Vec<&LidarPoint> {
    frame
        .points
        .iter()
        .filter(|p| point_range(p) <= max_m)
        .collect()
}

/// Compute the centroid of a point cloud.
pub fn cloud_centroid(points: &[LidarPoint]) -> Option<[f32; 3]> {
    if points.is_empty() {
        return None;
    }
    let n = points.len() as f32;
    let mut c = [0.0f32; 3];
    for p in points {
        c[0] += p.position[0];
        c[1] += p.position[1];
        c[2] += p.position[2];
    }
    Some([c[0] / n, c[1] / n, c[2] / n])
}

/// Return the number of points with intensity above a threshold.
pub fn high_intensity_count(frame: &LidarFrame, threshold: f32) -> usize {
    frame
        .points
        .iter()
        .filter(|p| p.intensity > threshold)
        .count()
}

/// Return the maximum range of any point in the frame.
pub fn max_point_range(frame: &LidarFrame) -> f32 {
    frame.points.iter().map(point_range).fold(0.0f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_point(x: f32, y: f32, z: f32) -> LidarPoint {
        LidarPoint {
            position: [x, y, z],
            intensity: 0.5,
            return_index: 0,
        }
    }

    #[test]
    fn test_point_range_unit() {
        /* unit point along x has range 1 */
        let p = make_point(1.0, 0.0, 0.0);
        assert!((point_range(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_filter_by_range() {
        /* filter keeps only near points */
        let frame = LidarFrame {
            frame_index: 0,
            time: 0.0,
            points: vec![make_point(1.0, 0.0, 0.0), make_point(200.0, 0.0, 0.0)],
        };
        let near = filter_by_range(&frame, 10.0);
        assert_eq!(near.len(), 1);
    }

    #[test]
    fn test_cloud_centroid_single() {
        /* centroid of one point is that point */
        let pts = vec![make_point(1.0, 2.0, 3.0)];
        let c = cloud_centroid(&pts).unwrap();
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cloud_centroid_empty() {
        /* empty cloud returns None */
        assert!(cloud_centroid(&[]).is_none());
    }

    #[test]
    fn test_high_intensity_count() {
        /* count points above threshold */
        let frame = LidarFrame {
            frame_index: 0,
            time: 0.0,
            points: vec![
                LidarPoint {
                    position: [0.0; 3],
                    intensity: 0.9,
                    return_index: 0,
                },
                LidarPoint {
                    position: [0.0; 3],
                    intensity: 0.1,
                    return_index: 0,
                },
            ],
        };
        assert_eq!(high_intensity_count(&frame, 0.5), 1);
    }

    #[test]
    fn test_max_point_range() {
        /* max range finds farthest point */
        let frame = LidarFrame {
            frame_index: 0,
            time: 0.0,
            points: vec![make_point(1.0, 0.0, 0.0), make_point(5.0, 0.0, 0.0)],
        };
        assert!((max_point_range(&frame) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_push_and_count() {
        /* push increments frame count */
        let mut s = LidarSensor::new(LidarConfig::default());
        s.push_frame(LidarFrame {
            frame_index: 0,
            time: 0.0,
            points: vec![],
        });
        assert_eq!(s.frame_count(), 1);
    }

    #[test]
    fn test_clear() {
        /* clear removes frames */
        let mut s = LidarSensor::new(LidarConfig::default());
        s.push_frame(LidarFrame {
            frame_index: 0,
            time: 0.0,
            points: vec![],
        });
        s.clear();
        assert_eq!(s.frame_count(), 0);
    }

    #[test]
    fn test_latest() {
        /* latest returns last pushed */
        let mut s = LidarSensor::new(LidarConfig::default());
        s.push_frame(LidarFrame {
            frame_index: 7,
            time: 1.0,
            points: vec![],
        });
        assert_eq!(s.latest().unwrap().frame_index, 7);
    }

    #[test]
    fn test_default_channel_count() {
        /* default has 64 channels */
        assert_eq!(LidarConfig::default().channel_count, 64);
    }
}
