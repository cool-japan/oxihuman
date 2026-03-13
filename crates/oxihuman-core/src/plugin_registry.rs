// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Plugin registration system for extensible asset loaders and target providers.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PluginKind {
    AssetLoader,
    TargetProvider,
    Exporter,
    Validator,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    /// Semver string e.g. "1.0.0".
    pub version: String,
    pub kind: PluginKind,
    pub supported_extensions: Vec<String>,
    pub description: String,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: Vec<PluginDescriptor>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin descriptor. Returns an error if a plugin with the
    /// same `id` already exists.
    pub fn register(&mut self, desc: PluginDescriptor) -> Result<(), String> {
        if self.plugins.iter().any(|p| p.id == desc.id) {
            return Err(format!(
                "plugin with id '{}' is already registered",
                desc.id
            ));
        }
        self.plugins.push(desc);
        Ok(())
    }

    /// Unregister a plugin by id. Returns `true` if it was found and removed.
    pub fn unregister(&mut self, id: &str) -> bool {
        let before = self.plugins.len();
        self.plugins.retain(|p| p.id != id);
        self.plugins.len() < before
    }

    /// Find a plugin by its unique id.
    pub fn find_by_id(&self, id: &str) -> Option<&PluginDescriptor> {
        self.plugins.iter().find(|p| p.id == id)
    }

    /// Return all plugins that declare support for the given file extension.
    pub fn find_by_extension(&self, ext: &str) -> Vec<&PluginDescriptor> {
        self.plugins
            .iter()
            .filter(|p| p.supported_extensions.iter().any(|e| e == ext))
            .collect()
    }

    /// Return all plugins of a given kind.
    pub fn find_by_kind(&self, kind: &PluginKind) -> Vec<&PluginDescriptor> {
        self.plugins.iter().filter(|p| &p.kind == kind).collect()
    }

    /// Total number of registered plugins.
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Slice of all registered plugins.
    pub fn all(&self) -> &[PluginDescriptor] {
        &self.plugins
    }

    /// Serialize the plugin list to a JSON array string.
    pub fn to_json(&self) -> String {
        let mut out = String::from("[\n");
        for (i, p) in self.plugins.iter().enumerate() {
            let kind_str = match p.kind {
                PluginKind::AssetLoader => "AssetLoader",
                PluginKind::TargetProvider => "TargetProvider",
                PluginKind::Exporter => "Exporter",
                PluginKind::Validator => "Validator",
            };
            let exts: Vec<String> = p
                .supported_extensions
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect();
            out.push_str(&format!(
                "  {{\"id\":\"{}\",\"name\":\"{}\",\"version\":\"{}\",\"kind\":\"{}\",\"extensions\":[{}],\"description\":\"{}\"}}",
                p.id, p.name, p.version, kind_str, exts.join(","), p.description
            ));
            if i + 1 < self.plugins.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push(']');
        out
    }
}

// ── built-in plugins ──────────────────────────────────────────────────────────

/// Return the list of built-in plugin descriptors shipped with OxiHuman.
#[allow(dead_code)]
pub fn default_builtin_plugins() -> Vec<PluginDescriptor> {
    vec![
        PluginDescriptor {
            id: "obj_loader".to_string(),
            name: "Wavefront OBJ Loader".to_string(),
            version: "1.0.0".to_string(),
            kind: PluginKind::AssetLoader,
            supported_extensions: vec!["obj".to_string()],
            description: "Loads Wavefront .obj mesh files".to_string(),
        },
        PluginDescriptor {
            id: "glb_loader".to_string(),
            name: "GLB Loader".to_string(),
            version: "1.0.0".to_string(),
            kind: PluginKind::AssetLoader,
            supported_extensions: vec!["glb".to_string(), "gltf".to_string()],
            description: "Loads binary or JSON glTF files".to_string(),
        },
        PluginDescriptor {
            id: "target_loader".to_string(),
            name: "MakeHuman Target Loader".to_string(),
            version: "1.0.0".to_string(),
            kind: PluginKind::TargetProvider,
            supported_extensions: vec!["target".to_string()],
            description: "Loads MakeHuman .target morph files".to_string(),
        },
        PluginDescriptor {
            id: "glb_exporter".to_string(),
            name: "GLB Exporter".to_string(),
            version: "1.0.0".to_string(),
            kind: PluginKind::Exporter,
            supported_extensions: vec!["glb".to_string()],
            description: "Exports meshes to binary glTF".to_string(),
        },
        PluginDescriptor {
            id: "ply_exporter".to_string(),
            name: "PLY Exporter".to_string(),
            version: "1.0.0".to_string(),
            kind: PluginKind::Exporter,
            supported_extensions: vec!["ply".to_string()],
            description: "Exports meshes to Stanford PLY format".to_string(),
        },
        PluginDescriptor {
            id: "pack_validator".to_string(),
            name: "Pack Validator".to_string(),
            version: "1.0.0".to_string(),
            kind: PluginKind::Validator,
            supported_extensions: vec!["toml".to_string(), "json".to_string()],
            description: "Validates OxiHuman asset pack manifests".to_string(),
        },
    ]
}

// ── semver helpers ────────────────────────────────────────────────────────────

