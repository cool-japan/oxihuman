#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export animation clip (sequence of transforms over time).

#[allow(dead_code)]
pub struct TransformKey {
    pub time: f32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[allow(dead_code)]
pub struct AnimClipTrack {
    pub bone_name: String,
    pub keys: Vec<TransformKey>,
}

#[allow(dead_code)]
pub struct AnimClipExport2 {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<AnimClipTrack>,
}

#[allow(dead_code)]
pub fn new_anim_clip_export(name: &str, duration: f32) -> AnimClipExport2 {
    AnimClipExport2 {
        name: name.to_string(),
        duration,
        tracks: vec![],
    }
}

#[allow(dead_code)]
pub fn add_track(clip: &mut AnimClipExport2, bone: &str) {
    clip.tracks.push(AnimClipTrack { bone_name: bone.to_string(), keys: vec![] });
}

#[allow(dead_code)]
pub fn add_key(
    track: &mut AnimClipTrack,
    time: f32,
    pos: [f32; 3],
    rot: [f32; 4],
    scale: [f32; 3],
) {
    track.keys.push(TransformKey { time, position: pos, rotation: rot, scale });
}

#[allow(dead_code)]
pub fn export_anim_clip_to_json(clip: &AnimClipExport2) -> String {
    let tracks_str: Vec<String> = clip.tracks.iter().map(|t| {
        let keys_str: Vec<String> = t.keys.iter().map(|k| {
            format!(
                r#"{{"time":{},"pos":[{},{},{}],"rot":[{},{},{},{}],"scale":[{},{},{}]}}"#,
                k.time,
                k.position[0], k.position[1], k.position[2],
                k.rotation[0], k.rotation[1], k.rotation[2], k.rotation[3],
                k.scale[0], k.scale[1], k.scale[2]
            )
        }).collect();
        format!(r#"{{"bone":"{}","keys":[{}]}}"#, t.bone_name, keys_str.join(","))
    }).collect();
    format!(
        r#"{{"name":"{}","duration":{},"tracks":[{}]}}"#,
        clip.name, clip.duration, tracks_str.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clip_has_correct_name() {
        let c = new_anim_clip_export("run", 2.0);
        assert_eq!(c.name, "run");
    }

    #[test]
    fn new_clip_has_duration() {
        let c = new_anim_clip_export("walk", 1.5);
        assert!((c.duration - 1.5).abs() < 1e-5);
    }

    #[test]
    fn new_clip_no_tracks() {
        let c = new_anim_clip_export("idle", 0.0);
        assert!(c.tracks.is_empty());
    }

    #[test]
    fn add_track_increments_count() {
        let mut c = new_anim_clip_export("walk", 1.0);
        add_track(&mut c, "spine");
        assert_eq!(c.tracks.len(), 1);
    }

    #[test]
    fn add_key_to_track() {
        let mut track = AnimClipTrack { bone_name: "head".to_string(), keys: vec![] };
        add_key(&mut track, 0.0, [0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3]);
        assert_eq!(track.keys.len(), 1);
    }

    #[test]
    fn add_key_time_stored() {
        let mut track = AnimClipTrack { bone_name: "arm".to_string(), keys: vec![] };
        add_key(&mut track, 0.5, [0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3]);
        assert!((track.keys[0].time - 0.5).abs() < 1e-5);
    }

    #[test]
    fn export_anim_clip_contains_name() {
        let c = new_anim_clip_export("sprint", 3.0);
        let json = export_anim_clip_to_json(&c);
        assert!(json.contains("sprint"));
    }

    #[test]
    fn export_anim_clip_contains_duration() {
        let c = new_anim_clip_export("jump", 0.8);
        let json = export_anim_clip_to_json(&c);
        assert!(json.contains("0.8"));
    }

    #[test]
    fn multiple_tracks_exported() {
        let mut c = new_anim_clip_export("test", 1.0);
        add_track(&mut c, "hip");
        add_track(&mut c, "knee");
        let json = export_anim_clip_to_json(&c);
        assert!(json.contains("hip"));
        assert!(json.contains("knee"));
    }
}
