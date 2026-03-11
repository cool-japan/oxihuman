// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Animation clip export with keyframes.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeType {
    Position,
    Rotation,
    Scale,
    Morph,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub value: Vec<f32>,
    pub keyframe_type: KeyframeType,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimationClipExport {
    pub name: String,
    pub duration: f32,
    pub fps: f32,
    pub keyframes: Vec<Keyframe>,
}

#[allow(dead_code)]
pub fn new_animation_clip_export(name: &str, fps: f32) -> AnimationClipExport {
    AnimationClipExport { name: name.to_string(), duration: 0.0, fps, keyframes: Vec::new() }
}

#[allow(dead_code)]
pub fn anim_add_keyframe(clip: &mut AnimationClipExport, kf: Keyframe) {
    if kf.time > clip.duration {
        clip.duration = kf.time;
    }
    clip.keyframes.push(kf);
}

#[allow(dead_code)]
pub fn anim_keyframe_count(clip: &AnimationClipExport) -> usize {
    clip.keyframes.len()
}

#[allow(dead_code)]
pub fn anim_duration(clip: &AnimationClipExport) -> f32 {
    clip.duration
}

#[allow(dead_code)]
pub fn anim_to_json(clip: &AnimationClipExport) -> String {
    format!(
        r#"{{"name":"{}","duration":{:.4},"fps":{:.2},"keyframe_count":{}}}"#,
        clip.name, clip.duration, clip.fps, clip.keyframes.len()
    )
}

#[allow(dead_code)]
pub fn anim_validate(clip: &AnimationClipExport) -> bool {
    clip.fps > 0.0 && clip.duration >= 0.0
}

#[allow(dead_code)]
pub fn anim_keyframes_at_time(clip: &AnimationClipExport, t: f32) -> Vec<usize> {
    clip.keyframes
        .iter()
        .enumerate()
        .filter(|(_, kf)| (kf.time - t).abs() < 1e-5)
        .map(|(i, _)| i)
        .collect()
}

#[allow(dead_code)]
pub fn anim_clear(clip: &mut AnimationClipExport) {
    clip.keyframes.clear();
    clip.duration = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_clip() -> AnimationClipExport {
        new_animation_clip_export("test_clip", 24.0)
    }

    fn make_kf(t: f32, kt: KeyframeType) -> Keyframe {
        Keyframe { time: t, value: vec![1.0, 0.0, 0.0], keyframe_type: kt }
    }

    #[test]
    fn new_clip_empty() {
        let clip = make_clip();
        assert_eq!(anim_keyframe_count(&clip), 0);
        assert!((anim_duration(&clip)).abs() < 1e-6);
    }

    #[test]
    fn add_keyframe_updates_count() {
        let mut clip = make_clip();
        anim_add_keyframe(&mut clip, make_kf(0.5, KeyframeType::Position));
        assert_eq!(anim_keyframe_count(&clip), 1);
    }

    #[test]
    fn duration_tracks_max_time() {
        let mut clip = make_clip();
        anim_add_keyframe(&mut clip, make_kf(1.0, KeyframeType::Scale));
        anim_add_keyframe(&mut clip, make_kf(2.5, KeyframeType::Rotation));
        assert!((anim_duration(&clip) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn validate_ok() {
        let clip = make_clip();
        assert!(anim_validate(&clip));
    }

    #[test]
    fn keyframes_at_time_found() {
        let mut clip = make_clip();
        anim_add_keyframe(&mut clip, make_kf(1.0, KeyframeType::Morph));
        anim_add_keyframe(&mut clip, make_kf(2.0, KeyframeType::Position));
        let hits = anim_keyframes_at_time(&clip, 1.0);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0], 0);
    }

    #[test]
    fn keyframes_at_time_none() {
        let mut clip = make_clip();
        anim_add_keyframe(&mut clip, make_kf(1.0, KeyframeType::Scale));
        let hits = anim_keyframes_at_time(&clip, 3.0);
        assert!(hits.is_empty());
    }

    #[test]
    fn clear_resets() {
        let mut clip = make_clip();
        anim_add_keyframe(&mut clip, make_kf(2.0, KeyframeType::Rotation));
        anim_clear(&mut clip);
        assert_eq!(anim_keyframe_count(&clip), 0);
        assert!((anim_duration(&clip)).abs() < 1e-6);
    }

    #[test]
    fn to_json_has_name() {
        let clip = make_clip();
        let json = anim_to_json(&clip);
        assert!(json.contains("test_clip"));
        assert!(json.contains("fps"));
    }
}