/// Parse a semver string "major.minor.patch" into a tuple.
/// Returns `None` if the string does not match the expected format.
#[allow(dead_code)]
pub fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some((major, minor, patch))
}

/// Return `true` if version `a` is greater than or equal to version `b`.
#[allow(dead_code)]
pub fn semver_gte(a: (u32, u32, u32), b: (u32, u32, u32)) -> bool {
    a >= b
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_desc(id: &str, kind: PluginKind, exts: &[&str]) -> PluginDescriptor {
        PluginDescriptor {
            id: id.to_string(),
            name: format!("Plugin {}", id),
            version: "1.0.0".to_string(),
            kind,
            supported_extensions: exts.iter().map(|e| e.to_string()).collect(),
            description: "test".to_string(),
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let reg = PluginRegistry::new();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn register_success() {
        let mut reg = PluginRegistry::new();
        let desc = make_desc("my_loader", PluginKind::AssetLoader, &["obj"]);
        assert!(reg.register(desc).is_ok());
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn duplicate_id_is_rejected() {
        let mut reg = PluginRegistry::new();
        reg.register(make_desc("dup", PluginKind::AssetLoader, &["obj"]))
            .expect("should succeed");
        let result = reg.register(make_desc("dup", PluginKind::Exporter, &["glb"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dup"));
    }

    #[test]
    fn unregister_removes_plugin() {
        let mut reg = PluginRegistry::new();
        reg.register(make_desc("to_remove", PluginKind::Validator, &[]))
            .expect("should succeed");
        assert_eq!(reg.count(), 1);
        let removed = reg.unregister("to_remove");
        assert!(removed);
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn unregister_nonexistent_returns_false() {
        let mut reg = PluginRegistry::new();
        assert!(!reg.unregister("nope"));
    }

    #[test]
    fn find_by_id_found() {
        let mut reg = PluginRegistry::new();
        reg.register(make_desc("finder", PluginKind::AssetLoader, &["obj"]))
            .expect("should succeed");
        assert!(reg.find_by_id("finder").is_some());
    }

    #[test]
    fn find_by_id_not_found() {
        let reg = PluginRegistry::new();
        assert!(reg.find_by_id("ghost").is_none());
    }

    #[test]
    fn find_by_extension_obj() {
        let mut reg = PluginRegistry::new();
        reg.register(make_desc("obj_l", PluginKind::AssetLoader, &["obj"]))
            .expect("should succeed");
        reg.register(make_desc("glb_l", PluginKind::AssetLoader, &["glb"]))
            .expect("should succeed");
        let results = reg.find_by_extension("obj");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "obj_l");
    }

    #[test]
    fn find_by_kind_count() {
        let mut reg = PluginRegistry::new();
        reg.register(make_desc("l1", PluginKind::AssetLoader, &[]))
            .expect("should succeed");
        reg.register(make_desc("l2", PluginKind::AssetLoader, &[]))
            .expect("should succeed");
        reg.register(make_desc("e1", PluginKind::Exporter, &[]))
            .expect("should succeed");
        let loaders = reg.find_by_kind(&PluginKind::AssetLoader);
        assert_eq!(loaders.len(), 2);
    }

    #[test]
    fn count_returns_correct_value() {
        let mut reg = PluginRegistry::new();
        for i in 0..5 {
            reg.register(make_desc(&format!("p{}", i), PluginKind::Validator, &[]))
                .expect("should succeed");
        }
        assert_eq!(reg.count(), 5);
    }

    #[test]
    fn to_json_contains_id() {
        let mut reg = PluginRegistry::new();
        reg.register(make_desc(
            "json_test_plugin",
            PluginKind::Exporter,
            &["glb"],
        ))
        .expect("should succeed");
        let json = reg.to_json();
        assert!(json.contains("json_test_plugin"));
    }

    #[test]
    fn default_builtin_plugins_has_six_or_more() {
        let plugins = default_builtin_plugins();
        assert!(plugins.len() >= 6);
    }

    #[test]
    fn parse_semver_valid() {
        assert_eq!(parse_semver("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("0.0.0"), Some((0, 0, 0)));
        assert_eq!(parse_semver("10.20.30"), Some((10, 20, 30)));
    }

    #[test]
    fn parse_semver_invalid_returns_none() {
        assert_eq!(parse_semver("1.2"), None);
        assert_eq!(parse_semver("a.b.c"), None);
        assert_eq!(parse_semver(""), None);
        assert_eq!(parse_semver("1.2.3.4"), None);
    }

    #[test]
    fn semver_gte_comparisons() {
        assert!(semver_gte((1, 0, 0), (1, 0, 0)));
        assert!(semver_gte((2, 0, 0), (1, 9, 9)));
        assert!(semver_gte((1, 1, 0), (1, 0, 9)));
        assert!(!semver_gte((1, 0, 0), (1, 0, 1)));
        assert!(!semver_gte((0, 9, 9), (1, 0, 0)));
    }

    #[test]
    fn all_returns_slice_of_plugins() {
        let mut reg = PluginRegistry::new();
        reg.register(make_desc("a1", PluginKind::AssetLoader, &[]))
            .expect("should succeed");
        reg.register(make_desc("a2", PluginKind::AssetLoader, &[]))
            .expect("should succeed");
        assert_eq!(reg.all().len(), 2);
    }
}
