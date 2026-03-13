// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Camera animation export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraKeyframe {
    pub time: f32,
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraAnimExport {
    pub name: String,
    pub fps: f32,
    pub keyframes: Vec<CameraKeyframe>,
}

#[allow(dead_code)]
pub fn new_camera_anim_export(name: &str, fps: f32) -> CameraAnimExport {
    CameraAnimExport { name: name.to_string(), fps, keyframes: Vec::new() }
}

#[allow(dead_code)]
pub fn ca_add_keyframe(exp: &mut CameraAnimExport, kf: CameraKeyframe) {
    exp.keyframes.push(kf);
}

#[allow(dead_code)]
pub fn ca_keyframe_count(exp: &CameraAnimExport) -> usize {
    exp.keyframes.len()
}

#[allow(dead_code)]
pub fn ca_duration(exp: &CameraAnimExport) -> f32 {
    exp.keyframes.iter().map(|kf| kf.time).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn ca_to_json(exp: &CameraAnimExport) -> String {
    format!(
        r#"{{"name":"{}","fps":{:.2},"keyframe_count":{}}}"#,
        exp.name, exp.fps, exp.keyframes.len()
    )
}

#[allow(dead_code)]
pub fn ca_validate(exp: &CameraAnimExport) -> bool {
    exp.fps > 0.0 && exp.keyframes.iter().all(|kf| kf.fov > 0.0)
}

/// Linear interpolation between bracketing keyframes.
#[allow(dead_code)]
pub fn ca_interpolate_at(exp: &CameraAnimExport, t: f32) -> Option<CameraKeyframe> {
    if exp.keyframes.is_empty() {
        return None;
    }
    // Find bracketing keyframes
    let mut before: Option<&CameraKeyframe> = None;
    let mut after: Option<&CameraKeyframe> = None;
    for kf in &exp.keyframes {
        if kf.time <= t && before.is_none_or(|b: &CameraKeyframe| kf.time >= b.time) {
            before = Some(kf);
        }
        if kf.time >= t && after.is_none_or(|a: &CameraKeyframe| kf.time <= a.time) {
            after = Some(kf);
        }
    }
    match (before, after) {
        (Some(b), Some(a)) if (a.time - b.time).abs() < 1e-6 => Some(b.clone()),
        (Some(b), Some(a)) => {
            let u = (t - b.time) / (a.time - b.time);
            Some(CameraKeyframe {
                time: t,
                position: lerp3(b.position, a.position, u),
                target: lerp3(b.target, a.target, u),
                fov: b.fov + (a.fov - b.fov) * u,
            })
        }
        (Some(b), None) => Some(b.clone()),
        (None, Some(a)) => Some(a.clone()),
        (None, None) => None,
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

#[allow(dead_code)]
pub fn ca_clear(exp: &mut CameraAnimExport) {
    exp.keyframes.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_kf(t: f32) -> CameraKeyframe {
        CameraKeyframe {
            time: t,
            position: [0.0, 0.0, t],
            target: [0.0, 0.0, 0.0],
            fov: 60.0,
        }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_camera_anim_export("cam", 24.0);
        assert_eq!(ca_keyframe_count(&exp), 0);
    }

    #[test]
    fn add_keyframe_increments() {
        let mut exp = new_camera_anim_export("cam", 24.0);
        ca_add_keyframe(&mut exp, make_kf(0.0));
        assert_eq!(ca_keyframe_count(&exp), 1);
    }

    #[test]
    fn duration_is_max_time() {
        let mut exp = new_camera_anim_export("cam", 24.0);
        ca_add_keyframe(&mut exp, make_kf(0.0));
        ca_add_keyframe(&mut exp, make_kf(5.0));
        assert!((ca_duration(&exp) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn validate_ok() {
        let mut exp = new_camera_anim_export("cam", 24.0);
        ca_add_keyframe(&mut exp, make_kf(0.0));
        assert!(ca_validate(&exp));
    }

    #[test]
    fn interpolate_at_exact() {
        let mut exp = new_camera_anim_export("cam", 24.0);
        ca_add_keyframe(&mut exp, make_kf(1.0));
        let kf = ca_interpolate_at(&exp, 1.0).expect("should succeed");
        assert!((kf.time - 1.0).abs() < 1e-5);
    }

    #[test]
    fn interpolate_between_keyframes() {
        let mut exp = new_camera_anim_export("cam", 24.0);
        ca_add_keyframe(&mut exp, make_kf(0.0));
        ca_add_keyframe(&mut exp, make_kf(2.0));
        let kf = ca_interpolate_at(&exp, 1.0).expect("should succeed");
        // position.z should be ~1.0 (midpoint)
        assert!((kf.position[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn interpolate_empty_is_none() {
        let exp = new_camera_anim_export("cam", 24.0);
        assert!(ca_interpolate_at(&exp, 1.0).is_none());
    }

    #[test]
    fn clear_removes_all() {
        let mut exp = new_camera_anim_export("cam", 24.0);
        ca_add_keyframe(&mut exp, make_kf(0.0));
        ca_clear(&mut exp);
        assert_eq!(ca_keyframe_count(&exp), 0);
    }
}
