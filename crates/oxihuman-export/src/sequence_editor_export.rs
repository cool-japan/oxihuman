#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export video sequence editor data.

#[allow(dead_code)]
pub struct SeqStrip {
    pub name: String,
    pub strip_type: String,
    pub frame_start: f32,
    pub frame_end: f32,
    pub channel: u32,
}

#[allow(dead_code)]
pub struct SequenceEditorExport {
    pub strips: Vec<SeqStrip>,
}

#[allow(dead_code)]
pub fn new_sequence_editor_export() -> SequenceEditorExport {
    SequenceEditorExport { strips: vec![] }
}

#[allow(dead_code)]
pub fn add_strip(
    exp: &mut SequenceEditorExport,
    name: &str,
    type_: &str,
    start: f32,
    end_: f32,
    ch: u32,
) {
    exp.strips.push(SeqStrip {
        name: name.to_string(),
        strip_type: type_.to_string(),
        frame_start: start,
        frame_end: end_,
        channel: ch,
    });
}

#[allow(dead_code)]
pub fn export_sequence_to_json(exp: &SequenceEditorExport) -> String {
    let strips_str: Vec<String> = exp.strips.iter().map(|s| {
        format!(
            r#"{{"name":"{}","type":"{}","start":{},"end":{},"channel":{}}}"#,
            s.name, s.strip_type, s.frame_start, s.frame_end, s.channel
        )
    }).collect();
    format!(r#"{{"strips":[{}]}}"#, strips_str.join(","))
}

#[allow(dead_code)]
pub fn strip_count(exp: &SequenceEditorExport) -> usize {
    exp.strips.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_sequence_editor_export();
        assert_eq!(strip_count(&e), 0);
    }

    #[test]
    fn add_strip_increments_count() {
        let mut e = new_sequence_editor_export();
        add_strip(&mut e, "clip1", "MOVIE", 1.0, 25.0, 1);
        assert_eq!(strip_count(&e), 1);
    }

    #[test]
    fn strip_name_stored() {
        let mut e = new_sequence_editor_export();
        add_strip(&mut e, "myclip", "MOVIE", 0.0, 10.0, 1);
        assert_eq!(e.strips[0].name, "myclip");
    }

    #[test]
    fn strip_frame_range_stored() {
        let mut e = new_sequence_editor_export();
        add_strip(&mut e, "s", "SOUND", 5.0, 30.0, 2);
        assert!((e.strips[0].frame_start - 5.0).abs() < 1e-5);
        assert!((e.strips[0].frame_end - 30.0).abs() < 1e-5);
    }

    #[test]
    fn strip_channel_stored() {
        let mut e = new_sequence_editor_export();
        add_strip(&mut e, "s", "IMAGE", 0.0, 1.0, 3);
        assert_eq!(e.strips[0].channel, 3);
    }

    #[test]
    fn export_json_contains_name() {
        let mut e = new_sequence_editor_export();
        add_strip(&mut e, "render_strip", "SCENE", 1.0, 50.0, 1);
        let json = export_sequence_to_json(&e);
        assert!(json.contains("render_strip"));
    }

    #[test]
    fn export_json_empty() {
        let e = new_sequence_editor_export();
        let json = export_sequence_to_json(&e);
        assert!(json.contains("strips"));
    }

    #[test]
    fn multiple_strips() {
        let mut e = new_sequence_editor_export();
        add_strip(&mut e, "a", "MOVIE", 0.0, 10.0, 1);
        add_strip(&mut e, "b", "SOUND", 5.0, 15.0, 2);
        assert_eq!(strip_count(&e), 2);
    }

    #[test]
    fn strip_type_stored() {
        let mut e = new_sequence_editor_export();
        add_strip(&mut e, "t", "EFFECT", 0.0, 5.0, 1);
        assert_eq!(e.strips[0].strip_type, "EFFECT");
    }
}
