// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GLSL/WGSL shader library manager.
//!
//! Manages shader source strings, variants, and compilation metadata without
//! performing actual GPU compilation.

// ── Types ─────────────────────────────────────────────────────────────────────

/// The pipeline stage a shader targets.
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

/// The shading language and version for a shader.
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderLanguage {
    Glsl { version: u32 },
    Wgsl,
    Hlsl { shader_model: String },
}

/// A named set of preprocessor defines that creates a variant of a shader.
#[derive(Debug, Clone)]
pub struct ShaderVariant {
    pub defines: Vec<(String, String)>,
    pub name: String,
}

/// A single shader entry stored in the library.
#[derive(Debug, Clone)]
pub struct ShaderEntry {
    pub name: String,
    pub stage: ShaderStage,
    pub language: ShaderLanguage,
    pub source: String,
    pub variants: Vec<ShaderVariant>,
    pub includes: Vec<String>,
    pub entry_point: String,
}

/// A collection of shader entries and an include-source cache.
#[derive(Debug, Clone)]
pub struct ShaderLibrary {
    pub shaders: std::collections::HashMap<String, ShaderEntry>,
    pub include_cache: std::collections::HashMap<String, String>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Create a new empty `ShaderLibrary`.
#[allow(dead_code)]
pub fn new_shader_library() -> ShaderLibrary {
    ShaderLibrary {
        shaders: std::collections::HashMap::new(),
        include_cache: std::collections::HashMap::new(),
    }
}

/// Register a shader entry in the library.  Overwrites any existing entry with the same name.
#[allow(dead_code)]
pub fn register_shader(lib: &mut ShaderLibrary, entry: ShaderEntry) {
    lib.shaders.insert(entry.name.clone(), entry);
}

/// Look up a shader by name.
#[allow(dead_code)]
pub fn get_shader<'a>(lib: &'a ShaderLibrary, name: &str) -> Option<&'a ShaderEntry> {
    lib.shaders.get(name)
}

