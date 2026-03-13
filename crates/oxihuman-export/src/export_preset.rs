//! Save and load export presets for repeated export configurations.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum ExportTarget {
    Glb,
    Obj,
    Fbx,
    Usdz,
    Ply,
    Stl,
    Collada,
    Custom(String),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ExportPreset {
    pub name: String,
    pub target: ExportTarget,
    pub include_normals: bool,
    pub include_uvs: bool,
    pub include_animations: bool,
    pub include_materials: bool,
    pub scale: f32,
    pub up_axis: String,
    pub custom_options: Vec<(String, String)>,
}

#[allow(dead_code)]
pub struct PresetLibraryExport {
    pub presets: Vec<ExportPreset>,
    pub default_preset: Option<String>,
}

#[allow(dead_code)]
pub fn default_glb_preset() -> ExportPreset {
    ExportPreset {
        name: "Default GLB".to_string(),
        target: ExportTarget::Glb,
        include_normals: true,
        include_uvs: true,
        include_animations: true,
        include_materials: true,
        scale: 1.0,
        up_axis: "Y".to_string(),
        custom_options: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn default_obj_preset() -> ExportPreset {
    ExportPreset {
        name: "Default OBJ".to_string(),
        target: ExportTarget::Obj,
        include_normals: true,
        include_uvs: true,
        include_animations: false,
        include_materials: true,
        scale: 1.0,
        up_axis: "Y".to_string(),
        custom_options: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn new_preset_library() -> PresetLibraryExport {
    PresetLibraryExport {
        presets: Vec::new(),
        default_preset: None,
    }
}

#[allow(dead_code)]
pub fn add_preset(lib: &mut PresetLibraryExport, preset: ExportPreset) {
    lib.presets.push(preset);
}

#[allow(dead_code)]
pub fn get_preset_by_name<'a>(
    lib: &'a PresetLibraryExport,
    name: &str,
) -> Option<&'a ExportPreset> {
    lib.presets.iter().find(|p| p.name == name)
}

#[allow(dead_code)]
pub fn remove_preset(lib: &mut PresetLibraryExport, name: &str) -> bool {
    let before = lib.presets.len();
    lib.presets.retain(|p| p.name != name);
    let removed = lib.presets.len() < before;
    if removed {
        if let Some(ref default) = lib.default_preset.clone() {
            if default == name {
                lib.default_preset = None;
            }
        }
    }
    removed
}

#[allow(dead_code)]
pub fn set_default_preset(lib: &mut PresetLibraryExport, name: &str) {
    lib.default_preset = Some(name.to_string());
}

#[allow(dead_code)]
pub fn preset_count(lib: &PresetLibraryExport) -> usize {
    lib.presets.len()
}

#[allow(dead_code)]
pub fn presets_for_target<'a>(
    lib: &'a PresetLibraryExport,
    target: &ExportTarget,
) -> Vec<&'a ExportPreset> {
    lib.presets.iter().filter(|p| &p.target == target).collect()
}

#[allow(dead_code)]
pub fn preset_to_json(preset: &ExportPreset) -> String {
    let target_str = match &preset.target {
        ExportTarget::Glb => "glb",
        ExportTarget::Obj => "obj",
        ExportTarget::Fbx => "fbx",
        ExportTarget::Usdz => "usdz",
        ExportTarget::Ply => "ply",
        ExportTarget::Stl => "stl",
        ExportTarget::Collada => "collada",
        ExportTarget::Custom(s) => s.as_str(),
    };
    let custom_opts: Vec<String> = preset
        .custom_options
        .iter()
        .map(|(k, v)| format!("{{\"key\":\"{k}\",\"value\":\"{v}\"}}"))
        .collect();
    format!(
        "{{\"name\":\"{}\",\"target\":\"{}\",\"include_normals\":{},\"include_uvs\":{},\
         \"include_animations\":{},\"include_materials\":{},\"scale\":{},\"up_axis\":\"{}\",\
         \"custom_options\":[{}]}}",
        preset.name,
        target_str,
        preset.include_normals,
        preset.include_uvs,
        preset.include_animations,
        preset.include_materials,
        preset.scale,
        preset.up_axis,
        custom_opts.join(",")
    )
}

#[allow(dead_code)]
pub fn preset_library_to_json(lib: &PresetLibraryExport) -> String {
    let presets_json: Vec<String> = lib.presets.iter().map(preset_to_json).collect();
    let default_str = match &lib.default_preset {
        Some(name) => format!("\"{name}\""),
        None => "null".to_string(),
    };
    format!(
        "{{\"presets\":[{}],\"default_preset\":{}}}",
        presets_json.join(","),
        default_str
    )
}

#[allow(dead_code)]
pub fn clone_preset(preset: &ExportPreset, new_name: &str) -> ExportPreset {
    let mut cloned = preset.clone();
    cloned.name = new_name.to_string();
    cloned
}

#[allow(dead_code)]
pub fn add_custom_option(preset: &mut ExportPreset, key: &str, value: &str) {
    preset
        .custom_options
        .push((key.to_string(), value.to_string()));
}

