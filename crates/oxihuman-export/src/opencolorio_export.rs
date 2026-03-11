// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! OpenColorIO config export stub — serializes color pipeline configs.

/// A color space entry in an OCIO config.
#[derive(Debug, Clone)]
pub struct OcioColorSpace {
    pub name: String,
    pub family: String,
    pub description: String,
    pub is_data: bool,
}

/// A stub OCIO config.
#[derive(Debug, Default, Clone)]
pub struct OcioConfig {
    pub version: u32,
    pub color_spaces: Vec<OcioColorSpace>,
    pub default_color_space: String,
}

impl OcioConfig {
    /// Creates a new OCIO config stub.
    pub fn new(version: u32) -> Self {
        Self {
            version,
            ..Default::default()
        }
    }

    /// Adds a color space to the config.
    pub fn add_color_space(&mut self, cs: OcioColorSpace) {
        self.color_spaces.push(cs);
    }

    /// Returns the number of registered color spaces.
    pub fn color_space_count(&self) -> usize {
        self.color_spaces.len()
    }
}

/// Serializes an OCIO config to a YAML-like string stub.
pub fn export_ocio_config(config: &OcioConfig) -> String {
    let mut out = format!("ocio_profile_version: {}\n", config.version);
    out.push_str("colorspaces:\n");
    for cs in &config.color_spaces {
        out.push_str(&format!("  - name: {}\n", cs.name));
        out.push_str(&format!("    family: {}\n", cs.family));
    }
    out
}

/// Finds a color space by name.
pub fn find_color_space<'a>(config: &'a OcioConfig, name: &str) -> Option<&'a OcioColorSpace> {
    config.color_spaces.iter().find(|cs| cs.name == name)
}

/// Validates the config (checks that default color space exists).
pub fn validate_ocio_config(config: &OcioConfig) -> bool {
    config.default_color_space.is_empty()
        || config
            .color_spaces
            .iter()
            .any(|cs| cs.name == config.default_color_space)
}

/// Returns all non-data color spaces.
pub fn non_data_color_spaces(config: &OcioConfig) -> Vec<&OcioColorSpace> {
    config
        .color_spaces
        .iter()
        .filter(|cs| !cs.is_data)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cs(name: &str) -> OcioColorSpace {
        OcioColorSpace {
            name: name.to_string(),
            family: "scene".to_string(),
            description: "".to_string(),
            is_data: false,
        }
    }

    #[test]
    fn test_new_config_empty() {
        /* New config should have zero color spaces */
        assert_eq!(OcioConfig::new(2).color_space_count(), 0);
    }

    #[test]
    fn test_add_color_space() {
        /* Adding a color space should increase count */
        let mut cfg = OcioConfig::new(2);
        cfg.add_color_space(make_cs("Linear"));
        assert_eq!(cfg.color_space_count(), 1);
    }

    #[test]
    fn test_export_contains_version() {
        /* Exported string should contain the version */
        let cfg = OcioConfig::new(2);
        assert!(export_ocio_config(&cfg).contains("2"));
    }

    #[test]
    fn test_export_contains_name() {
        /* Exported string should contain the color space name */
        let mut cfg = OcioConfig::new(2);
        cfg.add_color_space(make_cs("sRGB"));
        assert!(export_ocio_config(&cfg).contains("sRGB"));
    }

    #[test]
    fn test_find_color_space_found() {
        /* find_color_space should return Some for existing name */
        let mut cfg = OcioConfig::new(2);
        cfg.add_color_space(make_cs("ACEScg"));
        assert!(find_color_space(&cfg, "ACEScg").is_some());
    }

    #[test]
    fn test_find_color_space_not_found() {
        /* find_color_space should return None for missing name */
        let cfg = OcioConfig::new(2);
        assert!(find_color_space(&cfg, "ghost").is_none());
    }

    #[test]
    fn test_validate_empty_default() {
        /* Config with empty default should validate */
        assert!(validate_ocio_config(&OcioConfig::new(2)));
    }

    #[test]
    fn test_validate_matching_default() {
        /* Config whose default exists in list should validate */
        let mut cfg = OcioConfig::new(2);
        cfg.add_color_space(make_cs("Linear"));
        cfg.default_color_space = "Linear".to_string();
        assert!(validate_ocio_config(&cfg));
    }

    #[test]
    fn test_non_data_color_spaces() {
        /* non_data_color_spaces should exclude data color spaces */
        let mut cfg = OcioConfig::new(2);
        cfg.add_color_space(OcioColorSpace {
            name: "Linear".to_string(),
            family: "".to_string(),
            description: "".to_string(),
            is_data: false,
        });
        cfg.add_color_space(OcioColorSpace {
            name: "Raw".to_string(),
            family: "".to_string(),
            description: "".to_string(),
            is_data: true,
        });
        assert_eq!(non_data_color_spaces(&cfg).len(), 1);
    }

    #[test]
    fn test_config_version_stored() {
        /* Config should store the version number */
        assert_eq!(OcioConfig::new(3).version, 3);
    }
}
