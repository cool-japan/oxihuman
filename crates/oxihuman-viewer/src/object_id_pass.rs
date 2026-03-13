// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Object ID pass: renders unique IDs per object for GPU picking.

/// An object ID entry mapping an ID to a name.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ObjectIdEntry {
    pub id: u32,
    pub name: String,
    pub visible: bool,
}

/// Object ID pass state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ObjectIdPass {
    pub entries: Vec<ObjectIdEntry>,
    pub next_id: u32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_object_id_pass() -> ObjectIdPass {
    ObjectIdPass {
        entries: Vec::new(),
        next_id: 1,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn register_object(pass: &mut ObjectIdPass, name: &str) -> u32 {
    let id = pass.next_id;
    pass.entries.push(ObjectIdEntry {
        id,
        name: name.to_string(),
        visible: true,
    });
    pass.next_id += 1;
    id
}

#[allow(dead_code)]
pub fn lookup_object(pass: &ObjectIdPass, id: u32) -> Option<&ObjectIdEntry> {
    pass.entries.iter().find(|e| e.id == id)
}

#[allow(dead_code)]
pub fn lookup_by_name<'a>(pass: &'a ObjectIdPass, name: &str) -> Option<&'a ObjectIdEntry> {
    pass.entries.iter().find(|e| e.name == name)
}

#[allow(dead_code)]
pub fn object_count(pass: &ObjectIdPass) -> usize {
    pass.entries.len()
}

#[allow(dead_code)]
pub fn set_object_visible(pass: &mut ObjectIdPass, id: u32, visible: bool) {
    if let Some(entry) = pass.entries.iter_mut().find(|e| e.id == id) {
        entry.visible = visible;
    }
}

#[allow(dead_code)]
pub fn remove_object(pass: &mut ObjectIdPass, id: u32) -> bool {
    let before = pass.entries.len();
    pass.entries.retain(|e| e.id != id);
    pass.entries.len() != before
}

/// Convert an ID to an RGB color for encoding in a render target.
#[allow(dead_code)]
pub fn id_to_color(id: u32) -> [f32; 3] {
    let r = ((id >> 16) & 0xFF) as f32 / 255.0;
    let g = ((id >> 8) & 0xFF) as f32 / 255.0;
    let b = (id & 0xFF) as f32 / 255.0;
    [r, g, b]
}

/// Decode a color back to an object ID.
#[allow(dead_code)]
pub fn color_to_id(color: [f32; 3]) -> u32 {
    let r = (color[0] * 255.0).round() as u32;
    let g = (color[1] * 255.0).round() as u32;
    let b = (color[2] * 255.0).round() as u32;
    (r << 16) | (g << 8) | b
}

#[allow(dead_code)]
pub fn object_id_pass_to_json(pass: &ObjectIdPass) -> String {
    format!(
        r#"{{"object_count":{},"next_id":{},"enabled":{}}}"#,
        pass.entries.len(),
        pass.next_id,
        pass.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_object_id_pass() {
        let p = new_object_id_pass();
        assert_eq!(object_count(&p), 0);
        assert_eq!(p.next_id, 1);
    }

    #[test]
    fn test_register_object() {
        let mut p = new_object_id_pass();
        let id = register_object(&mut p, "body");
        assert_eq!(id, 1);
        assert_eq!(object_count(&p), 1);
    }

    #[test]
    fn test_lookup_object() {
        let mut p = new_object_id_pass();
        let id = register_object(&mut p, "head");
        let entry = lookup_object(&p, id).expect("should succeed");
        assert_eq!(entry.name, "head");
    }

    #[test]
    fn test_lookup_by_name() {
        let mut p = new_object_id_pass();
        register_object(&mut p, "arm");
        let entry = lookup_by_name(&p, "arm").expect("should succeed");
        assert_eq!(entry.id, 1);
    }

    #[test]
    fn test_lookup_missing() {
        let p = new_object_id_pass();
        assert!(lookup_object(&p, 999).is_none());
    }

    #[test]
    fn test_set_visible() {
        let mut p = new_object_id_pass();
        let id = register_object(&mut p, "leg");
        set_object_visible(&mut p, id, false);
        assert!(!lookup_object(&p, id).expect("should succeed").visible);
    }

    #[test]
    fn test_remove_object() {
        let mut p = new_object_id_pass();
        let id = register_object(&mut p, "hand");
        assert!(remove_object(&mut p, id));
        assert_eq!(object_count(&p), 0);
    }

    #[test]
    fn test_id_to_color_roundtrip() {
        let id = 42;
        let color = id_to_color(id);
        let back = color_to_id(color);
        assert_eq!(back, id);
    }

    #[test]
    fn test_color_to_id() {
        let id = color_to_id([0.0, 0.0, 1.0 / 255.0]);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_object_id_pass_to_json() {
        let p = new_object_id_pass();
        let j = object_id_pass_to_json(&p);
        assert!(j.contains("object_count"));
    }
}
