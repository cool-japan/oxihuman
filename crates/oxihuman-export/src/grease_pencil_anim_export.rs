#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export grease pencil animation frames.

#[allow(dead_code)]
pub struct GpKeyframe {
    pub frame: u32,
    pub layer_name: String,
    pub stroke_count: u32,
}

#[allow(dead_code)]
pub struct GpAnimExport {
    pub keyframes: Vec<GpKeyframe>,
}

#[allow(dead_code)]
pub fn new_gp_anim_export() -> GpAnimExport {
    GpAnimExport { keyframes: vec![] }
}

#[allow(dead_code)]
pub fn add_keyframe(exp: &mut GpAnimExport, frame: u32, layer: &str, strokes: u32) {
    exp.keyframes.push(GpKeyframe {
        frame,
        layer_name: layer.to_string(),
        stroke_count: strokes,
    });
}

#[allow(dead_code)]
pub fn export_gp_anim_to_json(exp: &GpAnimExport) -> String {
    let kf_str: Vec<String> = exp.keyframes.iter().map(|k| {
        format!(
            r#"{{"frame":{},"layer":"{}","strokes":{}}}"#,
            k.frame, k.layer_name, k.stroke_count
        )
    }).collect();
    format!(r#"{{"keyframes":[{}]}}"#, kf_str.join(","))
}

#[allow(dead_code)]
pub fn keyframe_count(exp: &GpAnimExport) -> usize {
    exp.keyframes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_gp_anim_export();
        assert_eq!(keyframe_count(&e), 0);
    }

    #[test]
    fn add_keyframe_increments_count() {
        let mut e = new_gp_anim_export();
        add_keyframe(&mut e, 1, "Lines", 3);
        assert_eq!(keyframe_count(&e), 1);
    }

    #[test]
    fn keyframe_frame_stored() {
        let mut e = new_gp_anim_export();
        add_keyframe(&mut e, 10, "Layer1", 2);
        assert_eq!(e.keyframes[0].frame, 10);
    }

    #[test]
    fn keyframe_layer_stored() {
        let mut e = new_gp_anim_export();
        add_keyframe(&mut e, 1, "Ink", 5);
        assert_eq!(e.keyframes[0].layer_name, "Ink");
    }

    #[test]
    fn keyframe_stroke_count_stored() {
        let mut e = new_gp_anim_export();
        add_keyframe(&mut e, 1, "L", 7);
        assert_eq!(e.keyframes[0].stroke_count, 7);
    }

    #[test]
    fn export_json_contains_layer() {
        let mut e = new_gp_anim_export();
        add_keyframe(&mut e, 1, "sketch", 2);
        let json = export_gp_anim_to_json(&e);
        assert!(json.contains("sketch"));
    }

    #[test]
    fn export_json_empty() {
        let e = new_gp_anim_export();
        let json = export_gp_anim_to_json(&e);
        assert!(json.contains("keyframes"));
    }

    #[test]
    fn multiple_keyframes() {
        let mut e = new_gp_anim_export();
        add_keyframe(&mut e, 1, "A", 1);
        add_keyframe(&mut e, 5, "B", 2);
        add_keyframe(&mut e, 10, "C", 3);
        assert_eq!(keyframe_count(&e), 3);
    }

    #[test]
    fn export_json_contains_frame_number() {
        let mut e = new_gp_anim_export();
        add_keyframe(&mut e, 42, "layer", 1);
        let json = export_gp_anim_to_json(&e);
        assert!(json.contains("42"));
    }
}
