#![allow(dead_code)]
//! Extension export.

/// Extension export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionExport {
    pub name: String,
    pub data: String,
    pub required: bool,
    pub version: String,
}

/// Export an extension.
#[allow(dead_code)]
pub fn export_extension(name: &str, data: &str, required: bool, version: &str) -> ExtensionExport {
    ExtensionExport {
        name: name.to_string(),
        data: data.to_string(),
        required,
        version: version.to_string(),
    }
}

/// Get extension name.
#[allow(dead_code)]
pub fn extension_name(e: &ExtensionExport) -> &str {
    &e.name
}

/// Get extension data.
#[allow(dead_code)]
pub fn extension_data(e: &ExtensionExport) -> &str {
    &e.data
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn extension_to_json(e: &ExtensionExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"required\":{},\"version\":\"{}\"}}",
        e.name, e.required, e.version
    )
}

/// Check if required.
#[allow(dead_code)]
pub fn extension_is_required(e: &ExtensionExport) -> bool {
    e.required
}

/// Get version.
#[allow(dead_code)]
pub fn extension_version(e: &ExtensionExport) -> &str {
    &e.version
}

/// Get export size estimate.
#[allow(dead_code)]
pub fn extension_export_size(e: &ExtensionExport) -> usize {
    e.name.len() + e.data.len() + e.version.len()
}

/// Validate extension.
#[allow(dead_code)]
pub fn validate_extension(e: &ExtensionExport) -> bool {
    !e.name.is_empty() && !e.version.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_extension() {
        let e = export_extension("KHR_materials_unlit", "{}", true, "1.0");
        assert_eq!(e.name, "KHR_materials_unlit");
    }

    #[test]
    fn test_extension_name() {
        let e = export_extension("test", "{}", false, "1.0");
        assert_eq!(extension_name(&e), "test");
    }

    #[test]
    fn test_extension_data() {
        let e = export_extension("t", "some_data", false, "1.0");
        assert_eq!(extension_data(&e), "some_data");
    }

    #[test]
    fn test_extension_to_json() {
        let e = export_extension("t", "{}", true, "2.0");
        let j = extension_to_json(&e);
        assert!(j.contains("required"));
    }

    #[test]
    fn test_extension_is_required() {
        let e = export_extension("t", "{}", true, "1.0");
        assert!(extension_is_required(&e));
    }

    #[test]
    fn test_extension_not_required() {
        let e = export_extension("t", "{}", false, "1.0");
        assert!(!extension_is_required(&e));
    }

    #[test]
    fn test_extension_version() {
        let e = export_extension("t", "{}", false, "2.1");
        assert_eq!(extension_version(&e), "2.1");
    }

    #[test]
    fn test_extension_export_size() {
        let e = export_extension("ab", "cd", false, "ef");
        assert_eq!(extension_export_size(&e), 6);
    }

    #[test]
    fn test_validate_ok() {
        let e = export_extension("test", "{}", false, "1.0");
        assert!(validate_extension(&e));
    }

    #[test]
    fn test_validate_empty_name() {
        let e = export_extension("", "{}", false, "1.0");
        assert!(!validate_extension(&e));
    }
}
