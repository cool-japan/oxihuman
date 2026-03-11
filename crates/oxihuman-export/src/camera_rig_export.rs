// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Camera rig export: export camera rigs with positions and targets.

/// A camera rig keyframe.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraRigKeyframe {
    pub time: f32,
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov: f32,
}

/// Camera rig export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraRigExport {
    pub name: String,
    pub keyframes: Vec<CameraRigKeyframe>,
}

/// Create a new camera rig export.
#[allow(dead_code)]
pub fn new_camera_rig_export(name: &str) -> CameraRigExport {
    CameraRigExport {
        name: name.to_string(),
        keyframes: Vec::new(),
    }
}

/// Add a keyframe.
#[allow(dead_code)]
pub fn rig_add_keyframe(
    rig: &mut CameraRigExport,
    time: f32,
    position: [f32; 3],
    target: [f32; 3],
    fov: f32,
) {
    rig.keyframes.push(CameraRigKeyframe {
        time,
        position,
        target,
        fov,
    });
}

/// Keyframe count.
#[allow(dead_code)]
pub fn rig_keyframe_count(rig: &CameraRigExport) -> usize {
    rig.keyframes.len()
}

/// Duration (last time - first time).
#[allow(dead_code)]
pub fn rig_duration(rig: &CameraRigExport) -> f32 {
    if rig.keyframes.len() < 2 {
        return 0.0;
    }
    rig.keyframes.last().map_or(0.0, |k| k.time) - rig.keyframes.first().map_or(0.0, |k| k.time)
}

/// Get keyframe at index.
#[allow(dead_code)]
pub fn rig_keyframe_at(rig: &CameraRigExport, idx: usize) -> Option<&CameraRigKeyframe> {
    rig.keyframes.get(idx)
}

/// Clear all keyframes.
#[allow(dead_code)]
pub fn rig_clear(rig: &mut CameraRigExport) {
    rig.keyframes.clear();
}

/// Export to JSON.
#[allow(dead_code)]
pub fn camera_rig_to_json(rig: &CameraRigExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"keyframes\":{},\"duration\":{:.6}}}",
        rig.name,
        rig.keyframes.len(),
        rig_duration(rig),
    )
}

/// Validate keyframes are sorted by time.
#[allow(dead_code)]
pub fn rig_validate(rig: &CameraRigExport) -> bool {
    rig.keyframes.windows(2).all(|w| w[0].time <= w[1].time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r = new_camera_rig_export("main");
        assert_eq!(rig_keyframe_count(&r), 0);
    }

    #[test]
    fn test_add_keyframe() {
        let mut r = new_camera_rig_export("cam");
        rig_add_keyframe(&mut r, 0.0, [0.0; 3], [0.0, 0.0, -1.0], 60.0);
        assert_eq!(rig_keyframe_count(&r), 1);
    }

    #[test]
    fn test_duration() {
        let mut r = new_camera_rig_export("cam");
        rig_add_keyframe(&mut r, 0.0, [0.0; 3], [0.0; 3], 60.0);
        rig_add_keyframe(&mut r, 2.5, [1.0; 3], [0.0; 3], 60.0);
        assert!((rig_duration(&r) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_duration_single() {
        let mut r = new_camera_rig_export("cam");
        rig_add_keyframe(&mut r, 1.0, [0.0; 3], [0.0; 3], 60.0);
        assert!((rig_duration(&r)).abs() < 1e-6);
    }

    #[test]
    fn test_keyframe_at() {
        let mut r = new_camera_rig_export("cam");
        rig_add_keyframe(&mut r, 0.0, [1.0, 2.0, 3.0], [0.0; 3], 45.0);
        let kf = rig_keyframe_at(&r, 0).unwrap();
        assert!((kf.fov - 45.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        let mut r = new_camera_rig_export("cam");
        rig_add_keyframe(&mut r, 0.0, [0.0; 3], [0.0; 3], 60.0);
        rig_clear(&mut r);
        assert_eq!(rig_keyframe_count(&r), 0);
    }

    #[test]
    fn test_to_json() {
        let r = new_camera_rig_export("test");
        assert!(camera_rig_to_json(&r).contains("\"name\":\"test\""));
    }

    #[test]
    fn test_validate_sorted() {
        let mut r = new_camera_rig_export("cam");
        rig_add_keyframe(&mut r, 0.0, [0.0; 3], [0.0; 3], 60.0);
        rig_add_keyframe(&mut r, 1.0, [0.0; 3], [0.0; 3], 60.0);
        assert!(rig_validate(&r));
    }

    #[test]
    fn test_validate_unsorted() {
        let mut r = new_camera_rig_export("cam");
        rig_add_keyframe(&mut r, 2.0, [0.0; 3], [0.0; 3], 60.0);
        rig_add_keyframe(&mut r, 1.0, [0.0; 3], [0.0; 3], 60.0);
        assert!(!rig_validate(&r));
    }

    #[test]
    fn test_keyframe_at_oob() {
        let r = new_camera_rig_export("cam");
        assert!(rig_keyframe_at(&r, 0).is_none());
    }
}
