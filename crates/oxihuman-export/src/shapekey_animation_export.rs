// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shape key animation timeline export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeKeyKeyframe {
    pub time: f32,
    pub key_name: String,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeKeyAnimExport {
    pub name: String,
    pub fps: f32,
    pub keyframes: Vec<ShapeKeyKeyframe>,
}

#[allow(dead_code)]
pub fn new_shapekey_anim_export(name: &str, fps: f32) -> ShapeKeyAnimExport {
    ShapeKeyAnimExport { name: name.to_string(), fps, keyframes: Vec::new() }
}

#[allow(dead_code)]
pub fn ska_add_keyframe(exp: &mut ShapeKeyAnimExport, kf: ShapeKeyKeyframe) {
    exp.keyframes.push(kf);
}

#[allow(dead_code)]
pub fn ska_keyframe_count(exp: &ShapeKeyAnimExport) -> usize {
    exp.keyframes.len()
}

#[allow(dead_code)]
pub fn ska_duration(exp: &ShapeKeyAnimExport) -> f32 {
    exp.keyframes.iter().map(|kf| kf.time).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn ska_to_json(exp: &ShapeKeyAnimExport) -> String {
    format!(
        r#"{{"name":"{}","fps":{:.2},"keyframe_count":{}}}"#,
        exp.name, exp.fps, exp.keyframes.len()
    )
}

#[allow(dead_code)]
pub fn ska_validate(exp: &ShapeKeyAnimExport) -> bool {
    exp.fps > 0.0 && exp.keyframes.iter().all(|kf| !kf.key_name.is_empty())
}

#[allow(dead_code)]
pub fn ska_keys_at_time(exp: &ShapeKeyAnimExport, t: f32) -> Vec<usize> {
    exp.keyframes
        .iter()
        .enumerate()
        .filter(|(_, kf)| (kf.time - t).abs() < 1e-5)
        .map(|(i, _)| i)
        .collect()
}

#[allow(dead_code)]
pub fn ska_clear(exp: &mut ShapeKeyAnimExport) {
    exp.keyframes.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_kf(t: f32, key: &str, w: f32) -> ShapeKeyKeyframe {
        ShapeKeyKeyframe { time: t, key_name: key.to_string(), weight: w }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_shapekey_anim_export("anim", 30.0);
        assert_eq!(ska_keyframe_count(&exp), 0);
    }

    #[test]
    fn add_keyframe_increments() {
        let mut exp = new_shapekey_anim_export("anim", 30.0);
        ska_add_keyframe(&mut exp, make_kf(0.5, "smile", 0.8));
        assert_eq!(ska_keyframe_count(&exp), 1);
    }

    #[test]
    fn duration_is_max_time() {
        let mut exp = new_shapekey_anim_export("anim", 30.0);
        ska_add_keyframe(&mut exp, make_kf(1.0, "blink", 1.0));
        ska_add_keyframe(&mut exp, make_kf(3.5, "smile", 0.5));
        assert!((ska_duration(&exp) - 3.5).abs() < 1e-6);
    }

    #[test]
    fn duration_empty_is_zero() {
        let exp = new_shapekey_anim_export("anim", 24.0);
        assert!((ska_duration(&exp)).abs() < 1e-6);
    }

    #[test]
    fn validate_ok() {
        let mut exp = new_shapekey_anim_export("anim", 24.0);
        ska_add_keyframe(&mut exp, make_kf(0.0, "key1", 1.0));
        assert!(ska_validate(&exp));
    }

    #[test]
    fn keys_at_time_found() {
        let mut exp = new_shapekey_anim_export("anim", 24.0);
        ska_add_keyframe(&mut exp, make_kf(1.0, "key", 0.5));
        let hits = ska_keys_at_time(&exp, 1.0);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn clear_removes_all() {
        let mut exp = new_shapekey_anim_export("anim", 24.0);
        ska_add_keyframe(&mut exp, make_kf(0.0, "k", 0.0));
        ska_clear(&mut exp);
        assert_eq!(ska_keyframe_count(&exp), 0);
    }

    #[test]
    fn to_json_contains_name() {
        let exp = new_shapekey_anim_export("my_anim", 30.0);
        let json = ska_to_json(&exp);
        assert!(json.contains("my_anim"));
        assert!(json.contains("fps"));
    }
}
