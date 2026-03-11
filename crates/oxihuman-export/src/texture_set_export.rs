#![allow(dead_code)]
//! Export texture sets.

/// Texture set export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TextureSetExport {
    pub textures: Vec<TextureEntry>,
}

/// A texture entry.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TextureEntry {
    pub name: String,
    pub format: String,
    pub size: usize,
}

/// Export a texture set.
#[allow(dead_code)]
pub fn export_texture_set(textures: Vec<TextureEntry>) -> TextureSetExport {
    TextureSetExport { textures }
}

/// Return texture count.
#[allow(dead_code)]
pub fn texture_set_count(exp: &TextureSetExport) -> usize {
    exp.textures.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn texture_set_to_json(exp: &TextureSetExport) -> String {
    let items: Vec<String> = exp
        .textures
        .iter()
        .map(|t| format!("{{\"name\":\"{}\",\"format\":\"{}\",\"size\":{}}}", t.name, t.format, t.size))
        .collect();
    format!("{{\"textures\":[{}]}}", items.join(","))
}

/// Return texture names.
#[allow(dead_code)]
pub fn texture_set_names(exp: &TextureSetExport) -> Vec<&str> {
    exp.textures.iter().map(|t| t.name.as_str()).collect()
}

/// Return texture formats.
#[allow(dead_code)]
pub fn texture_set_formats(exp: &TextureSetExport) -> Vec<&str> {
    exp.textures.iter().map(|t| t.format.as_str()).collect()
}

/// Return total size of all textures.
#[allow(dead_code)]
pub fn texture_set_total_size(exp: &TextureSetExport) -> usize {
    exp.textures.iter().map(|t| t.size).sum()
}

/// Compute export size.
#[allow(dead_code)]
pub fn texture_set_export_size(exp: &TextureSetExport) -> usize {
    texture_set_to_json(exp).len()
}

/// Validate texture set.
#[allow(dead_code)]
pub fn validate_texture_set(exp: &TextureSetExport) -> bool {
    !exp.textures.is_empty()
        && exp.textures.iter().all(|t| !t.name.is_empty() && !t.format.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tex() -> TextureEntry {
        TextureEntry {
            name: "diffuse".to_string(),
            format: "png".to_string(),
            size: 1024,
        }
    }

    #[test]
    fn test_export_texture_set() {
        let e = export_texture_set(vec![sample_tex()]);
        assert_eq!(texture_set_count(&e), 1);
    }

    #[test]
    fn test_texture_set_to_json() {
        let e = export_texture_set(vec![sample_tex()]);
        let j = texture_set_to_json(&e);
        assert!(j.contains("\"textures\""));
    }

    #[test]
    fn test_texture_set_names() {
        let e = export_texture_set(vec![sample_tex()]);
        assert_eq!(texture_set_names(&e), vec!["diffuse"]);
    }

    #[test]
    fn test_texture_set_formats() {
        let e = export_texture_set(vec![sample_tex()]);
        assert_eq!(texture_set_formats(&e), vec!["png"]);
    }

    #[test]
    fn test_texture_set_total_size() {
        let e = export_texture_set(vec![sample_tex(), sample_tex()]);
        assert_eq!(texture_set_total_size(&e), 2048);
    }

    #[test]
    fn test_texture_set_export_size() {
        let e = export_texture_set(vec![sample_tex()]);
        assert!(texture_set_export_size(&e) > 0);
    }

    #[test]
    fn test_validate_texture_set() {
        let e = export_texture_set(vec![sample_tex()]);
        assert!(validate_texture_set(&e));
    }

    #[test]
    fn test_validate_empty() {
        let e = export_texture_set(vec![]);
        assert!(!validate_texture_set(&e));
    }

    #[test]
    fn test_validate_empty_name() {
        let t = TextureEntry { name: "".into(), format: "png".into(), size: 0 };
        let e = export_texture_set(vec![t]);
        assert!(!validate_texture_set(&e));
    }

    #[test]
    fn test_empty_texture_set() {
        let e = export_texture_set(vec![]);
        assert_eq!(texture_set_count(&e), 0);
        assert_eq!(texture_set_total_size(&e), 0);
    }
}
