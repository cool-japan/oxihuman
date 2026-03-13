// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Layer-based export for organizing scene data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LayerExport {
    pub layers: Vec<ExportLayer>,
}

/// A single export layer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExportLayer {
    pub name: String,
    pub visible: bool,
    pub object_indices: Vec<u32>,
}

#[allow(dead_code)]
impl LayerExport {
    pub fn new() -> Self { Self { layers: Vec::new() } }

    pub fn add_layer(&mut self, name: &str, visible: bool, objects: Vec<u32>) {
        self.layers.push(ExportLayer { name: name.to_string(), visible, object_indices: objects });
    }

    pub fn count(&self) -> usize { self.layers.len() }

    pub fn find(&self, name: &str) -> Option<&ExportLayer> {
        self.layers.iter().find(|l| l.name == name)
    }

    pub fn visible_layers(&self) -> Vec<&ExportLayer> {
        self.layers.iter().filter(|l| l.visible).collect()
    }

    pub fn total_objects(&self) -> usize {
        self.layers.iter().map(|l| l.object_indices.len()).sum()
    }

    pub fn to_json(&self) -> String {
        let mut s = String::from("[");
        for (i, l) in self.layers.iter().enumerate() {
            if i > 0 { s.push(','); }
            s.push_str(&format!(
                "{{\"name\":\"{}\",\"visible\":{},\"objects\":{}}}",
                l.name, l.visible, l.object_indices.len()
            ));
        }
        s.push(']');
        s
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.layers.len() as u32).to_le_bytes());
        for l in &self.layers {
            bytes.push(if l.visible { 1 } else { 0 });
            bytes.extend_from_slice(&(l.object_indices.len() as u32).to_le_bytes());
            for &oi in &l.object_indices { bytes.extend_from_slice(&oi.to_le_bytes()); }
        }
        bytes
    }
}

impl Default for LayerExport {
    fn default() -> Self { Self::new() }
}

/// Validate layers.
#[allow(dead_code)]
pub fn validate_layers(le: &LayerExport) -> bool {
    !le.layers.is_empty() && le.layers.iter().all(|l| !l.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> LayerExport {
        let mut le = LayerExport::new();
        le.add_layer("base", true, vec![0, 1, 2]);
        le.add_layer("detail", false, vec![3, 4]);
        le
    }

    #[test]
    fn test_count() { assert_eq!(sample().count(), 2); }

    #[test]
    fn test_find() { assert!(sample().find("base").is_some()); }

    #[test]
    fn test_visible() { assert_eq!(sample().visible_layers().len(), 1); }

    #[test]
    fn test_total_objects() { assert_eq!(sample().total_objects(), 5); }

    #[test]
    fn test_to_json() { assert!(sample().to_json().contains("base")); }

    #[test]
    fn test_to_bytes() { assert!(!sample().to_bytes().is_empty()); }

    #[test]
    fn test_validate() { assert!(validate_layers(&sample())); }

    #[test]
    fn test_empty() { assert!(!validate_layers(&LayerExport::new())); }

    #[test]
    fn test_default() { assert_eq!(LayerExport::default().count(), 0); }

    #[test]
    fn test_find_missing() { assert!(sample().find("nope").is_none()); }
}
