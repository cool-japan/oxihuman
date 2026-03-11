// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Motion capture marker sensor stub.

/// A motion capture marker.
#[derive(Debug, Clone, PartialEq)]
pub struct MocapMarker {
    /// Marker label.
    pub label: String,
    /// 3-D position in metres [x, y, z].
    pub position: [f32; 3],
    /// Residual (reconstruction error) in metres.
    pub residual: f32,
    /// Whether the marker is occluded in this frame.
    pub occluded: bool,
}

/// A motion capture frame.
#[derive(Debug, Clone)]
pub struct MocapFrame {
    pub frame_index: u64,
    pub time: f32,
    pub markers: Vec<MocapMarker>,
}

/// Motion capture system configuration.
#[derive(Debug, Clone)]
pub struct MocapConfig {
    pub camera_count: usize,
    pub capture_rate_hz: f32,
    pub max_residual_mm: f32,
}

impl Default for MocapConfig {
    fn default() -> Self {
        MocapConfig {
            camera_count: 8,
            capture_rate_hz: 120.0,
            max_residual_mm: 1.0,
        }
    }
}

/// Motion capture sensor.
#[derive(Debug)]
pub struct MocapSensor {
    pub config: MocapConfig,
    frames: Vec<MocapFrame>,
}

impl MocapSensor {
    /// Create a new sensor.
    pub fn new(config: MocapConfig) -> Self {
        MocapSensor {
            config,
            frames: vec![],
        }
    }

    /// Record a frame.
    pub fn push_frame(&mut self, frame: MocapFrame) {
        self.frames.push(frame);
    }

    /// Return the number of frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Return the latest frame.
    pub fn latest(&self) -> Option<&MocapFrame> {
        self.frames.last()
    }

    /// Clear all frames.
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

/// Return the number of visible (non-occluded) markers in a frame.
pub fn visible_marker_count(frame: &MocapFrame) -> usize {
    frame.markers.iter().filter(|m| !m.occluded).count()
}

/// Find a marker by label in a frame.
pub fn find_marker<'a>(frame: &'a MocapFrame, label: &str) -> Option<&'a MocapMarker> {
    frame.markers.iter().find(|m| m.label == label)
}

/// Compute the centroid position of all visible markers.
pub fn marker_centroid(frame: &MocapFrame) -> Option<[f32; 3]> {
    let visible: Vec<_> = frame.markers.iter().filter(|m| !m.occluded).collect();
    if visible.is_empty() {
        return None;
    }
    let n = visible.len() as f32;
    let mut c = [0.0f32; 3];
    for m in &visible {
        c[0] += m.position[0];
        c[1] += m.position[1];
        c[2] += m.position[2];
    }
    Some([c[0] / n, c[1] / n, c[2] / n])
}

/// Return `true` if all markers satisfy the residual threshold.
pub fn all_markers_valid(frame: &MocapFrame, max_residual_m: f32) -> bool {
    frame
        .markers
        .iter()
        .all(|m| !m.occluded && m.residual <= max_residual_m)
}

/// Compute the distance between two markers.
pub fn marker_distance(a: &MocapMarker, b: &MocapMarker) -> f32 {
    let d = [
        a.position[0] - b.position[0],
        a.position[1] - b.position[1],
        a.position[2] - b.position[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_marker(label: &str, pos: [f32; 3]) -> MocapMarker {
        MocapMarker {
            label: label.to_string(),
            position: pos,
            residual: 0.0,
            occluded: false,
        }
    }

    #[test]
    fn test_push_and_count() {
        /* push increments frame count */
        let mut s = MocapSensor::new(MocapConfig::default());
        s.push_frame(MocapFrame {
            frame_index: 0,
            time: 0.0,
            markers: vec![],
        });
        assert_eq!(s.frame_count(), 1);
    }

    #[test]
    fn test_visible_marker_count() {
        /* counts only non-occluded markers */
        let mut m = make_marker("A", [0.0; 3]);
        m.occluded = true;
        let frame = MocapFrame {
            frame_index: 0,
            time: 0.0,
            markers: vec![m, make_marker("B", [1.0; 3])],
        };
        assert_eq!(visible_marker_count(&frame), 1);
    }

    #[test]
    fn test_find_marker() {
        /* find marker by label */
        let frame = MocapFrame {
            frame_index: 0,
            time: 0.0,
            markers: vec![make_marker("LASI", [1.0, 0.0, 0.0])],
        };
        assert!(find_marker(&frame, "LASI").is_some());
        assert!(find_marker(&frame, "RASI").is_none());
    }

    #[test]
    fn test_centroid_single_marker() {
        /* centroid of one marker equals its position */
        let frame = MocapFrame {
            frame_index: 0,
            time: 0.0,
            markers: vec![make_marker("A", [1.0, 2.0, 3.0])],
        };
        let c = marker_centroid(&frame).unwrap();
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_centroid_empty() {
        /* centroid of empty frame is None */
        let frame = MocapFrame {
            frame_index: 0,
            time: 0.0,
            markers: vec![],
        };
        assert!(marker_centroid(&frame).is_none());
    }

    #[test]
    fn test_all_markers_valid_true() {
        /* all low-residual markers are valid */
        let frame = MocapFrame {
            frame_index: 0,
            time: 0.0,
            markers: vec![make_marker("A", [0.0; 3])],
        };
        assert!(all_markers_valid(&frame, 0.001));
    }

    #[test]
    fn test_marker_distance() {
        /* distance between unit-offset markers */
        let a = make_marker("A", [0.0, 0.0, 0.0]);
        let b = make_marker("B", [1.0, 0.0, 0.0]);
        assert!((marker_distance(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        /* clear removes frames */
        let mut s = MocapSensor::new(MocapConfig::default());
        s.push_frame(MocapFrame {
            frame_index: 0,
            time: 0.0,
            markers: vec![],
        });
        s.clear();
        assert_eq!(s.frame_count(), 0);
    }

    #[test]
    fn test_latest() {
        /* latest returns the last frame */
        let mut s = MocapSensor::new(MocapConfig::default());
        s.push_frame(MocapFrame {
            frame_index: 5,
            time: 0.5,
            markers: vec![],
        });
        assert_eq!(s.latest().unwrap().frame_index, 5);
    }

    #[test]
    fn test_default_camera_count() {
        /* default config has positive camera count */
        assert!(MocapConfig::default().camera_count > 0);
    }
}
