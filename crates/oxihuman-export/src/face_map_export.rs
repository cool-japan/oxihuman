// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Face map export - assigns named groups to faces.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceMapExport {
    pub name: String,
    pub face_indices: Vec<u32>,
}

/// Collection of face maps.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FaceMapCollection {
    pub maps: Vec<FaceMapExport>,
}

#[allow(dead_code)]
impl FaceMapCollection {
    /// Create empty.
    pub fn new() -> Self { Self { maps: Vec::new() } }

    /// Add a face map.
    pub fn add(&mut self, name: &str, faces: Vec<u32>) {
        self.maps.push(FaceMapExport { name: name.to_string(), face_indices: faces });
    }

    /// Number of maps.
    pub fn count(&self) -> usize { self.maps.len() }

    /// Find a map by name.
    pub fn find(&self, name: &str) -> Option<&FaceMapExport> {
        self.maps.iter().find(|m| m.name == name)
    }

    /// Total faces across all maps.
    pub fn total_faces(&self) -> usize {
        self.maps.iter().map(|m| m.face_indices.len()).sum()
    }

    /// Export to JSON.
    pub fn to_json(&self) -> String {
        let mut s = String::from("[");
        for (i, m) in self.maps.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str(&format!("{{\"name\":\"{}\",\"faces\":{}}}", m.name, m.face_indices.len()));
        }
        s.push(']');
        s
    }

    /// Export to binary.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.maps.len() as u32).to_le_bytes());
        for m in &self.maps {
            let name_bytes = m.name.as_bytes();
            bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(name_bytes);
            bytes.extend_from_slice(&(m.face_indices.len() as u32).to_le_bytes());
            for &fi in &m.face_indices {
                bytes.extend_from_slice(&fi.to_le_bytes());
            }
        }
        bytes
    }

    /// Check if a face is in any map.
    pub fn face_in_any(&self, face: u32) -> bool {
        self.maps.iter().any(|m| m.face_indices.contains(&face))
    }

    /// Get all map names.
    pub fn names(&self) -> Vec<&str> {
        self.maps.iter().map(|m| m.name.as_str()).collect()
    }
}

/// Validate face map collection.
#[allow(dead_code)]
pub fn validate_face_maps(collection: &FaceMapCollection, total_faces: u32) -> bool {
    collection.maps.iter().all(|m| m.face_indices.iter().all(|&f| f < total_faces))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() { assert_eq!(FaceMapCollection::new().count(), 0); }

    #[test]
    fn test_add() {
        let mut c = FaceMapCollection::new();
        c.add("face_group_1", vec![0, 1, 2]);
        assert_eq!(c.count(), 1);
    }

    #[test]
    fn test_find() {
        let mut c = FaceMapCollection::new();
        c.add("top", vec![0]);
        assert!(c.find("top").is_some());
        assert!(c.find("bottom").is_none());
    }

    #[test]
    fn test_total_faces() {
        let mut c = FaceMapCollection::new();
        c.add("a", vec![0, 1]);
        c.add("b", vec![2, 3, 4]);
        assert_eq!(c.total_faces(), 5);
    }

    #[test]
    fn test_to_json() {
        let mut c = FaceMapCollection::new();
        c.add("test", vec![0]);
        assert!(c.to_json().contains("test"));
    }

    #[test]
    fn test_to_bytes() {
        let mut c = FaceMapCollection::new();
        c.add("x", vec![0]);
        assert!(!c.to_bytes().is_empty());
    }

    #[test]
    fn test_face_in_any() {
        let mut c = FaceMapCollection::new();
        c.add("a", vec![0, 1]);
        assert!(c.face_in_any(0));
        assert!(!c.face_in_any(5));
    }

    #[test]
    fn test_names() {
        let mut c = FaceMapCollection::new();
        c.add("alpha", vec![]);
        c.add("beta", vec![]);
        assert_eq!(c.names(), vec!["alpha", "beta"]);
    }

    #[test]
    fn test_validate() {
        let mut c = FaceMapCollection::new();
        c.add("a", vec![0, 1]);
        assert!(validate_face_maps(&c, 10));
        assert!(!validate_face_maps(&c, 1));
    }

    #[test]
    fn test_default() { assert_eq!(FaceMapCollection::default().count(), 0); }
}
