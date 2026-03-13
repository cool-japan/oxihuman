// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Camera clip plane export.

/// A camera clip plane configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraClipExport {
    pub near: f32,
    pub far: f32,
    pub fov_y_rad: f32,
    pub aspect: f32,
}

/// Keyframe for animated clip planes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClipKeyframe {
    pub time: f32,
    pub near: f32,
    pub far: f32,
}

/// Animated clip plane export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraClipAnimation {
    pub keyframes: Vec<ClipKeyframe>,
}

/// Default perspective clip config.
#[allow(dead_code)]
pub fn default_camera_clip() -> CameraClipExport {
    use std::f32::consts::FRAC_PI_4;
    CameraClipExport {
        near: 0.01,
        far: 1000.0,
        fov_y_rad: FRAC_PI_4,
        aspect: 16.0 / 9.0,
    }
}

/// Clip range (far - near).
#[allow(dead_code)]
pub fn clip_range(cam: &CameraClipExport) -> f32 {
    cam.far - cam.near
}

/// Validate clip: near > 0, far > near.
#[allow(dead_code)]
pub fn validate_clip(cam: &CameraClipExport) -> bool {
    cam.near > 0.0 && cam.far > cam.near && cam.aspect > 0.0
}

/// New animation.
#[allow(dead_code)]
pub fn new_clip_animation() -> CameraClipAnimation {
    CameraClipAnimation {
        keyframes: Vec::new(),
    }
}

/// Add a keyframe.
#[allow(dead_code)]
pub fn add_clip_keyframe(anim: &mut CameraClipAnimation, time: f32, near: f32, far: f32) {
    anim.keyframes.push(ClipKeyframe { time, near, far });
}

/// Keyframe count.
#[allow(dead_code)]
pub fn clip_keyframe_count(anim: &CameraClipAnimation) -> usize {
    anim.keyframes.len()
}

/// Duration of clip animation.
#[allow(dead_code)]
pub fn clip_animation_duration(anim: &CameraClipAnimation) -> f32 {
    anim.keyframes
        .iter()
        .map(|k| k.time)
        .fold(0.0_f32, f32::max)
}

/// Sample near/far at time t (linear interpolation between keyframes).
#[allow(dead_code)]
pub fn sample_clip_at(anim: &CameraClipAnimation, t: f32) -> Option<(f32, f32)> {
    if anim.keyframes.is_empty() {
        return None;
    }
    let kf = &anim.keyframes;
    if t <= kf[0].time {
        return Some((kf[0].near, kf[0].far));
    }
    if t >= kf[kf.len() - 1].time {
        let last = &kf[kf.len() - 1];
        return Some((last.near, last.far));
    }
    for i in 0..kf.len() - 1 {
        let a = &kf[i];
        let b = &kf[i + 1];
        if t >= a.time && t <= b.time {
            let dt = b.time - a.time;
            let alpha = if dt < 1e-12 { 0.0 } else { (t - a.time) / dt };
            return Some((
                a.near + alpha * (b.near - a.near),
                a.far + alpha * (b.far - a.far),
            ));
        }
    }
    None
}

/// Export to JSON.
#[allow(dead_code)]
pub fn camera_clip_to_json(cam: &CameraClipExport) -> String {
    format!(
        "{{\"near\":{:.6},\"far\":{:.6},\"fov_y\":{:.6},\"aspect\":{:.6}}}",
        cam.near, cam.far, cam.fov_y_rad, cam.aspect
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_4;

    #[test]
    fn test_default_camera_clip() {
        let cam = default_camera_clip();
        assert!(validate_clip(&cam));
    }

    #[test]
    fn test_clip_range() {
        let cam = CameraClipExport {
            near: 0.1,
            far: 100.0,
            fov_y_rad: FRAC_PI_4,
            aspect: 1.0,
        };
        assert!((clip_range(&cam) - 99.9).abs() < 1e-3);
    }

    #[test]
    fn test_validate_clip_invalid() {
        let cam = CameraClipExport {
            near: -1.0,
            far: 10.0,
            fov_y_rad: FRAC_PI_4,
            aspect: 1.0,
        };
        assert!(!validate_clip(&cam));
    }

    #[test]
    fn test_add_clip_keyframe() {
        let mut anim = new_clip_animation();
        add_clip_keyframe(&mut anim, 0.0, 0.01, 100.0);
        assert_eq!(clip_keyframe_count(&anim), 1);
    }

    #[test]
    fn test_clip_animation_duration() {
        let mut anim = new_clip_animation();
        add_clip_keyframe(&mut anim, 0.0, 0.01, 100.0);
        add_clip_keyframe(&mut anim, 2.0, 0.01, 200.0);
        assert!((clip_animation_duration(&anim) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_sample_clip_empty() {
        let anim = new_clip_animation();
        assert!(sample_clip_at(&anim, 1.0).is_none());
    }

    #[test]
    fn test_sample_clip_at_midpoint() {
        let mut anim = new_clip_animation();
        add_clip_keyframe(&mut anim, 0.0, 0.1, 100.0);
        add_clip_keyframe(&mut anim, 2.0, 0.1, 200.0);
        let (_, far) = sample_clip_at(&anim, 1.0).expect("should succeed");
        assert!((far - 150.0).abs() < 1e-3);
    }

    #[test]
    fn test_camera_clip_to_json() {
        let cam = default_camera_clip();
        let j = camera_clip_to_json(&cam);
        assert!(j.contains("\"near\":"));
    }

    #[test]
    fn test_fov_in_range() {
        let cam = default_camera_clip();
        assert!((0.0..=std::f32::consts::PI).contains(&cam.fov_y_rad));
    }

    #[test]
    fn test_clip_range_positive() {
        let cam = default_camera_clip();
        assert!(clip_range(&cam) > 0.0);
    }
}
