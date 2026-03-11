// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! LaTeX/TikZ skeleton diagram export.

#[derive(Debug, Clone)]
pub struct LatexBone {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub length: f32,
    pub angle_deg: f32,
}

#[derive(Debug, Clone)]
pub struct LatexSkeletonDoc {
    pub bones: Vec<LatexBone>,
    pub scale: f32,
    pub title: String,
}

pub fn new_latex_doc(title: &str, scale: f32) -> LatexSkeletonDoc {
    LatexSkeletonDoc {
        bones: Vec::new(),
        title: title.to_string(),
        scale,
    }
}

pub fn add_latex_bone(
    doc: &mut LatexSkeletonDoc,
    name: &str,
    x: f32,
    y: f32,
    length: f32,
    angle_deg: f32,
) {
    doc.bones.push(LatexBone {
        name: name.to_string(),
        x,
        y,
        length,
        angle_deg,
    });
}

pub fn render_latex(doc: &LatexSkeletonDoc) -> String {
    let mut s = String::new();
    s.push_str("\\documentclass{standalone}\n\\usepackage{tikz}\n\\begin{document}\n");
    s.push_str(&format!("\\begin{{tikzpicture}}[scale={}]\n", doc.scale));
    for bone in &doc.bones {
        let rad = bone.angle_deg * std::f32::consts::PI / 180.0;
        let ex = bone.x + bone.length * rad.cos();
        let ey = bone.y + bone.length * rad.sin();
        s.push_str(&format!(
            "  \\draw ({:.3},{:.3}) -- ({:.3},{:.3}) node[midway,above] {{{}}};\n",
            bone.x, bone.y, ex, ey, bone.name
        ));
    }
    s.push_str("\\end{tikzpicture}\n\\end{document}\n");
    s
}

pub fn export_latex(doc: &LatexSkeletonDoc) -> Vec<u8> {
    render_latex(doc).into_bytes()
}

pub fn bone_count_latex(doc: &LatexSkeletonDoc) -> usize {
    doc.bones.len()
}
pub fn validate_latex_doc(doc: &LatexSkeletonDoc) -> bool {
    doc.scale > 0.0
}
pub fn latex_size_bytes(doc: &LatexSkeletonDoc) -> usize {
    render_latex(doc).len()
}

pub fn default_biped_latex_doc() -> LatexSkeletonDoc {
    let mut doc = new_latex_doc("Biped Skeleton", 1.0);
    add_latex_bone(&mut doc, "spine", 0.0, 0.0, 1.5, 90.0);
    add_latex_bone(&mut doc, "L_arm", -0.5, 1.0, 1.0, 180.0);
    add_latex_bone(&mut doc, "R_arm", 0.5, 1.0, 1.0, 0.0);
    add_latex_bone(&mut doc, "L_leg", -0.3, 0.0, 1.2, 270.0);
    add_latex_bone(&mut doc, "R_leg", 0.3, 0.0, 1.2, 270.0);
    doc
}

pub fn latex_set_scale(doc: &mut LatexSkeletonDoc, scale: f32) {
    doc.scale = scale.max(0.01);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_latex_doc() {
        let d = new_latex_doc("Test", 1.0);
        assert_eq!(d.title, "Test");
        assert!((d.scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_bone() {
        let mut d = new_latex_doc("T", 1.0);
        add_latex_bone(&mut d, "spine", 0.0, 0.0, 1.0, 90.0);
        assert_eq!(bone_count_latex(&d), 1);
    }

    #[test]
    fn test_render_contains_tikz() {
        let d = new_latex_doc("T", 1.0);
        let s = render_latex(&d);
        assert!(s.contains("tikzpicture"));
    }

    #[test]
    fn test_export_latex_nonempty() {
        let d = new_latex_doc("T", 1.0);
        assert!(!export_latex(&d).is_empty());
    }

    #[test]
    fn test_validate_latex_doc() {
        let d = new_latex_doc("T", 1.0);
        assert!(validate_latex_doc(&d));
        let bad = new_latex_doc("T", 0.0);
        assert!(!validate_latex_doc(&bad));
    }

    #[test]
    fn test_default_biped_doc() {
        let d = default_biped_latex_doc();
        assert_eq!(bone_count_latex(&d), 5);
    }

    #[test]
    fn test_latex_set_scale() {
        let mut d = new_latex_doc("T", 1.0);
        latex_set_scale(&mut d, 2.5);
        assert!((d.scale - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_latex_size_bytes() {
        let d = new_latex_doc("T", 1.0);
        assert!(latex_size_bytes(&d) > 0);
    }
}
