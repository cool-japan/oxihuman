#![allow(dead_code)]
//! Material texture export.

/// Material texture export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialTextureExport {
    pub slots: Vec<TextureSlot>,
}

/// A texture slot.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TextureSlot {
    pub name: String,
    pub texture_path: String,
    pub uv_channel: u32,
}

/// Export material textures.
#[allow(dead_code)]
pub fn export_material_textures(slots: Vec<TextureSlot>) -> MaterialTextureExport {
    MaterialTextureExport { slots }
}

/// Get slot count.
#[allow(dead_code)]
pub fn texture_slot_count(e: &MaterialTextureExport) -> usize {
    e.slots.len()
}

/// Get slot name.
#[allow(dead_code)]
pub fn slot_name(e: &MaterialTextureExport, index: usize) -> &str {
    if index < e.slots.len() {
        &e.slots[index].name
    } else {
        ""
    }
}

/// Get slot texture path.
#[allow(dead_code)]
pub fn slot_texture_path(e: &MaterialTextureExport, index: usize) -> &str {
    if index < e.slots.len() {
        &e.slots[index].texture_path
    } else {
        ""
    }
}

/// Serialize slot to JSON.
#[allow(dead_code)]
pub fn slot_to_json(e: &MaterialTextureExport, index: usize) -> String {
    if index < e.slots.len() {
        let s = &e.slots[index];
        format!(
            "{{\"name\":\"{}\",\"path\":\"{}\",\"uv\":{}}}",
            s.name, s.texture_path, s.uv_channel
        )
    } else {
        "{}".to_string()
    }
}

/// Get UV channel for slot.
#[allow(dead_code)]
pub fn slot_uv_channel(e: &MaterialTextureExport, index: usize) -> u32 {
    if index < e.slots.len() {
        e.slots[index].uv_channel
    } else {
        0
    }
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn texture_export_size_mt(e: &MaterialTextureExport) -> usize {
    e.slots
        .iter()
        .map(|s| s.name.len() + s.texture_path.len() + 4)
        .sum()
}

/// Validate material textures.
#[allow(dead_code)]
pub fn validate_material_textures(e: &MaterialTextureExport) -> bool {
    e.slots
        .iter()
        .all(|s| !s.name.is_empty() && !s.texture_path.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_slot(name: &str, path: &str, uv: u32) -> TextureSlot {
        TextureSlot {
            name: name.to_string(),
            texture_path: path.to_string(),
            uv_channel: uv,
        }
    }

    #[test]
    fn test_export_material_textures() {
        let e = export_material_textures(vec![make_slot("diffuse", "tex.png", 0)]);
        assert_eq!(e.slots.len(), 1);
    }

    #[test]
    fn test_texture_slot_count() {
        let e = export_material_textures(vec![make_slot("a", "b", 0)]);
        assert_eq!(texture_slot_count(&e), 1);
    }

    #[test]
    fn test_slot_name() {
        let e = export_material_textures(vec![make_slot("diffuse", "t.png", 0)]);
        assert_eq!(slot_name(&e, 0), "diffuse");
        assert_eq!(slot_name(&e, 5), "");
    }

    #[test]
    fn test_slot_texture_path() {
        let e = export_material_textures(vec![make_slot("d", "path.png", 0)]);
        assert_eq!(slot_texture_path(&e, 0), "path.png");
    }

    #[test]
    fn test_slot_to_json() {
        let e = export_material_textures(vec![make_slot("d", "t.png", 0)]);
        let j = slot_to_json(&e, 0);
        assert!(j.contains("name"));
    }

    #[test]
    fn test_slot_to_json_oob() {
        let e = export_material_textures(vec![]);
        assert_eq!(slot_to_json(&e, 0), "{}");
    }

    #[test]
    fn test_slot_uv_channel() {
        let e = export_material_textures(vec![make_slot("d", "t.png", 2)]);
        assert_eq!(slot_uv_channel(&e, 0), 2);
        assert_eq!(slot_uv_channel(&e, 5), 0);
    }

    #[test]
    fn test_texture_export_size() {
        let e = export_material_textures(vec![make_slot("ab", "cd", 0)]);
        assert!(texture_export_size_mt(&e) > 0);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_material_textures(vec![make_slot("a", "b", 0)]);
        assert!(validate_material_textures(&e));
    }

    #[test]
    fn test_validate_empty_name() {
        let e = export_material_textures(vec![make_slot("", "b", 0)]);
        assert!(!validate_material_textures(&e));
    }
}
