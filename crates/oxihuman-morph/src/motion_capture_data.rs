#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A single joint in a motion capture frame.
#[derive(Debug, Clone)]
pub struct McapJoint {
    pub name: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

/// A single frame of motion capture data.
#[derive(Debug, Clone)]
pub struct McapFrame {
    pub time: f32,
    pub joints: Vec<McapJoint>,
}

/// A motion capture clip.
#[derive(Debug, Clone)]
pub struct McapClip {
    pub name: String,
    pub fps: f32,
    pub frames: Vec<McapFrame>,
}

#[allow(dead_code)]
pub fn new_mcap_clip(name: &str, fps: f32) -> McapClip {
    McapClip {
        name: name.to_string(),
        fps,
        frames: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_frame(clip: &mut McapClip, time: f32, joints: Vec<McapJoint>) {
    clip.frames.push(McapFrame { time, joints });
}

#[allow(dead_code)]
pub fn mcap_duration(clip: &McapClip) -> f32 {
    clip.frames
        .iter()
        .map(|f| f.time)
        .fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn frame_at_time(clip: &McapClip, time: f32) -> Option<&McapFrame> {
    // Return the frame whose time is closest to the requested time.
    clip.frames
        .iter()
        .min_by(|a, b| {
            (a.time - time)
                .abs()
                .partial_cmp(&(b.time - time).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clip_empty() {
        let c = new_mcap_clip("test", 30.0);
        assert_eq!(c.name, "test");
        assert!(c.frames.is_empty());
    }

    #[test]
    fn test_fps_stored() {
        let c = new_mcap_clip("clip", 60.0);
        assert!((c.fps - 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_frame() {
        let mut c = new_mcap_clip("clip", 30.0);
        add_frame(&mut c, 0.0, vec![]);
        assert_eq!(c.frames.len(), 1);
    }

    #[test]
    fn test_mcap_duration_empty() {
        let c = new_mcap_clip("clip", 30.0);
        assert!((mcap_duration(&c)).abs() < 1e-6);
    }

    #[test]
    fn test_mcap_duration_nonempty() {
        let mut c = new_mcap_clip("clip", 30.0);
        add_frame(&mut c, 0.0, vec![]);
        add_frame(&mut c, 1.0, vec![]);
        add_frame(&mut c, 2.0, vec![]);
        assert!((mcap_duration(&c) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_frame_at_time_none_empty() {
        let c = new_mcap_clip("clip", 30.0);
        assert!(frame_at_time(&c, 1.0).is_none());
    }

    #[test]
    fn test_frame_at_time_exact() {
        let mut c = new_mcap_clip("clip", 30.0);
        add_frame(&mut c, 0.5, vec![]);
        add_frame(&mut c, 1.0, vec![]);
        let f = frame_at_time(&c, 1.0).unwrap();
        assert!((f.time - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_frame_at_time_nearest() {
        let mut c = new_mcap_clip("clip", 30.0);
        add_frame(&mut c, 0.0, vec![]);
        add_frame(&mut c, 1.0, vec![]);
        let f = frame_at_time(&c, 0.4).unwrap();
        assert!((f.time - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_joint_data_stored() {
        let mut c = new_mcap_clip("clip", 30.0);
        let j = McapJoint {
            name: "Hips".to_string(),
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
        };
        add_frame(&mut c, 0.0, vec![j]);
        assert_eq!(c.frames[0].joints[0].name, "Hips");
    }
}
