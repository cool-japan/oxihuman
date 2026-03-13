#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export NLA (Non-Linear Animation) tracks.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NlaStrip {
    pub name: String,
    pub action_name: String,
    pub frame_start: f32,
    pub frame_end: f32,
    pub scale: f32,
    pub blend_type: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NlaTrack {
    pub name: String,
    pub strips: Vec<NlaStrip>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct NlaExport {
    pub tracks: Vec<NlaTrack>,
}

#[allow(dead_code)]
pub fn new_nla_export() -> NlaExport {
    NlaExport { tracks: Vec::new() }
}

#[allow(dead_code)]
pub fn add_nla_track(exp: &mut NlaExport, name: &str) {
    exp.tracks.push(NlaTrack { name: name.to_string(), strips: Vec::new() });
}

#[allow(dead_code)]
pub fn add_nla_strip(track: &mut NlaTrack, name: &str, action: &str, start: f32, end: f32) {
    track.strips.push(NlaStrip {
        name: name.to_string(),
        action_name: action.to_string(),
        frame_start: start,
        frame_end: end,
        scale: 1.0,
        blend_type: 0,
    });
}

#[allow(dead_code)]
pub fn export_nla_to_json(exp: &NlaExport) -> String {
    let mut tracks_json = String::new();
    for (i, t) in exp.tracks.iter().enumerate() {
        if i > 0 {
            tracks_json.push(',');
        }
        let strips: Vec<String> = t
            .strips
            .iter()
            .map(|s| {
                format!(
                    r#"{{"name":"{}","action":"{}","start":{},"end":{},"scale":{},"blend":{}}}"#,
                    s.name, s.action_name, s.frame_start, s.frame_end, s.scale, s.blend_type
                )
            })
            .collect();
        tracks_json.push_str(&format!(
            r#"{{"name":"{}","strips":[{}]}}"#,
            t.name,
            strips.join(",")
        ));
    }
    format!(r#"{{"tracks":[{}]}}"#, tracks_json)
}

#[allow(dead_code)]
pub fn nla_strip_count(exp: &NlaExport) -> usize {
    exp.tracks.iter().map(|t| t.strips.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_nla_export();
        assert!(e.tracks.is_empty());
    }

    #[test]
    fn add_track_increases_count() {
        let mut e = new_nla_export();
        add_nla_track(&mut e, "Walk");
        assert_eq!(e.tracks.len(), 1);
    }

    #[test]
    fn track_name_stored() {
        let mut e = new_nla_export();
        add_nla_track(&mut e, "Run");
        assert_eq!(e.tracks[0].name, "Run");
    }

    #[test]
    fn add_strip_to_track() {
        let mut e = new_nla_export();
        add_nla_track(&mut e, "Walk");
        add_nla_strip(&mut e.tracks[0], "walk_strip", "WalkAction", 0.0, 30.0);
        assert_eq!(e.tracks[0].strips.len(), 1);
    }

    #[test]
    fn strip_fields_stored() {
        let mut e = new_nla_export();
        add_nla_track(&mut e, "t");
        add_nla_strip(&mut e.tracks[0], "s", "action_a", 10.0, 40.0);
        assert_eq!(e.tracks[0].strips[0].action_name, "action_a");
        assert!((e.tracks[0].strips[0].frame_start - 10.0).abs() < 1e-6);
    }

    #[test]
    fn nla_strip_count_total() {
        let mut e = new_nla_export();
        add_nla_track(&mut e, "t1");
        add_nla_strip(&mut e.tracks[0], "s1", "a", 0.0, 10.0);
        add_nla_track(&mut e, "t2");
        add_nla_strip(&mut e.tracks[1], "s2", "b", 0.0, 10.0);
        assert_eq!(nla_strip_count(&e), 2);
    }

    #[test]
    fn export_json_has_tracks() {
        let e = new_nla_export();
        let j = export_nla_to_json(&e);
        assert!(j.contains("tracks"));
    }

    #[test]
    fn export_json_has_track_name() {
        let mut e = new_nla_export();
        add_nla_track(&mut e, "Jump");
        let j = export_nla_to_json(&e);
        assert!(j.contains("Jump"));
    }

    #[test]
    fn strip_scale_defaults_one() {
        let mut e = new_nla_export();
        add_nla_track(&mut e, "t");
        add_nla_strip(&mut e.tracks[0], "s", "a", 0.0, 10.0);
        assert!((e.tracks[0].strips[0].scale - 1.0).abs() < 1e-6);
    }
}
