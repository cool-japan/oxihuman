// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SVG skeleton/rig diagram export.

#[derive(Debug, Clone)]
pub struct SvgBone {
    pub name: String,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct SvgSkeletonDoc {
    pub bones: Vec<SvgBone>,
    pub width: f32,
    pub height: f32,
    pub title: String,
}

pub fn new_svg_skeleton_doc(title: &str, width: f32, height: f32) -> SvgSkeletonDoc {
    SvgSkeletonDoc {
        bones: Vec::new(),
        width,
        height,
        title: title.to_string(),
    }
}

pub fn add_svg_bone(
    doc: &mut SvgSkeletonDoc,
    name: &str,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: &str,
) {
    doc.bones.push(SvgBone {
        name: name.to_string(),
        x0,
        y0,
        x1,
        y1,
        color: color.to_string(),
    });
}

pub fn render_svg_skeleton(doc: &SvgSkeletonDoc) -> String {
    let mut s = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <title>{}</title>
"#,
        doc.width, doc.height, doc.width, doc.height, doc.title
    );
    for bone in &doc.bones {
        s.push_str(&format!(
            "  <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"3\"/>\n",
            bone.x0, bone.y0, bone.x1, bone.y1, bone.color
        ));
        let mx = (bone.x0 + bone.x1) * 0.5;
        let my = (bone.y0 + bone.y1) * 0.5;
        s.push_str(&format!(
            "  <text x=\"{:.2}\" y=\"{:.2}\" font-size=\"12\">{}</text>\n",
            mx, my, bone.name
        ));
    }
    s.push_str("</svg>\n");
    s
}

pub fn export_svg_skeleton(doc: &SvgSkeletonDoc) -> Vec<u8> {
    render_svg_skeleton(doc).into_bytes()
}
pub fn svg_bone_count(doc: &SvgSkeletonDoc) -> usize {
    doc.bones.len()
}
pub fn validate_svg_skeleton_doc(doc: &SvgSkeletonDoc) -> bool {
    doc.width > 0.0 && doc.height > 0.0
}
pub fn svg_skeleton_size_bytes(doc: &SvgSkeletonDoc) -> usize {
    render_svg_skeleton(doc).len()
}

pub fn default_biped_svg_skeleton() -> SvgSkeletonDoc {
    let mut d = new_svg_skeleton_doc("Biped", 400.0, 600.0);
    add_svg_bone(&mut d, "spine", 200.0, 300.0, 200.0, 100.0, "#333");
    add_svg_bone(&mut d, "L_arm", 200.0, 150.0, 100.0, 250.0, "#555");
    add_svg_bone(&mut d, "R_arm", 200.0, 150.0, 300.0, 250.0, "#555");
    d
}

pub fn bone_length_px(bone: &SvgBone) -> f32 {
    let dx = bone.x1 - bone.x0;
    let dy = bone.y1 - bone.y0;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_svg_skeleton_doc() {
        let d = new_svg_skeleton_doc("T", 800.0, 600.0);
        assert_eq!(d.title, "T");
    }

    #[test]
    fn test_add_svg_bone() {
        let mut d = new_svg_skeleton_doc("T", 400.0, 600.0);
        add_svg_bone(&mut d, "spine", 0.0, 0.0, 0.0, 100.0, "#000");
        assert_eq!(svg_bone_count(&d), 1);
    }

    #[test]
    fn test_render_contains_svg_tag() {
        let d = new_svg_skeleton_doc("T", 400.0, 600.0);
        assert!(render_svg_skeleton(&d).contains("<svg"));
    }

    #[test]
    fn test_export_svg_skeleton_nonempty() {
        let d = new_svg_skeleton_doc("T", 400.0, 600.0);
        assert!(!export_svg_skeleton(&d).is_empty());
    }

    #[test]
    fn test_validate_svg_skeleton_doc() {
        let d = new_svg_skeleton_doc("T", 400.0, 600.0);
        assert!(validate_svg_skeleton_doc(&d));
        let bad = new_svg_skeleton_doc("T", 0.0, 0.0);
        assert!(!validate_svg_skeleton_doc(&bad));
    }

    #[test]
    fn test_default_biped_svg_skeleton() {
        let d = default_biped_svg_skeleton();
        assert!(svg_bone_count(&d) >= 3);
    }

    #[test]
    fn test_bone_length_px() {
        let bone = SvgBone {
            name: "b".into(),
            x0: 0.0,
            y0: 0.0,
            x1: 3.0,
            y1: 4.0,
            color: "#000".into(),
        };
        assert!((bone_length_px(&bone) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_svg_skeleton_size_bytes() {
        let d = new_svg_skeleton_doc("T", 400.0, 600.0);
        assert!(svg_skeleton_size_bytes(&d) > 0);
    }
}
