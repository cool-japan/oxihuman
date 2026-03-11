// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Bone mapping for skeleton export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneMapExport {
    pub entries: Vec<BoneMapEntry>,
}

/// Single bone mapping entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoneMapEntry {
    pub source_name: String,
    pub target_name: String,
    pub source_index: u32,
    pub target_index: u32,
}

#[allow(dead_code)]
impl BoneMapExport {
    /// Create an empty bone map.
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Add a mapping entry.
    pub fn add(&mut self, source_name: &str, target_name: &str, source_idx: u32, target_idx: u32) {
        self.entries.push(BoneMapEntry {
            source_name: source_name.to_string(),
            target_name: target_name.to_string(),
            source_index: source_idx,
            target_index: target_idx,
        });
    }

    /// Number of mappings.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Find target index for a source index.
    pub fn map_index(&self, source_idx: u32) -> Option<u32> {
        self.entries.iter()
            .find(|e| e.source_index == source_idx)
            .map(|e| e.target_index)
    }

    /// Find target name for a source name.
    pub fn map_name(&self, source_name: &str) -> Option<&str> {
        self.entries.iter()
            .find(|e| e.source_name == source_name)
            .map(|e| e.target_name.as_str())
    }

    /// Check if all source indices are unique.
    pub fn is_unique(&self) -> bool {
        let mut seen = std::collections::HashSet::new();
        self.entries.iter().all(|e| seen.insert(e.source_index))
    }

    /// Export to JSON.
    pub fn to_json(&self) -> String {
        let mut s = String::from("[");
        for (i, e) in self.entries.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str(&format!(
                "{{\"src\":\"{}\",\"dst\":\"{}\",\"src_idx\":{},\"dst_idx\":{}}}",
                e.source_name, e.target_name, e.source_index, e.target_index
            ));
        }
        s.push(']');
        s
    }

    /// Export to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        for e in &self.entries {
            bytes.extend_from_slice(&e.source_index.to_le_bytes());
            bytes.extend_from_slice(&e.target_index.to_le_bytes());
        }
        bytes
    }
}

impl Default for BoneMapExport {
    fn default() -> Self { Self::new() }
}

/// Create identity bone map.
#[allow(dead_code)]
pub fn identity_bone_map(count: u32) -> BoneMapExport {
    let mut map = BoneMapExport::new();
    for i in 0..count {
        map.add(&format!("bone_{}", i), &format!("bone_{}", i), i, i);
    }
    map
}

/// Validate bone map.
#[allow(dead_code)]
pub fn validate_bone_map(map: &BoneMapExport) -> bool {
    map.is_unique()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = BoneMapExport::new();
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn test_add() {
        let mut m = BoneMapExport::new();
        m.add("hip", "pelvis", 0, 0);
        assert_eq!(m.count(), 1);
    }

    #[test]
    fn test_map_index() {
        let mut m = BoneMapExport::new();
        m.add("hip", "pelvis", 0, 5);
        assert_eq!(m.map_index(0), Some(5));
        assert_eq!(m.map_index(99), None);
    }

    #[test]
    fn test_map_name() {
        let mut m = BoneMapExport::new();
        m.add("hip", "pelvis", 0, 0);
        assert_eq!(m.map_name("hip"), Some("pelvis"));
    }

    #[test]
    fn test_is_unique() {
        let m = identity_bone_map(3);
        assert!(m.is_unique());
    }

    #[test]
    fn test_not_unique() {
        let mut m = BoneMapExport::new();
        m.add("a", "b", 0, 1);
        m.add("c", "d", 0, 2);
        assert!(!m.is_unique());
    }

    #[test]
    fn test_to_json() {
        let m = identity_bone_map(2);
        let json = m.to_json();
        assert!(json.contains("bone_0"));
    }

    #[test]
    fn test_to_bytes() {
        let m = identity_bone_map(2);
        let bytes = m.to_bytes();
        assert_eq!(bytes.len(), 4 + 2 * 8);
    }

    #[test]
    fn test_validate() {
        let m = identity_bone_map(3);
        assert!(validate_bone_map(&m));
    }

    #[test]
    fn test_default() {
        let m = BoneMapExport::default();
        assert_eq!(m.count(), 0);
    }
}
