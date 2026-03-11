// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Geometry modifier type.
#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum GeoModType {
    Subdivide,
    Decimate,
    Mirror,
    Solidify,
    Bevel,
    Custom(String),
}

impl GeoModType {
    #[allow(dead_code)]
    pub fn type_name(&self) -> String {
        match self {
            GeoModType::Subdivide => "subdivide".to_string(),
            GeoModType::Decimate => "decimate".to_string(),
            GeoModType::Mirror => "mirror".to_string(),
            GeoModType::Solidify => "solidify".to_string(),
            GeoModType::Bevel => "bevel".to_string(),
            GeoModType::Custom(s) => s.clone(),
        }
    }
}

/// A geometry modifier entry.
#[allow(dead_code)]
pub struct GeoModEntry {
    pub name: String,
    pub mod_type: GeoModType,
    pub enabled: bool,
    pub realtime: bool,
}

/// Geo modifier stack export.
#[allow(dead_code)]
#[derive(Default)]
pub struct GeoModifierExport {
    pub modifiers: Vec<GeoModEntry>,
}

/// Create a new geo modifier export.
#[allow(dead_code)]
pub fn new_geo_modifier_export() -> GeoModifierExport {
    GeoModifierExport::default()
}

/// Add a modifier.
#[allow(dead_code)]
pub fn add_geo_modifier(
    export: &mut GeoModifierExport,
    name: &str,
    mt: GeoModType,
    enabled: bool,
    realtime: bool,
) {
    export.modifiers.push(GeoModEntry {
        name: name.to_string(),
        mod_type: mt,
        enabled,
        realtime,
    });
}

/// Count modifiers.
#[allow(dead_code)]
pub fn geo_mod_count(export: &GeoModifierExport) -> usize {
    export.modifiers.len()
}

/// Count enabled modifiers.
#[allow(dead_code)]
pub fn geo_mod_enabled_count(export: &GeoModifierExport) -> usize {
    export.modifiers.iter().filter(|m| m.enabled).count()
}

/// Find modifier by name.
#[allow(dead_code)]
pub fn find_geo_modifier<'a>(export: &'a GeoModifierExport, name: &str) -> Option<&'a GeoModEntry> {
    export.modifiers.iter().find(|m| m.name == name)
}

/// Get modifiers of a given type.
#[allow(dead_code)]
pub fn mods_of_type<'a>(export: &'a GeoModifierExport, mt: &GeoModType) -> Vec<&'a GeoModEntry> {
    export
        .modifiers
        .iter()
        .filter(|m| &m.mod_type == mt)
        .collect()
}

/// Realtime modifier count.
#[allow(dead_code)]
pub fn realtime_mod_count(export: &GeoModifierExport) -> usize {
    export.modifiers.iter().filter(|m| m.realtime).count()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn geo_modifier_to_json(export: &GeoModifierExport) -> String {
    format!(
        r#"{{"modifiers":{},"enabled":{}}}"#,
        export.modifiers.len(),
        geo_mod_enabled_count(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut e = new_geo_modifier_export();
        add_geo_modifier(&mut e, "subdiv", GeoModType::Subdivide, true, true);
        assert_eq!(geo_mod_count(&e), 1);
    }

    #[test]
    fn enabled_count() {
        let mut e = new_geo_modifier_export();
        add_geo_modifier(&mut e, "a", GeoModType::Decimate, true, true);
        add_geo_modifier(&mut e, "b", GeoModType::Mirror, false, false);
        assert_eq!(geo_mod_enabled_count(&e), 1);
    }

    #[test]
    fn find_modifier() {
        let mut e = new_geo_modifier_export();
        add_geo_modifier(&mut e, "bev", GeoModType::Bevel, true, true);
        assert!(find_geo_modifier(&e, "bev").is_some());
    }

    #[test]
    fn find_missing() {
        let e = new_geo_modifier_export();
        assert!(find_geo_modifier(&e, "x").is_none());
    }

    #[test]
    fn type_filter() {
        let mut e = new_geo_modifier_export();
        add_geo_modifier(&mut e, "a", GeoModType::Subdivide, true, true);
        add_geo_modifier(&mut e, "b", GeoModType::Decimate, true, true);
        assert_eq!(mods_of_type(&e, &GeoModType::Subdivide).len(), 1);
    }

    #[test]
    fn realtime_count() {
        let mut e = new_geo_modifier_export();
        add_geo_modifier(&mut e, "a", GeoModType::Solidify, true, true);
        add_geo_modifier(&mut e, "b", GeoModType::Bevel, true, false);
        assert_eq!(realtime_mod_count(&e), 1);
    }

    #[test]
    fn json_has_modifiers() {
        let e = new_geo_modifier_export();
        let j = geo_modifier_to_json(&e);
        assert!(j.contains("\"modifiers\":0"));
    }

    #[test]
    fn type_name() {
        assert_eq!(GeoModType::Subdivide.type_name(), "subdivide");
    }

    #[test]
    fn custom_type_name() {
        let t = GeoModType::Custom("shrinkwrap".to_string());
        assert_eq!(t.type_name(), "shrinkwrap");
    }

    #[test]
    fn empty_export() {
        let e = new_geo_modifier_export();
        assert_eq!(geo_mod_count(&e), 0);
    }
}
