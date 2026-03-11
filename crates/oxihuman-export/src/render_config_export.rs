#![allow(dead_code)]
//! Export render configuration.

/// Render config export data.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct RenderConfigExport {
    pub shadow_enabled: bool,
    pub ao_enabled: bool,
    pub resolution: (u32, u32),
    pub quality: String,
}

/// Export render config.
#[allow(dead_code)]
pub fn export_render_config(
    shadow: bool,
    ao: bool,
    width: u32,
    height: u32,
    quality: &str,
) -> RenderConfigExport {
    RenderConfigExport {
        shadow_enabled: shadow,
        ao_enabled: ao,
        resolution: (width, height),
        quality: quality.to_string(),
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn config_to_json(exp: &RenderConfigExport) -> String {
    format!(
        "{{\"shadow\":{},\"ao\":{},\"width\":{},\"height\":{},\"quality\":\"{}\"}}",
        exp.shadow_enabled, exp.ao_enabled, exp.resolution.0, exp.resolution.1, exp.quality
    )
}

/// Check if shadows are enabled.
#[allow(dead_code)]
pub fn config_shadow_enabled(exp: &RenderConfigExport) -> bool {
    exp.shadow_enabled
}

/// Check if AO is enabled.
#[allow(dead_code)]
pub fn config_ao_enabled(exp: &RenderConfigExport) -> bool {
    exp.ao_enabled
}

/// Get resolution.
#[allow(dead_code)]
pub fn config_resolution(exp: &RenderConfigExport) -> (u32, u32) {
    exp.resolution
}

/// Get quality string.
#[allow(dead_code)]
pub fn config_quality(exp: &RenderConfigExport) -> &str {
    &exp.quality
}

/// Compute export size.
#[allow(dead_code)]
pub fn config_export_size(exp: &RenderConfigExport) -> usize {
    config_to_json(exp).len()
}

/// Validate config.
#[allow(dead_code)]
pub fn validate_render_config(exp: &RenderConfigExport) -> bool {
    exp.resolution.0 > 0 && exp.resolution.1 > 0 && !exp.quality.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_render_config() {
        let e = export_render_config(true, false, 1920, 1080, "high");
        assert!(config_shadow_enabled(&e));
    }

    #[test]
    fn test_config_to_json() {
        let e = export_render_config(true, true, 800, 600, "medium");
        let j = config_to_json(&e);
        assert!(j.contains("\"shadow\":true"));
    }

    #[test]
    fn test_config_ao() {
        let e = export_render_config(false, true, 100, 100, "low");
        assert!(config_ao_enabled(&e));
    }

    #[test]
    fn test_config_resolution() {
        let e = export_render_config(false, false, 1920, 1080, "high");
        assert_eq!(config_resolution(&e), (1920, 1080));
    }

    #[test]
    fn test_config_quality() {
        let e = export_render_config(false, false, 100, 100, "ultra_high");
        assert_eq!(config_quality(&e), "ultra_high");
    }

    #[test]
    fn test_config_export_size() {
        let e = export_render_config(true, true, 100, 100, "low");
        assert!(config_export_size(&e) > 0);
    }

    #[test]
    fn test_validate_render_config() {
        let e = export_render_config(true, true, 1920, 1080, "high");
        assert!(validate_render_config(&e));
    }

    #[test]
    fn test_validate_zero_resolution() {
        let e = export_render_config(true, true, 0, 1080, "high");
        assert!(!validate_render_config(&e));
    }

    #[test]
    fn test_validate_empty_quality() {
        let e = export_render_config(true, true, 100, 100, "");
        assert!(!validate_render_config(&e));
    }

    #[test]
    fn test_shadow_disabled() {
        let e = export_render_config(false, false, 100, 100, "low");
        assert!(!config_shadow_enabled(&e));
    }
}
