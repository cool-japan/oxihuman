// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Multi-material library export (MTL/JSON format).

// ── Enums ─────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialLibFormat {
    Mtl,
    Json,
    Xml,
}

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MatLibConfig {
    pub format: MaterialLibFormat,
    pub embed_textures: bool,
    pub normalize_values: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialEntry {
    pub name: String,
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    pub opacity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialLibrary {
    pub materials: Vec<MaterialEntry>,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MatLibExportResult {
    pub data_string: String,
    pub material_count: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_mat_lib_config() -> MatLibConfig {
    MatLibConfig {
        format: MaterialLibFormat::Json,
        embed_textures: false,
        normalize_values: true,
    }
}

#[allow(dead_code)]
pub fn new_material_library(name: &str) -> MaterialLibrary {
    MaterialLibrary {
        materials: Vec::new(),
        name: name.to_string(),
    }
}

#[allow(dead_code)]
pub fn add_material(lib: &mut MaterialLibrary, mat: MaterialEntry) {
    lib.materials.push(mat);
}

#[allow(dead_code)]
pub fn new_material_entry(name: &str) -> MaterialEntry {
    MaterialEntry {
        name: name.to_string(),
        diffuse: [0.8, 0.8, 0.8],
        specular: [0.5, 0.5, 0.5],
        roughness: 0.5,
        metallic: 0.0,
        opacity: 1.0,
    }
}

#[allow(dead_code)]
pub fn export_material_library(
    lib: &MaterialLibrary,
    cfg: &MatLibConfig,
) -> MatLibExportResult {
    let data_string = match cfg.format {
        MaterialLibFormat::Json => export_as_json(lib),
        MaterialLibFormat::Mtl => export_as_mtl(lib),
        MaterialLibFormat::Xml => export_as_xml(lib),
    };
    MatLibExportResult {
        material_count: lib.materials.len(),
        data_string,
    }
}

fn export_as_json(lib: &MaterialLibrary) -> String {
    let mats: Vec<String> = lib
        .materials
        .iter()
        .map(|m| {
            format!(
                r#"{{"name":"{}","diffuse":[{},{},{}],"specular":[{},{},{}],"roughness":{},"metallic":{},"opacity":{}}}"#,
                m.name,
                m.diffuse[0], m.diffuse[1], m.diffuse[2],
                m.specular[0], m.specular[1], m.specular[2],
                m.roughness, m.metallic, m.opacity
            )
        })
        .collect();
    format!(r#"{{"library":"{}","materials":[{}]}}"#, lib.name, mats.join(","))
}

fn export_as_mtl(lib: &MaterialLibrary) -> String {
    let mut out = format!("# Material Library: {}\n", lib.name);
    for m in &lib.materials {
        out.push_str(&format!(
            "\nnewmtl {}\nKd {} {} {}\nKs {} {} {}\nNs {}\nd {}\n",
            m.name,
            m.diffuse[0], m.diffuse[1], m.diffuse[2],
            m.specular[0], m.specular[1], m.specular[2],
            (1.0 - m.roughness) * 100.0,
            m.opacity
        ));
    }
    out
}

fn export_as_xml(lib: &MaterialLibrary) -> String {
    let mut out = format!("<library name=\"{}\">\n", lib.name);
    for m in &lib.materials {
        out.push_str(&format!(
            "  <material name=\"{}\" roughness=\"{}\" metallic=\"{}\" opacity=\"{}\"/>\n",
            m.name, m.roughness, m.metallic, m.opacity
        ));
    }
    out.push_str("</library>\n");
    out
}

#[allow(dead_code)]
pub fn material_count_lib(lib: &MaterialLibrary) -> usize {
    lib.materials.len()
}

#[allow(dead_code)]
pub fn find_material<'a>(lib: &'a MaterialLibrary, name: &str) -> Option<&'a MaterialEntry> {
    lib.materials.iter().find(|m| m.name == name)
}

#[allow(dead_code)]
pub fn mat_lib_format_name(cfg: &MatLibConfig) -> &'static str {
    match cfg.format {
        MaterialLibFormat::Mtl => "mtl",
        MaterialLibFormat::Json => "json",
        MaterialLibFormat::Xml => "xml",
    }
}

#[allow(dead_code)]
pub fn mat_lib_result_to_json(r: &MatLibExportResult) -> String {
    let escaped = r.data_string.replace('"', "\\\"").replace('\n', "\\n");
    format!(
        r#"{{"material_count":{},"data":"{}"}}"#,
        r.material_count, escaped
    )
}

#[allow(dead_code)]
pub fn validate_material_library(lib: &MaterialLibrary) -> bool {
    if lib.name.is_empty() {
        return false;
    }
    for m in &lib.materials {
        if m.name.is_empty() {
            return false;
        }
        if !(0.0..=1.0).contains(&m.roughness)
            || !(0.0..=1.0).contains(&m.metallic)
            || !(0.0..=1.0).contains(&m.opacity)
        {
            return false;
        }
    }
    true
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_json() {
        let cfg = default_mat_lib_config();
        assert_eq!(cfg.format, MaterialLibFormat::Json);
        assert!(!cfg.embed_textures);
        assert!(cfg.normalize_values);
    }

    #[test]
    fn new_library_is_empty() {
        let lib = new_material_library("skin");
        assert_eq!(lib.name, "skin");
        assert!(lib.materials.is_empty());
    }

    #[test]
    fn add_material_appends() {
        let mut lib = new_material_library("test");
        add_material(&mut lib, new_material_entry("metal"));
        assert_eq!(material_count_lib(&lib), 1);
    }

    #[test]
    fn find_material_returns_correct() {
        let mut lib = new_material_library("lib");
        add_material(&mut lib, new_material_entry("gold"));
        add_material(&mut lib, new_material_entry("silver"));
        let m = find_material(&lib, "gold").expect("should find gold");
        assert_eq!(m.name, "gold");
        assert!(find_material(&lib, "bronze").is_none());
    }

    #[test]
    fn export_json_contains_name() {
        let cfg = default_mat_lib_config();
        let mut lib = new_material_library("mylib");
        add_material(&mut lib, new_material_entry("plastic"));
        let result = export_material_library(&lib, &cfg);
        assert!(result.data_string.contains("plastic"));
        assert_eq!(result.material_count, 1);
    }

    #[test]
    fn export_mtl_format() {
        let cfg = MatLibConfig {
            format: MaterialLibFormat::Mtl,
            embed_textures: false,
            normalize_values: true,
        };
        let mut lib = new_material_library("mylib");
        add_material(&mut lib, new_material_entry("wood"));
        let result = export_material_library(&lib, &cfg);
        assert!(result.data_string.contains("newmtl wood"));
    }

    #[test]
    fn validate_valid_library() {
        let mut lib = new_material_library("valid");
        add_material(&mut lib, new_material_entry("mat1"));
        assert!(validate_material_library(&lib));
    }

    #[test]
    fn validate_empty_name_fails() {
        let lib = new_material_library("");
        assert!(!validate_material_library(&lib));
    }

    #[test]
    fn mat_lib_format_name_correct() {
        assert_eq!(mat_lib_format_name(&default_mat_lib_config()), "json");
        let cfg = MatLibConfig {
            format: MaterialLibFormat::Mtl,
            embed_textures: false,
            normalize_values: false,
        };
        assert_eq!(mat_lib_format_name(&cfg), "mtl");
    }

    #[test]
    fn mat_lib_result_to_json_contains_count() {
        let r = MatLibExportResult {
            data_string: "{}".to_string(),
            material_count: 5,
        };
        let json = mat_lib_result_to_json(&r);
        assert!(json.contains("5"));
        assert!(json.contains("material_count"));
    }
}
