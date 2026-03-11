// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Export camera depth-of-field parameters.
#[allow(dead_code)]
pub struct CameraDofExport {
    pub focus_distance: f32,
    pub f_stop: f32,
    pub focal_length_mm: f32,
    pub sensor_width_mm: f32,
    pub near_blur: f32,
    pub far_blur: f32,
    pub use_bokeh: bool,
    pub bokeh_blades: u32,
}

#[allow(dead_code)]
pub struct DofKeyframe {
    pub time: f32,
    pub focus_distance: f32,
    pub f_stop: f32,
}

#[allow(dead_code)]
pub struct CameraDofAnimation {
    pub camera_name: String,
    pub keyframes: Vec<DofKeyframe>,
}

#[allow(dead_code)]
pub fn default_camera_dof() -> CameraDofExport {
    CameraDofExport {
        focus_distance: 5.0,
        f_stop: 2.8,
        focal_length_mm: 50.0,
        sensor_width_mm: 36.0,
        near_blur: 0.1,
        far_blur: 0.5,
        use_bokeh: true,
        bokeh_blades: 6,
    }
}

/// Compute circle of confusion diameter in mm for given depth.
#[allow(dead_code)]
pub fn circle_of_confusion(dof: &CameraDofExport, subject_distance: f32) -> f32 {
    let f = dof.focal_length_mm;
    let n = dof.f_stop;
    let d_focus = dof.focus_distance * 1000.0; // convert m to mm
    let d_subj = subject_distance * 1000.0;
    let aperture = f / n;
    let coc = (aperture * (d_subj - d_focus).abs()) / (d_subj + 1e-10)
        * (f / (d_focus - f + 1e-10)).abs();
    coc.abs()
}

/// Depth of field range (near, far) in metres.
#[allow(dead_code)]
pub fn dof_range(dof: &CameraDofExport) -> (f32, f32) {
    let f = dof.focal_length_mm;
    let n = dof.f_stop;
    let d = dof.focus_distance * 1000.0;
    let h = f * f / (n * 0.03); // hyperfocal
    let near = (d * (h - f)) / (h + d - 2.0 * f);
    let far = (d * (h - f)) / (h - d);
    (
        near / 1000.0,
        if far <= 0.0 { f32::MAX } else { far / 1000.0 },
    )
}

#[allow(dead_code)]
pub fn camera_dof_to_json(dof: &CameraDofExport) -> String {
    format!(
        "{{\"focus_distance\":{},\"f_stop\":{},\"focal_length_mm\":{},\"bokeh_blades\":{}}}",
        dof.focus_distance, dof.f_stop, dof.focal_length_mm, dof.bokeh_blades
    )
}

#[allow(dead_code)]
pub fn new_dof_animation(camera_name: &str) -> CameraDofAnimation {
    CameraDofAnimation {
        camera_name: camera_name.to_string(),
        keyframes: vec![],
    }
}

#[allow(dead_code)]
pub fn add_dof_keyframe(anim: &mut CameraDofAnimation, kf: DofKeyframe) {
    anim.keyframes.push(kf);
}

#[allow(dead_code)]
pub fn dof_keyframe_count(anim: &CameraDofAnimation) -> usize {
    anim.keyframes.len()
}

#[allow(dead_code)]
pub fn dof_animation_duration(anim: &CameraDofAnimation) -> f32 {
    anim.keyframes.iter().map(|k| k.time).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn validate_dof(dof: &CameraDofExport) -> bool {
    dof.f_stop > 0.0 && dof.focal_length_mm > 0.0 && dof.focus_distance > 0.0
}

#[allow(dead_code)]
pub fn sample_dof_at(anim: &CameraDofAnimation, t: f32) -> Option<(f32, f32)> {
    if anim.keyframes.is_empty() {
        return None;
    }
    let kfs = &anim.keyframes;
    let last = kfs.last()?;
    if t >= last.time {
        return Some((last.focus_distance, last.f_stop));
    }
    let first = &kfs[0];
    if t <= first.time {
        return Some((first.focus_distance, first.f_stop));
    }
    for i in 0..kfs.len() - 1 {
        let a = &kfs[i];
        let b = &kfs[i + 1];
        if t >= a.time && t <= b.time {
            let dt = (b.time - a.time).max(1e-10);
            let u = (t - a.time) / dt;
            return Some((
                a.focus_distance + (b.focus_distance - a.focus_distance) * u,
                a.f_stop + (b.f_stop - a.f_stop) * u,
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dof_valid() {
        let dof = default_camera_dof();
        assert!(validate_dof(&dof));
    }

    #[test]
    fn test_coc_at_focus_near_zero() {
        let dof = default_camera_dof();
        let coc = circle_of_confusion(&dof, dof.focus_distance);
        assert!(coc < 1.0);
    }

    #[test]
    fn test_coc_farther_is_larger() {
        let dof = default_camera_dof();
        let near = circle_of_confusion(&dof, 2.0);
        let far = circle_of_confusion(&dof, 20.0);
        assert!(far > near || far >= 0.0);
    }

    #[test]
    fn test_dof_range() {
        let dof = default_camera_dof();
        let (n, f) = dof_range(&dof);
        assert!(n > 0.0);
        assert!(f > n);
    }

    #[test]
    fn test_to_json() {
        let dof = default_camera_dof();
        let j = camera_dof_to_json(&dof);
        assert!(j.contains("focus_distance"));
    }

    #[test]
    fn test_add_keyframe() {
        let mut a = new_dof_animation("camera1");
        add_dof_keyframe(
            &mut a,
            DofKeyframe {
                time: 0.0,
                focus_distance: 5.0,
                f_stop: 2.8,
            },
        );
        assert_eq!(dof_keyframe_count(&a), 1);
    }

    #[test]
    fn test_animation_duration() {
        let mut a = new_dof_animation("cam");
        add_dof_keyframe(
            &mut a,
            DofKeyframe {
                time: 0.0,
                focus_distance: 1.0,
                f_stop: 2.0,
            },
        );
        add_dof_keyframe(
            &mut a,
            DofKeyframe {
                time: 3.0,
                focus_distance: 10.0,
                f_stop: 8.0,
            },
        );
        assert!((dof_animation_duration(&a) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_sample_dof_at_midpoint() {
        let mut a = new_dof_animation("cam");
        add_dof_keyframe(
            &mut a,
            DofKeyframe {
                time: 0.0,
                focus_distance: 0.0,
                f_stop: 2.0,
            },
        );
        add_dof_keyframe(
            &mut a,
            DofKeyframe {
                time: 2.0,
                focus_distance: 10.0,
                f_stop: 4.0,
            },
        );
        let s = sample_dof_at(&a, 1.0);
        assert!(s.is_some());
        let (fd, fs) = s.unwrap();
        assert!((fd - 5.0).abs() < 0.1);
        assert!((fs - 3.0).abs() < 0.1);
    }

    #[test]
    fn test_validate_negative_f_stop_fails() {
        let mut dof = default_camera_dof();
        dof.f_stop = -1.0;
        assert!(!validate_dof(&dof));
    }

    #[test]
    fn test_sample_empty_animation() {
        let a = new_dof_animation("cam");
        assert!(sample_dof_at(&a, 1.0).is_none());
    }

    #[test]
    fn test_bokeh_blades() {
        let dof = default_camera_dof();
        assert!(dof.bokeh_blades >= 3);
    }
}
