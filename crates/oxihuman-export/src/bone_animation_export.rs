// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export bone animation keyframes.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneKeyframe {
    pub time: f32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneAnimTrack {
    pub bone_name: String,
    pub keyframes: Vec<BoneKeyframe>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneAnimationExport {
    pub clip_name: String,
    pub tracks: Vec<BoneAnimTrack>,
    pub fps: f32,
}

#[allow(dead_code)]
pub fn new_bone_animation_export(clip: &str, fps: f32) -> BoneAnimationExport {
    BoneAnimationExport { clip_name: clip.to_string(), tracks: Vec::new(), fps: fps.max(1.0) }
}

#[allow(dead_code)]
pub fn bae_add_track(bae: &mut BoneAnimationExport, bone: &str) -> usize {
    bae.tracks.push(BoneAnimTrack { bone_name: bone.to_string(), keyframes: Vec::new() });
    bae.tracks.len() - 1
}

#[allow(dead_code)]
pub fn bae_add_keyframe(bae: &mut BoneAnimationExport, track: usize, kf: BoneKeyframe) {
    if track < bae.tracks.len() { bae.tracks[track].keyframes.push(kf); }
}

#[allow(dead_code)]
pub fn bae_track_count(bae: &BoneAnimationExport) -> usize { bae.tracks.len() }

#[allow(dead_code)]
pub fn bae_total_keyframes(bae: &BoneAnimationExport) -> usize {
    bae.tracks.iter().map(|t| t.keyframes.len()).sum()
}

#[allow(dead_code)]
pub fn bae_duration(bae: &BoneAnimationExport) -> f32 {
    bae.tracks.iter().flat_map(|t| t.keyframes.iter().map(|k| k.time)).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn bae_validate(bae: &BoneAnimationExport) -> bool {
    !bae.tracks.is_empty() && bae.tracks.iter().all(|t| !t.keyframes.is_empty())
}

#[allow(dead_code)]
pub fn bae_to_json(bae: &BoneAnimationExport) -> String {
    format!("{{\"clip\":\"{}\",\"fps\":{:.1},\"tracks\":{},\"duration\":{:.4}}}", bae.clip_name, bae.fps, bae.tracks.len(), bae_duration(bae))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kf(t: f32) -> BoneKeyframe { BoneKeyframe { time: t, position: [0.0,0.0,0.0], rotation: [0.0,0.0,0.0,1.0], scale: [1.0,1.0,1.0] } }

    fn anim() -> BoneAnimationExport {
        let mut a = new_bone_animation_export("walk", 30.0);
        let t = bae_add_track(&mut a, "hip");
        bae_add_keyframe(&mut a, t, kf(0.0));
        bae_add_keyframe(&mut a, t, kf(1.0));
        a
    }

    #[test] fn test_new() { let a = new_bone_animation_export("idle", 24.0); assert_eq!(a.tracks.len(), 0); }
    #[test] fn test_add_track() { assert_eq!(bae_track_count(&anim()), 1); }
    #[test] fn test_total_kf() { assert_eq!(bae_total_keyframes(&anim()), 2); }
    #[test] fn test_duration() { assert!((bae_duration(&anim()) - 1.0).abs() < 1e-5); }
    #[test] fn test_validate() { assert!(bae_validate(&anim())); }
    #[test] fn test_to_json() { assert!(bae_to_json(&anim()).contains("walk")); }
    #[test] fn test_fps() { let a = anim(); assert!((a.fps - 30.0).abs() < 1e-5); }
    #[test] fn test_empty_invalid() { let a = new_bone_animation_export("x", 30.0); assert!(!bae_validate(&a)); }
    #[test] fn test_clip_name() { let a = anim(); assert_eq!(a.clip_name, "walk"); }
    #[test] fn test_bone_name() { let a = anim(); assert_eq!(a.tracks[0].bone_name, "hip"); }
}
