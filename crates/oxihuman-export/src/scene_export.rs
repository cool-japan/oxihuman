// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export full scene data to JSON-compatible format.

#![allow(dead_code)]

/// Reference to a scene object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneObjectRef {
    pub object_type: u8,
    pub name: String,
    pub layer: u32,
}

/// Full scene export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneExport {
    pub name: String,
    pub objects: Vec<SceneObjectRef>,
    pub frame_start: f32,
    pub frame_end: f32,
    pub fps: f32,
}

/// Create a new empty scene export.
#[allow(dead_code)]
pub fn new_scene_export(name: &str) -> SceneExport {
    SceneExport {
        name: name.to_string(),
        objects: Vec::new(),
        frame_start: 0.0,
        frame_end: 250.0,
        fps: 24.0,
    }
}

/// Add an object reference to a scene.
#[allow(dead_code)]
pub fn add_object_ref(scene: &mut SceneExport, type_: u8, name: &str, layer: u32) {
    scene.objects.push(SceneObjectRef {
        object_type: type_,
        name: name.to_string(),
        layer,
    });
}

/// Return the number of objects in the scene.
#[allow(dead_code)]
pub fn object_count(scene: &SceneExport) -> usize {
    scene.objects.len()
}

/// Serialize the scene to a JSON string.
#[allow(dead_code)]
pub fn export_scene_to_json(scene: &SceneExport) -> String {
    let objs: Vec<String> = scene
        .objects
        .iter()
        .map(|o| {
            format!(
                r#"{{"object_type":{t},"name":"{n}","layer":{l}}}"#,
                t = o.object_type,
                n = o.name,
                l = o.layer,
            )
        })
        .collect();
    format!(
        r#"{{"name":"{name}","objects":[{objects}],"frame_start":{fs},"frame_end":{fe},"fps":{fps}}}"#,
        name = scene.name,
        objects = objs.join(","),
        fs = scene.frame_start,
        fe = scene.frame_end,
        fps = scene.fps,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scene_name() {
        let s = new_scene_export("main_scene");
        assert_eq!(s.name, "main_scene");
    }

    #[test]
    fn test_new_scene_empty() {
        let s = new_scene_export("s");
        assert_eq!(object_count(&s), 0);
    }

    #[test]
    fn test_add_object_ref() {
        let mut s = new_scene_export("s");
        add_object_ref(&mut s, 0, "mesh_01", 0);
        assert_eq!(object_count(&s), 1);
    }

    #[test]
    fn test_add_multiple_objects() {
        let mut s = new_scene_export("s");
        add_object_ref(&mut s, 0, "obj1", 0);
        add_object_ref(&mut s, 1, "cam", 0);
        add_object_ref(&mut s, 2, "light", 1);
        assert_eq!(object_count(&s), 3);
    }

    #[test]
    fn test_default_fps() {
        let s = new_scene_export("s");
        assert!((s.fps - 24.0).abs() < 1e-5);
    }

    #[test]
    fn test_json_contains_name() {
        let s = new_scene_export("my_scene");
        let json = export_scene_to_json(&s);
        assert!(json.contains("my_scene"));
    }

    #[test]
    fn test_json_contains_fps() {
        let s = new_scene_export("s");
        let json = export_scene_to_json(&s);
        assert!(json.contains("fps"));
    }

    #[test]
    fn test_json_contains_object_name() {
        let mut s = new_scene_export("s");
        add_object_ref(&mut s, 0, "the_mesh", 0);
        let json = export_scene_to_json(&s);
        assert!(json.contains("the_mesh"));
    }

    #[test]
    fn test_frame_range() {
        let s = new_scene_export("s");
        assert!(s.frame_start < s.frame_end);
    }

    #[test]
    fn test_object_layer() {
        let mut s = new_scene_export("s");
        add_object_ref(&mut s, 0, "o", 5);
        assert_eq!(s.objects[0].layer, 5);
    }
}