#[allow(dead_code)]
pub fn target_extension(target: &ExportTarget) -> &'static str {
    match target {
        ExportTarget::Glb => ".glb",
        ExportTarget::Obj => ".obj",
        ExportTarget::Fbx => ".fbx",
        ExportTarget::Usdz => ".usdz",
        ExportTarget::Ply => ".ply",
        ExportTarget::Stl => ".stl",
        ExportTarget::Collada => ".dae",
        ExportTarget::Custom(_) => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_glb_preset() {
        let p = default_glb_preset();
        assert_eq!(p.target, ExportTarget::Glb);
        assert!(p.include_normals);
        assert!(p.include_animations);
        assert!((p.scale - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_default_obj_preset() {
        let p = default_obj_preset();
        assert_eq!(p.target, ExportTarget::Obj);
        assert!(!p.include_animations);
        assert!(p.include_uvs);
    }

    #[test]
    fn test_new_library() {
        let lib = new_preset_library();
        assert!(lib.presets.is_empty());
        assert!(lib.default_preset.is_none());
    }

    #[test]
    fn test_add_preset() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, default_glb_preset());
        assert_eq!(preset_count(&lib), 1);
    }

    #[test]
    fn test_get_preset_by_name() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, default_glb_preset());
        let found = get_preset_by_name(&lib, "Default GLB");
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").target, ExportTarget::Glb);
    }

    #[test]
    fn test_get_preset_by_name_not_found() {
        let lib = new_preset_library();
        assert!(get_preset_by_name(&lib, "Missing").is_none());
    }

    #[test]
    fn test_remove_preset() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, default_glb_preset());
        let removed = remove_preset(&mut lib, "Default GLB");
        assert!(removed);
        assert_eq!(preset_count(&lib), 0);
    }

    #[test]
    fn test_remove_preset_not_found() {
        let mut lib = new_preset_library();
        let removed = remove_preset(&mut lib, "Nonexistent");
        assert!(!removed);
    }

    #[test]
    fn test_set_default_preset() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, default_glb_preset());
        set_default_preset(&mut lib, "Default GLB");
        assert_eq!(lib.default_preset.as_deref(), Some("Default GLB"));
    }

    #[test]
    fn test_preset_count() {
        let mut lib = new_preset_library();
        assert_eq!(preset_count(&lib), 0);
        add_preset(&mut lib, default_glb_preset());
        add_preset(&mut lib, default_obj_preset());
        assert_eq!(preset_count(&lib), 2);
    }

    #[test]
    fn test_presets_for_target() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, default_glb_preset());
        add_preset(&mut lib, default_obj_preset());
        let glb_presets = presets_for_target(&lib, &ExportTarget::Glb);
        assert_eq!(glb_presets.len(), 1);
        assert_eq!(glb_presets[0].target, ExportTarget::Glb);
    }

    #[test]
    fn test_preset_to_json_non_empty() {
        let p = default_glb_preset();
        let json = preset_to_json(&p);
        assert!(!json.is_empty());
        assert!(json.contains("glb"));
        assert!(json.contains("Default GLB"));
    }

    #[test]
    fn test_preset_library_to_json() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, default_glb_preset());
        let json = preset_library_to_json(&lib);
        assert!(!json.is_empty());
        assert!(json.contains("presets"));
    }

    #[test]
    fn test_clone_preset() {
        let p = default_glb_preset();
        let cloned = clone_preset(&p, "My GLB Clone");
        assert_eq!(cloned.name, "My GLB Clone");
        assert_eq!(cloned.target, ExportTarget::Glb);
    }

    #[test]
    fn test_add_custom_option() {
        let mut p = default_glb_preset();
        add_custom_option(&mut p, "compress", "true");
        assert_eq!(p.custom_options.len(), 1);
        assert_eq!(p.custom_options[0].0, "compress");
        assert_eq!(p.custom_options[0].1, "true");
    }

    #[test]
    fn test_target_extension() {
        assert_eq!(target_extension(&ExportTarget::Glb), ".glb");
        assert_eq!(target_extension(&ExportTarget::Obj), ".obj");
        assert_eq!(target_extension(&ExportTarget::Fbx), ".fbx");
        assert_eq!(target_extension(&ExportTarget::Usdz), ".usdz");
        assert_eq!(target_extension(&ExportTarget::Ply), ".ply");
        assert_eq!(target_extension(&ExportTarget::Stl), ".stl");
        assert_eq!(target_extension(&ExportTarget::Collada), ".dae");
        assert_eq!(
            target_extension(&ExportTarget::Custom("abc".to_string())),
            ""
        );
    }

    #[test]
    fn test_remove_default_preset_clears_default() {
        let mut lib = new_preset_library();
        add_preset(&mut lib, default_glb_preset());
        set_default_preset(&mut lib, "Default GLB");
        remove_preset(&mut lib, "Default GLB");
        assert!(lib.default_preset.is_none());
    }
}