/// Resolve `#include "name"` directives in `source` by substituting the cached content.
/// Includes that are not in the cache are left as-is with a `// MISSING:` comment.
#[allow(dead_code)]
pub fn resolve_includes(lib: &ShaderLibrary, source: &str) -> String {
    let mut result = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("#include") {
            let rest = rest.trim();
            // Extract the name between quotes.
            if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
                let inc_name = &rest[1..rest.len() - 1];
                if let Some(content) = lib.include_cache.get(inc_name) {
                    result.push_str(content);
                    result.push('\n');
                    continue;
                } else {
                    result.push_str(&format!("// MISSING: {}\n", inc_name));
                    continue;
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Prepend `#define` statements from a variant to the shader source.
#[allow(dead_code)]
pub fn compile_variant(entry: &ShaderEntry, variant: &ShaderVariant) -> String {
    let mut out = String::new();
    for (name, value) in &variant.defines {
        out.push_str(&format!("#define {} {}\n", name, value));
    }
    out.push_str(&entry.source);
    out
}

/// Register an include source snippet under the given name.
#[allow(dead_code)]
pub fn register_include(lib: &mut ShaderLibrary, name: &str, source: &str) {
    lib.include_cache
        .insert(name.to_string(), source.to_string());
}

/// Return all shader names in alphabetical order.
#[allow(dead_code)]
pub fn list_shaders(lib: &ShaderLibrary) -> Vec<&str> {
    let mut names: Vec<&str> = lib.shaders.keys().map(|s| s.as_str()).collect();
    names.sort_unstable();
    names
}

/// Create a library pre-populated with a minimal PBR vertex and fragment shader pair.
#[allow(dead_code)]
pub fn default_pbr_shaders() -> ShaderLibrary {
    let mut lib = new_shader_library();

    register_shader(
        &mut lib,
        ShaderEntry {
            name: "pbr_vertex".to_string(),
            stage: ShaderStage::Vertex,
            language: ShaderLanguage::Glsl { version: 300 },
            source: concat!(
                "#version 300 es\n",
                "in vec3 a_position;\n",
                "in vec3 a_normal;\n",
                "in vec2 a_uv;\n",
                "uniform mat4 u_mvp;\n",
                "out vec3 v_normal;\n",
                "out vec2 v_uv;\n",
                "void main() {\n",
                "    v_normal = a_normal;\n",
                "    v_uv = a_uv;\n",
                "    gl_Position = u_mvp * vec4(a_position, 1.0);\n",
                "}\n",
            )
            .to_string(),
            variants: Vec::new(),
            includes: Vec::new(),
            entry_point: "main".to_string(),
        },
    );

    register_shader(
        &mut lib,
        ShaderEntry {
            name: "pbr_fragment".to_string(),
            stage: ShaderStage::Fragment,
            language: ShaderLanguage::Glsl { version: 300 },
            source: concat!(
                "#version 300 es\n",
                "precision highp float;\n",
                "in vec3 v_normal;\n",
                "in vec2 v_uv;\n",
                "uniform vec4 u_base_color;\n",
                "uniform float u_metallic;\n",
                "uniform float u_roughness;\n",
                "out vec4 frag_color;\n",
                "void main() {\n",
                "    vec3 n = normalize(v_normal);\n",
                "    float NdotL = max(dot(n, vec3(0.0,1.0,0.0)), 0.0);\n",
                "    frag_color = u_base_color * (0.3 + 0.7 * NdotL);\n",
                "}\n",
            )
            .to_string(),
            variants: Vec::new(),
            includes: Vec::new(),
            entry_point: "main".to_string(),
        },
    );

    lib
}

/// Compute a simple djb2 hash of the shader source string.
#[allow(dead_code)]
pub fn shader_hash(entry: &ShaderEntry) -> u64 {
    let mut hash: u64 = 5381;
    for b in entry.source.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    hash
}

/// Validate a `ShaderEntry` and return a list of issue descriptions.
#[allow(dead_code)]
pub fn validate_shader_entry(entry: &ShaderEntry) -> Vec<String> {
    let mut issues = Vec::new();
    if entry.name.is_empty() {
        issues.push("shader name is empty".to_string());
    }
    if entry.source.is_empty() {
        issues.push("shader source is empty".to_string());
    }
    if entry.entry_point.is_empty() {
        issues.push("entry point is empty".to_string());
    }
    // Check that GLSL sources start with #version
    if let ShaderLanguage::Glsl { .. } = &entry.language {
        if !entry.source.contains("#version") {
            issues.push("GLSL shader source missing #version directive".to_string());
        }
    }
    issues
}

/// Remove a shader from the library by name.  Returns `true` if it existed.
#[allow(dead_code)]
pub fn remove_shader(lib: &mut ShaderLibrary, name: &str) -> bool {
    lib.shaders.remove(name).is_some()
}

/// Return the number of shaders currently registered.
#[allow(dead_code)]
pub fn shader_count(lib: &ShaderLibrary) -> usize {
    lib.shaders.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(name: &str, stage: ShaderStage) -> ShaderEntry {
        ShaderEntry {
            name: name.to_string(),
            stage,
            language: ShaderLanguage::Glsl { version: 300 },
            source: "#version 300 es\nvoid main() {}\n".to_string(),
            variants: Vec::new(),
            includes: Vec::new(),
            entry_point: "main".to_string(),
        }
    }

    #[test]
    fn new_shader_library_is_empty() {
        let lib = new_shader_library();
        assert!(lib.shaders.is_empty());
        assert!(lib.include_cache.is_empty());
    }

    #[test]
    fn register_and_get_shader() {
        let mut lib = new_shader_library();
        register_shader(&mut lib, sample_entry("vert", ShaderStage::Vertex));
        let entry = get_shader(&lib, "vert");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().name, "vert");
    }

    #[test]
    fn get_shader_missing_returns_none() {
        let lib = new_shader_library();
        assert!(get_shader(&lib, "nonexistent").is_none());
    }

    #[test]
    fn resolve_includes_substitutes_cached() {
        let mut lib = new_shader_library();
        register_include(&mut lib, "common.glsl", "// common code\n");
        let source = "#include \"common.glsl\"\nvoid main() {}\n";
        let resolved = resolve_includes(&lib, source);
        assert!(
            resolved.contains("// common code"),
            "resolved: {}",
            resolved
        );
        assert!(!resolved.contains("#include"));
    }

    #[test]
    fn resolve_includes_missing_leaves_comment() {
        let lib = new_shader_library();
        let source = "#include \"missing.glsl\"\nvoid main() {}\n";
        let resolved = resolve_includes(&lib, source);
        assert!(resolved.contains("// MISSING:"), "resolved: {}", resolved);
    }

    #[test]
    fn compile_variant_prepends_defines() {
        let entry = sample_entry("shader", ShaderStage::Fragment);
        let variant = ShaderVariant {
            name: "skinned".to_string(),
            defines: vec![("SKINNING".to_string(), "1".to_string())],
        };
        let out = compile_variant(&entry, &variant);
        assert!(
            out.starts_with("#define SKINNING 1\n"),
            "got: {}",
            &out[..40]
        );
        assert!(out.contains("void main()"));
    }

    #[test]
    fn list_shaders_sorted() {
        let mut lib = new_shader_library();
        register_shader(&mut lib, sample_entry("z_vert", ShaderStage::Vertex));
        register_shader(&mut lib, sample_entry("a_frag", ShaderStage::Fragment));
        let names = list_shaders(&lib);
        assert_eq!(names, vec!["a_frag", "z_vert"]);
    }

    #[test]
    fn default_pbr_shaders_contains_vertex_and_fragment() {
        let lib = default_pbr_shaders();
        assert!(get_shader(&lib, "pbr_vertex").is_some());
        assert!(get_shader(&lib, "pbr_fragment").is_some());
    }

    #[test]
    fn shader_hash_nonzero() {
        let entry = sample_entry("test", ShaderStage::Vertex);
        let h = shader_hash(&entry);
        assert_ne!(h, 0);
    }

    #[test]
    fn shader_hash_different_sources_differ() {
        let mut e1 = sample_entry("a", ShaderStage::Vertex);
        let mut e2 = sample_entry("a", ShaderStage::Vertex);
        e1.source = "#version 300 es\nvoid main() { gl_Position = vec4(0); }\n".to_string();
        e2.source = "#version 300 es\nvoid main() { gl_Position = vec4(1); }\n".to_string();
        assert_ne!(shader_hash(&e1), shader_hash(&e2));
    }

    #[test]
    fn validate_shader_entry_valid() {
        let entry = sample_entry("v", ShaderStage::Vertex);
        let issues = validate_shader_entry(&entry);
        assert!(
            issues.is_empty(),
            "valid entry should have no issues: {:?}",
            issues
        );
    }

    #[test]
    fn validate_shader_entry_empty_name() {
        let mut entry = sample_entry("", ShaderStage::Vertex);
        entry.name = String::new();
        let issues = validate_shader_entry(&entry);
        assert!(issues.iter().any(|i| i.contains("name is empty")));
    }

    #[test]
    fn validate_shader_entry_empty_source() {
        let mut entry = sample_entry("v", ShaderStage::Vertex);
        entry.source = String::new();
        let issues = validate_shader_entry(&entry);
        assert!(issues.iter().any(|i| i.contains("source is empty")));
    }

    #[test]
    fn remove_shader_returns_true_when_exists() {
        let mut lib = new_shader_library();
        register_shader(&mut lib, sample_entry("s", ShaderStage::Fragment));
        assert!(remove_shader(&mut lib, "s"));
        assert!(get_shader(&lib, "s").is_none());
    }

    #[test]
    fn remove_shader_returns_false_when_missing() {
        let mut lib = new_shader_library();
        assert!(!remove_shader(&mut lib, "ghost"));
    }

    #[test]
    fn shader_count_tracks_additions_and_removals() {
        let mut lib = new_shader_library();
        assert_eq!(shader_count(&lib), 0);
        register_shader(&mut lib, sample_entry("a", ShaderStage::Vertex));
        register_shader(&mut lib, sample_entry("b", ShaderStage::Fragment));
        assert_eq!(shader_count(&lib), 2);
        remove_shader(&mut lib, "a");
        assert_eq!(shader_count(&lib), 1);
    }
}
