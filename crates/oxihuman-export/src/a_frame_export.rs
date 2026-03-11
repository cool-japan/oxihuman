// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A-Frame VR/AR scene format export.

/// An A-Frame entity in the scene.
#[derive(Clone, Debug)]
pub struct AFrameEntity {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
}

/// An A-Frame scene document.
#[derive(Clone, Debug, Default)]
pub struct AFrameScene {
    pub entities: Vec<AFrameEntity>,
    pub title: String,
}

/// Create a new empty A-Frame scene.
pub fn new_aframe_scene(title: &str) -> AFrameScene {
    AFrameScene {
        entities: Vec::new(),
        title: title.to_string(),
    }
}

/// Add an entity to the scene.
pub fn aframe_push_entity(scene: &mut AFrameScene, tag: &str, attrs: Vec<(&str, &str)>) {
    scene.entities.push(AFrameEntity {
        tag: tag.to_string(),
        attributes: attrs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    });
}

/// Return the number of entities in the scene.
pub fn aframe_entity_count(scene: &AFrameScene) -> usize {
    scene.entities.len()
}

/// Render the scene to an HTML string.
pub fn render_aframe_html(scene: &AFrameScene) -> String {
    let mut out = format!(
        "<!DOCTYPE html>\n<html>\n<head><title>{}</title>\n\
         <script src=\"https://aframe.io/releases/1.4.0/aframe.min.js\"></script>\n\
         </head>\n<body>\n<a-scene>\n",
        html_escape_af(&scene.title)
    );
    for entity in &scene.entities {
        out.push_str(&format!("  <{}", entity.tag));
        for (k, v) in &entity.attributes {
            out.push_str(&format!(" {}=\"{}\"", k, html_escape_af(v)));
        }
        out.push_str(&format!("></{}>", entity.tag));
        out.push('\n');
    }
    out.push_str("</a-scene>\n</body>\n</html>\n");
    out
}

/// Add a mesh (as a box primitive) to the scene.
pub fn aframe_add_box(scene: &mut AFrameScene, pos: [f32; 3], color: &str) {
    let pos_str = format!("{} {} {}", pos[0], pos[1], pos[2]);
    aframe_push_entity(
        scene,
        "a-box",
        vec![("position", &pos_str), ("color", color)],
    );
}

/// Add a sphere primitive.
pub fn aframe_add_sphere(scene: &mut AFrameScene, radius: f32, pos: [f32; 3], color: &str) {
    let pos_str = format!("{} {} {}", pos[0], pos[1], pos[2]);
    let r_str = format!("{}", radius);
    aframe_push_entity(
        scene,
        "a-sphere",
        vec![("position", &pos_str), ("radius", &r_str), ("color", color)],
    );
}

/// Export mesh positions as a-frame boxes.
pub fn export_mesh_as_aframe(positions: &[[f32; 3]], title: &str) -> AFrameScene {
    let mut scene = new_aframe_scene(title);
    for &p in positions {
        aframe_add_box(&mut scene, p, "#4CC3D9");
    }
    scene
}

/// Validate scene (non-empty title, entities have non-empty tags).
pub fn validate_aframe(scene: &AFrameScene) -> bool {
    !scene.title.is_empty() && scene.entities.iter().all(|e| !e.tag.is_empty())
}

fn html_escape_af(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scene_empty() {
        let s = new_aframe_scene("test");
        assert_eq!(aframe_entity_count(&s), 0);
        assert_eq!(s.title, "test");
    }

    #[test]
    fn push_entity_increments_count() {
        let mut s = new_aframe_scene("test");
        aframe_push_entity(&mut s, "a-box", vec![("color", "red")]);
        assert_eq!(aframe_entity_count(&s), 1);
    }

    #[test]
    fn render_html_contains_aframe_tag() {
        let s = new_aframe_scene("demo");
        let html = render_aframe_html(&s);
        assert!(html.contains("<a-scene>"));
    }

    #[test]
    fn add_box_adds_entity() {
        let mut s = new_aframe_scene("test");
        aframe_add_box(&mut s, [0.0, 1.0, 0.0], "#fff");
        assert_eq!(aframe_entity_count(&s), 1);
        assert_eq!(s.entities[0].tag, "a-box");
    }

    #[test]
    fn add_sphere_adds_entity() {
        let mut s = new_aframe_scene("test");
        aframe_add_sphere(&mut s, 1.5, [0.0, 0.0, 0.0], "blue");
        assert_eq!(s.entities[0].tag, "a-sphere");
    }

    #[test]
    fn export_mesh_creates_one_entity_per_vertex() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let s = export_mesh_as_aframe(&pos, "mesh");
        assert_eq!(aframe_entity_count(&s), 3);
    }

    #[test]
    fn validate_valid_scene() {
        let mut s = new_aframe_scene("test");
        aframe_push_entity(&mut s, "a-box", vec![]);
        assert!(validate_aframe(&s));
    }

    #[test]
    fn html_escape_works() {
        let s = html_escape_af("<script>");
        assert!(s.contains("&lt;") && s.contains("&gt;"));
    }
}
