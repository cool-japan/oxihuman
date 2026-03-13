// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AAF (Advanced Authoring Format) stub export.

/// AAF essence kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AafEssenceKind {
    Video,
    Audio,
    Data,
}

/// An AAF component (clip or filler).
#[derive(Debug, Clone)]
pub struct AafComponent {
    pub name: String,
    pub kind: AafEssenceKind,
    pub length_frames: u32,
    pub source_ref: String,
}

/// An AAF track.
#[derive(Debug, Clone)]
pub struct AafTrack {
    pub track_id: u32,
    pub label: String,
    pub components: Vec<AafComponent>,
}

/// An AAF composition (master mob).
#[derive(Debug, Clone)]
pub struct AafExport {
    pub name: String,
    pub edit_rate: (u32, u32), /* numerator/denominator */
    pub tracks: Vec<AafTrack>,
}

/// Create a new AAF export.
pub fn new_aaf_export(name: &str, edit_rate_num: u32, edit_rate_den: u32) -> AafExport {
    AafExport {
        name: name.to_string(),
        edit_rate: (edit_rate_num, edit_rate_den),
        tracks: Vec::new(),
    }
}

/// Add a track.
pub fn aaf_add_track(export: &mut AafExport, track_id: u32, label: &str) {
    export.tracks.push(AafTrack {
        track_id,
        label: label.to_string(),
        components: Vec::new(),
    });
}

/// Add a component to the last track.
pub fn aaf_add_component(
    export: &mut AafExport,
    name: &str,
    kind: AafEssenceKind,
    length: u32,
    source_ref: &str,
) {
    if let Some(track) = export.tracks.last_mut() {
        track.components.push(AafComponent {
            name: name.to_string(),
            kind,
            length_frames: length,
            source_ref: source_ref.to_string(),
        });
    }
}

/// Return the track count.
pub fn aaf_track_count(export: &AafExport) -> usize {
    export.tracks.len()
}

/// Return the total component count.
pub fn aaf_component_count(export: &AafExport) -> usize {
    export.tracks.iter().map(|t| t.components.len()).sum()
}

/// Total duration of the first track in frames.
pub fn aaf_duration_frames(export: &AafExport) -> u32 {
    export
        .tracks
        .first()
        .map(|t| t.components.iter().map(|c| c.length_frames).sum())
        .unwrap_or(0)
}

/// Validate the export.
pub fn validate_aaf(export: &AafExport) -> bool {
    !export.name.is_empty() && export.edit_rate.1 > 0
}

/// Generate a stub AAF XML description.
pub fn aaf_to_xml_stub(export: &AafExport) -> String {
    format!(
        "<AAFFile><Composition name=\"{}\" editRate=\"{}/{}\"/></AAFFile>",
        export.name, export.edit_rate.0, export.edit_rate.1
    )
}

/// Estimate the AAF file size.
pub fn aaf_size_estimate(export: &AafExport) -> usize {
    aaf_to_xml_stub(export).len() + aaf_component_count(export) * 64
}

/// Find a track by ID.
pub fn aaf_find_track(export: &AafExport, track_id: u32) -> Option<&AafTrack> {
    export.tracks.iter().find(|t| t.track_id == track_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AafExport {
        let mut exp = new_aaf_export("MyComp", 24, 1);
        aaf_add_track(&mut exp, 1, "V1");
        aaf_add_component(&mut exp, "clip1", AafEssenceKind::Video, 48, "mob:001");
        aaf_add_component(&mut exp, "clip2", AafEssenceKind::Video, 24, "mob:002");
        aaf_add_track(&mut exp, 2, "A1");
        aaf_add_component(&mut exp, "audio1", AafEssenceKind::Audio, 72, "mob:003");
        exp
    }

    #[test]
    fn test_track_count() {
        assert_eq!(aaf_track_count(&sample()), 2);
    }

    #[test]
    fn test_component_count() {
        assert_eq!(aaf_component_count(&sample()), 3);
    }

    #[test]
    fn test_duration_frames() {
        assert_eq!(aaf_duration_frames(&sample()), 72);
    }

    #[test]
    fn test_validate_valid() {
        assert!(validate_aaf(&sample()));
    }

    #[test]
    fn test_validate_bad_rate() {
        let exp = new_aaf_export("X", 24, 0);
        assert!(!validate_aaf(&exp));
    }

    #[test]
    fn test_to_xml() {
        let s = aaf_to_xml_stub(&sample());
        assert!(s.contains("MyComp"));
    }

    #[test]
    fn test_find_track() {
        let exp = sample();
        assert!(aaf_find_track(&exp, 1).is_some());
        assert!(aaf_find_track(&exp, 99).is_none());
    }

    #[test]
    fn test_size_estimate() {
        assert!(aaf_size_estimate(&sample()) > 0);
    }

    #[test]
    fn test_empty_duration() {
        let exp = new_aaf_export("empty", 25, 1);
        assert_eq!(aaf_duration_frames(&exp), 0);
    }
}
