#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export Three.js scene format.

#[allow(dead_code)]
pub struct ThreeJsGeometry {
    pub uuid: String,
    pub vertices: Vec<f32>,
    pub faces: Vec<u32>,
}

#[allow(dead_code)]
pub struct ThreeJsObject {
    pub name: String,
    pub geometry_uuid: String,
    pub position: [f32; 3],
}

#[allow(dead_code)]
pub struct ThreeJsScene {
    pub objects: Vec<ThreeJsObject>,
    pub geometries: Vec<ThreeJsGeometry>,
}

#[allow(dead_code)]
pub fn new_three_js_scene() -> ThreeJsScene {
    ThreeJsScene {
        objects: Vec::new(),
        geometries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_geometry(scene: &mut ThreeJsScene, uuid: &str, verts: Vec<f32>, faces: Vec<u32>) {
    scene.geometries.push(ThreeJsGeometry {
        uuid: uuid.to_string(),
        vertices: verts,
        faces,
    });
}

#[allow(dead_code)]
pub fn add_object(scene: &mut ThreeJsScene, name: &str, geo_uuid: &str, pos: [f32; 3]) {
    scene.objects.push(ThreeJsObject {
        name: name.to_string(),
        geometry_uuid: geo_uuid.to_string(),
        position: pos,
    });
}

#[allow(dead_code)]
pub fn export_threejs_to_json(scene: &ThreeJsScene) -> String {
    let mut s = "{\"geometries\":[".to_string();
    for (i, g) in scene.geometries.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"uuid\":\"{}\",\"vertex_count\":{},\"face_count\":{}}}",
            g.uuid,
            g.vertices.len(),
            g.faces.len()
        ));
    }
    s.push_str("],\"objects\":[");
    for (i, o) in scene.objects.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"geometry\":\"{}\",\"position\":[{},{},{}]}}",
            o.name, o.geometry_uuid, o.position[0], o.position[1], o.position[2]
        ));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scene_empty() {
        let s = new_three_js_scene();
        assert!(s.objects.is_empty());
        assert!(s.geometries.is_empty());
    }

    #[test]
    fn add_geometry_stored() {
        let mut s = new_three_js_scene();
        add_geometry(&mut s, "uuid-1", vec![0.0, 0.0, 0.0], vec![0, 1, 2]);
        assert_eq!(s.geometries.len(), 1);
    }

    #[test]
    fn geometry_uuid_stored() {
        let mut s = new_three_js_scene();
        add_geometry(&mut s, "my-uuid", vec![], vec![]);
        assert_eq!(s.geometries[0].uuid, "my-uuid");
    }

    #[test]
    fn add_object_stored() {
        let mut s = new_three_js_scene();
        add_object(&mut s, "mesh1", "uuid-1", [0.0, 0.0, 0.0]);
        assert_eq!(s.objects.len(), 1);
    }

    #[test]
    fn object_position_stored() {
        let mut s = new_three_js_scene();
        add_object(&mut s, "o", "u", [1.0, 2.0, 3.0]);
        assert!((s.objects[0].position[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn export_json_contains_uuid() {
        let mut s = new_three_js_scene();
        add_geometry(&mut s, "test-geo-uuid", vec![], vec![]);
        let j = export_threejs_to_json(&s);
        assert!(j.contains("test-geo-uuid"));
    }

    #[test]
    fn export_json_contains_object_name() {
        let mut s = new_three_js_scene();
        add_object(&mut s, "MyMesh", "u", [0.0, 0.0, 0.0]);
        let j = export_threejs_to_json(&s);
        assert!(j.contains("MyMesh"));
    }

    #[test]
    fn geometry_vertices_stored() {
        let mut s = new_three_js_scene();
        add_geometry(&mut s, "u", vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![]);
        assert_eq!(s.geometries[0].vertices.len(), 6);
    }

    #[test]
    fn geometry_faces_stored() {
        let mut s = new_three_js_scene();
        add_geometry(&mut s, "u", vec![], vec![0, 1, 2]);
        assert_eq!(s.geometries[0].faces.len(), 3);
    }
}
