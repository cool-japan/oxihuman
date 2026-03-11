#![allow(dead_code)]
//! Asset info export.

/// Asset info export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AssetInfoExport {
    pub generator: String,
    pub version: String,
    pub copyright: String,
    pub min_version: String,
}

/// Export asset info.
#[allow(dead_code)]
pub fn export_asset_info(generator: &str, version: &str, copyright: &str, min_version: &str) -> AssetInfoExport {
    AssetInfoExport {
        generator: generator.to_string(),
        version: version.to_string(),
        copyright: copyright.to_string(),
        min_version: min_version.to_string(),
    }
}

/// Get generator name.
#[allow(dead_code)]
pub fn asset_generator(e: &AssetInfoExport) -> &str {
    &e.generator
}

/// Get version.
#[allow(dead_code)]
pub fn asset_version_aie(e: &AssetInfoExport) -> &str {
    &e.version
}

/// Get copyright.
#[allow(dead_code)]
pub fn asset_copyright(e: &AssetInfoExport) -> &str {
    &e.copyright
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn asset_to_json(e: &AssetInfoExport) -> String {
    format!(
        "{{\"generator\":\"{}\",\"version\":\"{}\",\"copyright\":\"{}\"}}",
        e.generator, e.version, e.copyright
    )
}

/// Get minimum version.
#[allow(dead_code)]
pub fn asset_min_version(e: &AssetInfoExport) -> &str {
    &e.min_version
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn asset_export_size(e: &AssetInfoExport) -> usize {
    e.generator.len() + e.version.len() + e.copyright.len() + e.min_version.len()
}

/// Validate asset info.
#[allow(dead_code)]
pub fn validate_asset_info(e: &AssetInfoExport) -> bool {
    !e.version.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_asset_info() {
        let e = export_asset_info("oxihuman", "2.0", "2024", "2.0");
        assert_eq!(e.generator, "oxihuman");
    }

    #[test]
    fn test_asset_generator() {
        let e = export_asset_info("gen", "2.0", "", "");
        assert_eq!(asset_generator(&e), "gen");
    }

    #[test]
    fn test_asset_version() {
        let e = export_asset_info("", "2.0", "", "");
        assert_eq!(asset_version_aie(&e), "2.0");
    }

    #[test]
    fn test_asset_copyright() {
        let e = export_asset_info("", "2.0", "MIT", "");
        assert_eq!(asset_copyright(&e), "MIT");
    }

    #[test]
    fn test_asset_to_json() {
        let e = export_asset_info("gen", "2.0", "c", "");
        let j = asset_to_json(&e);
        assert!(j.contains("generator"));
    }

    #[test]
    fn test_asset_min_version() {
        let e = export_asset_info("", "2.0", "", "1.0");
        assert_eq!(asset_min_version(&e), "1.0");
    }

    #[test]
    fn test_asset_export_size() {
        let e = export_asset_info("ab", "cd", "ef", "gh");
        assert_eq!(asset_export_size(&e), 8);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_asset_info("", "2.0", "", "");
        assert!(validate_asset_info(&e));
    }

    #[test]
    fn test_validate_empty_version() {
        let e = export_asset_info("gen", "", "", "");
        assert!(!validate_asset_info(&e));
    }

    #[test]
    fn test_asset_info_full() {
        let e = export_asset_info("oxihuman", "2.0", "2024 OxiHuman", "2.0");
        assert!(validate_asset_info(&e));
        assert!(!asset_to_json(&e).is_empty());
    }
}
