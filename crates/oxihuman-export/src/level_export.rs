// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Level/scene export container.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LevelExport {
    pub name: String,
    pub objects: Vec<LevelObject>,
}

/// Object placed in a level.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LevelObject {
    pub name: String,
    pub mesh_ref: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[allow(dead_code)]
impl LevelExport {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), objects: Vec::new() }
    }

    pub fn add_object(&mut self, name: &str, mesh_ref: &str, position: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) {
        self.objects.push(LevelObject {
            name: name.to_string(), mesh_ref: mesh_ref.to_string(),
            position, rotation, scale,
        });
    }

    pub fn object_count(&self) -> usize { self.objects.len() }

    pub fn find_object(&self, name: &str) -> Option<&LevelObject> {
        self.objects.iter().find(|o| o.name == name)
    }

    pub fn unique_meshes(&self) -> Vec<&str> {
        let mut meshes: Vec<&str> = self.objects.iter().map(|o| o.mesh_ref.as_str()).collect();
        meshes.sort();
        meshes.dedup();
        meshes
    }

    pub fn to_json(&self) -> String {
        let mut s = format!("{{\"name\":\"{}\",\"objects\":[", self.name);
        for (i, o) in self.objects.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str(&format!(
                "{{\"name\":\"{}\",\"mesh\":\"{}\"}}",
                o.name, o.mesh_ref
            ));
        }
        s.push_str("]}");
        s
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.objects.len() as u32).to_le_bytes());
        for o in &self.objects {
            for &p in &o.position { bytes.extend_from_slice(&p.to_le_bytes()); }
            for &r in &o.rotation { bytes.extend_from_slice(&r.to_le_bytes()); }
            for &s in &o.scale { bytes.extend_from_slice(&s.to_le_bytes()); }
        }
        bytes
    }
}

/// Validate level.
#[allow(dead_code)]
pub fn validate_level(level: &LevelExport) -> bool {
    !level.name.is_empty() && level.objects.iter().all(|o| !o.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_level() -> LevelExport {
        let mut l = LevelExport::new("test_level");
        l.add_object("cube1", "cube_mesh", [0.0,0.0,0.0], [0.0,0.0,0.0,1.0], [1.0,1.0,1.0]);
        l.add_object("cube2", "cube_mesh", [5.0,0.0,0.0], [0.0,0.0,0.0,1.0], [1.0,1.0,1.0]);
        l
    }

    #[test]
    fn test_object_count() { assert_eq!(sample_level().object_count(), 2); }

    #[test]
    fn test_find_object() { assert!(sample_level().find_object("cube1").is_some()); }

    #[test]
    fn test_unique_meshes() { assert_eq!(sample_level().unique_meshes().len(), 1); }

    #[test]
    fn test_to_json() { assert!(sample_level().to_json().contains("test_level")); }

    #[test]
    fn test_to_bytes() { assert!(!sample_level().to_bytes().is_empty()); }

    #[test]
    fn test_validate() { assert!(validate_level(&sample_level())); }

    #[test]
    fn test_empty_level() {
        let l = LevelExport::new("empty");
        assert_eq!(l.object_count(), 0);
    }

    #[test]
    fn test_find_missing() { assert!(sample_level().find_object("missing").is_none()); }

    #[test]
    fn test_invalid_name() {
        let l = LevelExport::new("");
        assert!(!validate_level(&l));
    }

    #[test]
    fn test_multiple_meshes() {
        let mut l = LevelExport::new("multi");
        l.add_object("a", "mesh_a", [0.0;3], [0.0,0.0,0.0,1.0], [1.0;3]);
        l.add_object("b", "mesh_b", [0.0;3], [0.0,0.0,0.0,1.0], [1.0;3]);
        assert_eq!(l.unique_meshes().len(), 2);
    }
}
