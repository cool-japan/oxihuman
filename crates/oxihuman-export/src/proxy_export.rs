// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Proxy / LOD mesh export.

#![allow(dead_code)]

/// Configuration for proxy export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProxyExportConfig {
    /// Number of LOD levels.
    pub lod_levels: usize,
    /// Name of the proxy.
    pub name: String,
}

/// A single proxy LOD level.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProxyLevel {
    /// LOD index (0 = full detail).
    pub level: usize,
    /// Vertex count at this level.
    pub vertex_count: usize,
    /// Face count at this level.
    pub face_count: usize,
}

/// Container for proxy LOD export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProxyExport {
    /// Configuration.
    pub config: ProxyExportConfig,
    /// LOD levels.
    pub levels: Vec<ProxyLevel>,
}

/// Returns the default [`ProxyExportConfig`].
#[allow(dead_code)]
pub fn default_proxy_export_config() -> ProxyExportConfig {
    ProxyExportConfig {
        lod_levels: 4,
        name: "Proxy".to_string(),
    }
}

/// Creates a new empty [`ProxyExport`].
#[allow(dead_code)]
pub fn new_proxy_export(config: ProxyExportConfig) -> ProxyExport {
    ProxyExport { config, levels: Vec::new() }
}

/// Adds a LOD level.
#[allow(dead_code)]
pub fn proxy_add_level(export: &mut ProxyExport, level: ProxyLevel) {
    export.levels.push(level);
}

/// Returns the number of LOD levels.
#[allow(dead_code)]
pub fn proxy_level_count(export: &ProxyExport) -> usize {
    export.levels.len()
}

/// Returns the LOD level at `index`.
#[allow(dead_code)]
pub fn proxy_get_level(export: &ProxyExport, index: usize) -> Option<&ProxyLevel> {
    export.levels.get(index)
}

/// Serialises to a minimal JSON string.
#[allow(dead_code)]
pub fn proxy_to_json(export: &ProxyExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"lod_count\":{}}}",
        export.config.name,
        export.levels.len()
    )
}

/// Validates the proxy export (all levels have non-zero vertex/face counts).
#[allow(dead_code)]
pub fn proxy_validate(export: &ProxyExport) -> bool {
    export.levels.iter().all(|l| l.vertex_count > 0 && l.face_count > 0)
}

/// Returns the total vertex count across all LOD levels.
#[allow(dead_code)]
pub fn proxy_total_vertices(export: &ProxyExport) -> usize {
    export.levels.iter().map(|l| l.vertex_count).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_level(level: usize) -> ProxyLevel {
        ProxyLevel { level, vertex_count: 100 >> level, face_count: 50 >> level }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_proxy_export_config();
        assert_eq!(cfg.lod_levels, 4);
    }

    #[test]
    fn test_new_export_empty() {
        let export = new_proxy_export(default_proxy_export_config());
        assert_eq!(proxy_level_count(&export), 0);
    }

    #[test]
    fn test_add_level() {
        let mut export = new_proxy_export(default_proxy_export_config());
        proxy_add_level(&mut export, make_level(0));
        assert_eq!(proxy_level_count(&export), 1);
    }

    #[test]
    fn test_get_level() {
        let mut export = new_proxy_export(default_proxy_export_config());
        proxy_add_level(&mut export, make_level(0));
        assert!(proxy_get_level(&export, 0).is_some());
        assert!(proxy_get_level(&export, 99).is_none());
    }

    #[test]
    fn test_total_vertices() {
        let mut export = new_proxy_export(default_proxy_export_config());
        proxy_add_level(&mut export, make_level(0));
        proxy_add_level(&mut export, make_level(1));
        assert_eq!(proxy_total_vertices(&export), 150);
    }

    #[test]
    fn test_to_json() {
        let export = new_proxy_export(default_proxy_export_config());
        let json = proxy_to_json(&export);
        assert!(json.contains("lod_count"));
    }

    #[test]
    fn test_validate_valid() {
        let mut export = new_proxy_export(default_proxy_export_config());
        proxy_add_level(&mut export, make_level(0));
        assert!(proxy_validate(&export));
    }

    #[test]
    fn test_validate_zero_faces() {
        let mut export = new_proxy_export(default_proxy_export_config());
        proxy_add_level(&mut export, ProxyLevel { level: 3, vertex_count: 1, face_count: 0 });
        assert!(!proxy_validate(&export));
    }

    #[test]
    fn test_empty_validate() {
        let export = new_proxy_export(default_proxy_export_config());
        // No levels → vacuously true
        assert!(proxy_validate(&export));
    }
}
