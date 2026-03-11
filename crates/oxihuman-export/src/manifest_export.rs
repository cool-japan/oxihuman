// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Software manifest (name, version, deps) export.

/// A dependency entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
}

/// A software manifest.
#[allow(dead_code)]
pub struct SoftwareManifest {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub license: String,
    pub description: String,
    pub dependencies: Vec<Dependency>,
}

impl SoftwareManifest {
    #[allow(dead_code)]
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            authors: Vec::new(),
            license: String::new(),
            description: String::new(),
            dependencies: Vec::new(),
        }
    }
}

/// Add an author.
#[allow(dead_code)]
pub fn add_author(manifest: &mut SoftwareManifest, author: &str) {
    manifest.authors.push(author.to_string());
}

/// Add a dependency.
#[allow(dead_code)]
pub fn add_dependency(manifest: &mut SoftwareManifest, name: &str, version: &str, optional: bool) {
    manifest.dependencies.push(Dependency {
        name: name.to_string(),
        version: version.to_string(),
        optional,
    });
}

/// Set license string.
#[allow(dead_code)]
pub fn set_license(manifest: &mut SoftwareManifest, license: &str) {
    manifest.license = license.to_string();
}

/// Set description.
#[allow(dead_code)]
pub fn set_description(manifest: &mut SoftwareManifest, desc: &str) {
    manifest.description = desc.to_string();
}

/// Export as TOML-like string.
#[allow(dead_code)]
pub fn export_manifest_toml(manifest: &SoftwareManifest) -> String {
    let mut out = format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\nlicense = \"{}\"\ndescription = \"{}\"\n",
        manifest.name, manifest.version, manifest.license, manifest.description
    );
    if !manifest.authors.is_empty() {
        out.push_str(&format!(
            "authors = [{}]\n",
            manifest
                .authors
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !manifest.dependencies.is_empty() {
        out.push_str("\n[dependencies]\n");
        for dep in &manifest.dependencies {
            if dep.optional {
                out.push_str(&format!(
                    "{} = {{ version = \"{}\", optional = true }}\n",
                    dep.name, dep.version
                ));
            } else {
                out.push_str(&format!("{} = \"{}\"\n", dep.name, dep.version));
            }
        }
    }
    out
}

/// Export as JSON-like string.
#[allow(dead_code)]
pub fn export_manifest_json(manifest: &SoftwareManifest) -> String {
    let authors_json = manifest
        .authors
        .iter()
        .map(|a| format!("\"{}\"", a))
        .collect::<Vec<_>>()
        .join(",");
    let deps_json = manifest
        .dependencies
        .iter()
        .map(|d| {
            format!(
                "{{\"name\":\"{}\",\"version\":\"{}\",\"optional\":{}}}",
                d.name, d.version, d.optional
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"name\":\"{}\",\"version\":\"{}\",\"license\":\"{}\",\
        \"description\":\"{}\",\"authors\":[{}],\"dependencies\":[{}]}}",
        manifest.name,
        manifest.version,
        manifest.license,
        manifest.description,
        authors_json,
        deps_json
    )
}

/// Dependency count.
#[allow(dead_code)]
pub fn dependency_count(manifest: &SoftwareManifest) -> usize {
    manifest.dependencies.len()
}

/// Optional dependency count.
#[allow(dead_code)]
pub fn optional_dependency_count(manifest: &SoftwareManifest) -> usize {
    manifest.dependencies.iter().filter(|d| d.optional).count()
}

/// Find a dependency by name.
#[allow(dead_code)]
pub fn find_dependency<'a>(manifest: &'a SoftwareManifest, name: &str) -> Option<&'a Dependency> {
    manifest.dependencies.iter().find(|d| d.name == name)
}

/// Author count.
#[allow(dead_code)]
pub fn author_count(manifest: &SoftwareManifest) -> usize {
    manifest.authors.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> SoftwareManifest {
        let mut m = SoftwareManifest::new("oxihuman", "1.0.0");
        set_license(&mut m, "MIT OR Apache-2.0");
        set_description(&mut m, "Human body mesh toolkit");
        add_author(&mut m, "Team KitaSan");
        add_dependency(&mut m, "glam", "0.27", false);
        add_dependency(&mut m, "serde", "1.0", true);
        add_dependency(&mut m, "bytemuck", "1.14", false);
        m
    }

    #[test]
    fn dependency_count_correct() {
        let m = sample_manifest();
        assert_eq!(dependency_count(&m), 3);
    }

    #[test]
    fn optional_dependency_count_correct() {
        let m = sample_manifest();
        assert_eq!(optional_dependency_count(&m), 1);
    }

    #[test]
    fn author_count_correct() {
        let m = sample_manifest();
        assert_eq!(author_count(&m), 1);
    }

    #[test]
    fn toml_contains_package_header() {
        let m = sample_manifest();
        let toml = export_manifest_toml(&m);
        assert!(toml.contains("[package]"));
    }

    #[test]
    fn toml_contains_version() {
        let m = sample_manifest();
        let toml = export_manifest_toml(&m);
        assert!(toml.contains("1.0.0"));
    }

    #[test]
    fn toml_contains_dependencies_header() {
        let m = sample_manifest();
        let toml = export_manifest_toml(&m);
        assert!(toml.contains("[dependencies]"));
    }

    #[test]
    fn json_contains_name() {
        let m = sample_manifest();
        let json = export_manifest_json(&m);
        assert!(json.contains("oxihuman"));
    }

    #[test]
    fn find_dependency_some() {
        let m = sample_manifest();
        assert!(find_dependency(&m, "glam").is_some());
    }

    #[test]
    fn find_dependency_none() {
        let m = sample_manifest();
        assert!(find_dependency(&m, "tokio").is_none());
    }

    #[test]
    fn toml_optional_marker_present() {
        let m = sample_manifest();
        let toml = export_manifest_toml(&m);
        assert!(toml.contains("optional = true"));
    }
}
