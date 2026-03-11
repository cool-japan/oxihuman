#![allow(dead_code)]
//! Export scene assets.

/// Scene asset export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneAssetExport {
    pub asset_type: String,
    pub name: String,
    pub dependencies: Vec<String>,
    pub data_size: usize,
}

/// Export a scene asset.
#[allow(dead_code)]
pub fn export_scene_asset(
    asset_type: &str,
    name: &str,
    dependencies: &[&str],
    data_size: usize,
) -> SceneAssetExport {
    SceneAssetExport {
        asset_type: asset_type.to_string(),
        name: name.to_string(),
        dependencies: dependencies.iter().map(|s| s.to_string()).collect(),
        data_size,
    }
}

/// Return asset type.
#[allow(dead_code)]
pub fn asset_type(exp: &SceneAssetExport) -> &str {
    &exp.asset_type
}

/// Return asset name.
#[allow(dead_code)]
pub fn asset_name(exp: &SceneAssetExport) -> &str {
    &exp.name
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn asset_to_json(exp: &SceneAssetExport) -> String {
    let deps: Vec<String> = exp.dependencies.iter().map(|d| format!("\"{}\"", d)).collect();
    format!(
        "{{\"type\":\"{}\",\"name\":\"{}\",\"dependencies\":[{}],\"size\":{}}}",
        exp.asset_type,
        exp.name,
        deps.join(","),
        exp.data_size
    )
}

/// Return dependencies.
#[allow(dead_code)]
pub fn asset_dependencies(exp: &SceneAssetExport) -> &[String] {
    &exp.dependencies
}

/// Return data size.
#[allow(dead_code)]
pub fn asset_size(exp: &SceneAssetExport) -> usize {
    exp.data_size
}

/// Compute export size.
#[allow(dead_code)]
pub fn asset_export_size(exp: &SceneAssetExport) -> usize {
    asset_to_json(exp).len()
}

/// Validate scene asset.
#[allow(dead_code)]
pub fn validate_scene_asset(exp: &SceneAssetExport) -> bool {
    !exp.asset_type.is_empty() && !exp.name.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_scene_asset() {
        let e = export_scene_asset("mesh", "body", &["skeleton"], 1024);
        assert_eq!(asset_name(&e), "body");
    }

    #[test]
    fn test_asset_type() {
        let e = export_scene_asset("texture", "diffuse", &[], 512);
        assert_eq!(asset_type(&e), "texture");
    }

    #[test]
    fn test_asset_to_json() {
        let e = export_scene_asset("mesh", "m", &[], 0);
        let j = asset_to_json(&e);
        assert!(j.contains("\"type\":\"mesh\""));
    }

    #[test]
    fn test_asset_dependencies() {
        let e = export_scene_asset("mesh", "m", &["tex", "skel"], 0);
        assert_eq!(asset_dependencies(&e).len(), 2);
    }

    #[test]
    fn test_asset_size() {
        let e = export_scene_asset("mesh", "m", &[], 2048);
        assert_eq!(asset_size(&e), 2048);
    }

    #[test]
    fn test_asset_export_size() {
        let e = export_scene_asset("mesh", "m", &[], 0);
        assert!(asset_export_size(&e) > 0);
    }

    #[test]
    fn test_validate_scene_asset() {
        let e = export_scene_asset("mesh", "m", &[], 0);
        assert!(validate_scene_asset(&e));
    }

    #[test]
    fn test_validate_empty_type() {
        let e = export_scene_asset("", "m", &[], 0);
        assert!(!validate_scene_asset(&e));
    }

    #[test]
    fn test_validate_empty_name() {
        let e = export_scene_asset("mesh", "", &[], 0);
        assert!(!validate_scene_asset(&e));
    }

    #[test]
    fn test_no_dependencies() {
        let e = export_scene_asset("mesh", "m", &[], 0);
        assert!(asset_dependencies(&e).is_empty());
    }
}
