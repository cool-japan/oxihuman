#![allow(dead_code)]
//! Export scene configuration.

/// Scene config export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneConfigExport {
    pub unit: String,
    pub up_axis: String,
    pub author: String,
    pub created_at: String,
}

/// Export scene config.
#[allow(dead_code)]
pub fn export_scene_config(
    unit: &str,
    up_axis: &str,
    author: &str,
    created_at: &str,
) -> SceneConfigExport {
    SceneConfigExport {
        unit: unit.to_string(),
        up_axis: up_axis.to_string(),
        author: author.to_string(),
        created_at: created_at.to_string(),
    }
}

/// Return unit string.
#[allow(dead_code)]
pub fn scene_unit(exp: &SceneConfigExport) -> &str {
    &exp.unit
}

/// Return up axis.
#[allow(dead_code)]
pub fn scene_up_axis(exp: &SceneConfigExport) -> &str {
    &exp.up_axis
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn scene_to_json_config(exp: &SceneConfigExport) -> String {
    format!(
        "{{\"unit\":\"{}\",\"up_axis\":\"{}\",\"author\":\"{}\",\"created_at\":\"{}\"}}",
        exp.unit, exp.up_axis, exp.author, exp.created_at
    )
}

/// Return author.
#[allow(dead_code)]
pub fn scene_author(exp: &SceneConfigExport) -> &str {
    &exp.author
}

/// Return creation timestamp.
#[allow(dead_code)]
pub fn scene_created_at(exp: &SceneConfigExport) -> &str {
    &exp.created_at
}

/// Compute export size.
#[allow(dead_code)]
pub fn scene_export_size_config(exp: &SceneConfigExport) -> usize {
    scene_to_json_config(exp).len()
}

/// Validate scene config.
#[allow(dead_code)]
pub fn validate_scene_config(exp: &SceneConfigExport) -> bool {
    !exp.unit.is_empty() && !exp.up_axis.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_scene_config() {
        let e = export_scene_config("meters", "Y", "test", "2025-01-01");
        assert_eq!(scene_unit(&e), "meters");
    }

    #[test]
    fn test_scene_up_axis() {
        let e = export_scene_config("cm", "Z", "", "");
        assert_eq!(scene_up_axis(&e), "Z");
    }

    #[test]
    fn test_scene_to_json() {
        let e = export_scene_config("m", "Y", "a", "t");
        let j = scene_to_json_config(&e);
        assert!(j.contains("\"unit\":\"m\""));
    }

    #[test]
    fn test_scene_author() {
        let e = export_scene_config("m", "Y", "alice", "");
        assert_eq!(scene_author(&e), "alice");
    }

    #[test]
    fn test_scene_created_at() {
        let e = export_scene_config("m", "Y", "", "2025-06-01");
        assert_eq!(scene_created_at(&e), "2025-06-01");
    }

    #[test]
    fn test_scene_export_size() {
        let e = export_scene_config("m", "Y", "a", "t");
        assert!(scene_export_size_config(&e) > 0);
    }

    #[test]
    fn test_validate_scene_config() {
        let e = export_scene_config("m", "Y", "", "");
        assert!(validate_scene_config(&e));
    }

    #[test]
    fn test_validate_empty_unit() {
        let e = export_scene_config("", "Y", "", "");
        assert!(!validate_scene_config(&e));
    }

    #[test]
    fn test_validate_empty_axis() {
        let e = export_scene_config("m", "", "", "");
        assert!(!validate_scene_config(&e));
    }

    #[test]
    fn test_default_scene() {
        let e = export_scene_config("meters", "Y", "author", "now");
        assert_eq!(scene_unit(&e), "meters");
        assert_eq!(scene_up_axis(&e), "Y");
    }
}
