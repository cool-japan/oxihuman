#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export impostor (billboard sprite) settings.

#[allow(dead_code)]
pub struct ImpostorFrame {
    pub angle_deg: f32,
    pub elevation_deg: f32,
    pub texture_atlas_idx: u32,
}

#[allow(dead_code)]
pub struct ImpostorExport {
    pub name: String,
    pub frames: Vec<ImpostorFrame>,
    pub atlas_cols: u32,
    pub atlas_rows: u32,
}

#[allow(dead_code)]
pub fn new_impostor_export(name: &str, cols: u32, rows: u32) -> ImpostorExport {
    ImpostorExport {
        name: name.to_string(),
        frames: Vec::new(),
        atlas_cols: cols,
        atlas_rows: rows,
    }
}

#[allow(dead_code)]
pub fn add_frame(exp: &mut ImpostorExport, angle: f32, elev: f32, idx: u32) {
    exp.frames.push(ImpostorFrame {
        angle_deg: angle,
        elevation_deg: elev,
        texture_atlas_idx: idx,
    });
}

#[allow(dead_code)]
pub fn export_impostor_to_json(exp: &ImpostorExport) -> String {
    let mut s = format!(
        "{{\"name\":\"{}\",\"atlas_cols\":{},\"atlas_rows\":{},\"frames\":[",
        exp.name, exp.atlas_cols, exp.atlas_rows
    );
    for (i, f) in exp.frames.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"angle\":{},\"elevation\":{},\"atlas_idx\":{}}}",
            f.angle_deg, f.elevation_deg, f.texture_atlas_idx
        ));
    }
    s.push_str("]}");
    s
}

#[allow(dead_code)]
pub fn frame_count(exp: &ImpostorExport) -> usize {
    exp.frames.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ImpostorExport {
        let mut exp = new_impostor_export("tree", 8, 4);
        for i in 0..8u32 {
            add_frame(&mut exp, i as f32 * 45.0, 0.0, i);
        }
        exp
    }

    #[test]
    fn new_export_empty() {
        let exp = new_impostor_export("x", 4, 4);
        assert_eq!(frame_count(&exp), 0);
    }

    #[test]
    fn atlas_dims_stored() {
        let exp = new_impostor_export("x", 8, 4);
        assert_eq!(exp.atlas_cols, 8);
        assert_eq!(exp.atlas_rows, 4);
    }

    #[test]
    fn name_preserved() {
        let exp = new_impostor_export("bush", 4, 2);
        assert_eq!(exp.name, "bush");
    }

    #[test]
    fn add_frame_increases_count() {
        let mut exp = new_impostor_export("x", 4, 4);
        add_frame(&mut exp, 0.0, 0.0, 0);
        assert_eq!(frame_count(&exp), 1);
    }

    #[test]
    fn eight_frames() {
        let exp = sample();
        assert_eq!(frame_count(&exp), 8);
    }

    #[test]
    fn frame_angle_stored() {
        let exp = sample();
        assert!((exp.frames[1].angle_deg - 45.0).abs() < 1e-4);
    }

    #[test]
    fn frame_atlas_idx_stored() {
        let exp = sample();
        assert_eq!(exp.frames[3].texture_atlas_idx, 3);
    }

    #[test]
    fn json_contains_name() {
        let exp = sample();
        let json = export_impostor_to_json(&exp);
        assert!(json.contains("tree"));
    }

    #[test]
    fn json_contains_frames() {
        let exp = sample();
        let json = export_impostor_to_json(&exp);
        assert!(json.contains("frames"));
    }

    #[test]
    fn json_valid_brackets() {
        let exp = sample();
        let json = export_impostor_to_json(&exp);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }
}
