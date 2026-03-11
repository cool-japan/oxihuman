// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export renderer configuration (resolution, AA, shadow, AO settings) as JSON.

#![allow(dead_code)]

/// Configuration for render-settings export.
#[derive(Debug, Clone)]
pub struct RenderSettingsExportConfig {
    /// Pretty-print JSON output.
    pub pretty: bool,
    /// Whether to validate settings before export.
    pub validate: bool,
}

/// A single renderer setting key-value pair.
#[derive(Debug, Clone)]
pub struct RenderSetting {
    /// Setting key (e.g. "resolution_width", "aa_samples").
    pub key: String,
    /// Setting value serialised as a string.
    pub value: String,
    /// Optional category grouping (e.g. "anti-aliasing", "shadow").
    pub category: String,
}

/// Container holding all renderer settings for export.
#[derive(Debug, Clone)]
pub struct RenderSettingsExport {
    /// All render settings.
    pub settings: Vec<RenderSetting>,
    /// Byte count of the last serialised output.
    pub total_bytes: usize,
}

/// Returns the default [`RenderSettingsExportConfig`].
pub fn default_render_settings_export_config() -> RenderSettingsExportConfig {
    RenderSettingsExportConfig {
        pretty: true,
        validate: true,
    }
}

/// Creates a new, empty [`RenderSettingsExport`].
pub fn new_render_settings_export() -> RenderSettingsExport {
    RenderSettingsExport {
        settings: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds (or overwrites) a setting in the export container.
pub fn rse_add_setting(export: &mut RenderSettingsExport, setting: RenderSetting) {
    if let Some(existing) = export.settings.iter_mut().find(|s| s.key == setting.key) {
        *existing = setting;
    } else {
        export.settings.push(setting);
    }
}

/// Serialises all settings as JSON.
pub fn rse_to_json(export: &mut RenderSettingsExport, cfg: &RenderSettingsExportConfig) -> String {
    let indent = if cfg.pretty { "  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };
    let mut out = format!("{{{nl}{indent}\"settings\":[{nl}");
    for (i, s) in export.settings.iter().enumerate() {
        let comma = if i + 1 < export.settings.len() { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"key\":\"{}\",\"value\":\"{}\",\"category\":\"{}\"}}{}{nl}",
            s.key, s.value, s.category, comma
        ));
    }
    out.push_str(&format!("{indent}]{nl}}}"));
    export.total_bytes = out.len();
    out
}

/// Returns the number of settings currently stored.
pub fn rse_setting_count(export: &RenderSettingsExport) -> usize {
    export.settings.len()
}

/// Looks up a setting by key.  Returns `None` if not found.
pub fn rse_get_setting<'a>(export: &'a RenderSettingsExport, key: &str) -> Option<&'a RenderSetting> {
    export.settings.iter().find(|s| s.key == key)
}

/// Writes JSON to a file path (stub — returns byte count).
pub fn rse_write_to_file(
    export: &mut RenderSettingsExport,
    cfg: &RenderSettingsExportConfig,
    _path: &str,
) -> usize {
    let json = rse_to_json(export, cfg);
    export.total_bytes = json.len();
    export.total_bytes
}

/// Clears all settings and resets state.
pub fn rse_clear(export: &mut RenderSettingsExport) {
    export.settings.clear();
    export.total_bytes = 0;
}

/// Returns the byte count of the last serialised output.
pub fn rse_total_bytes(export: &RenderSettingsExport) -> usize {
    export.total_bytes
}

/// Validates settings: checks that no key or value is empty.
/// Returns a list of validation error strings (empty = valid).
pub fn rse_validate(export: &RenderSettingsExport, cfg: &RenderSettingsExportConfig) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    if !cfg.validate {
        return errors;
    }
    for s in &export.settings {
        if s.key.is_empty() {
            errors.push("setting with empty key".to_string());
        }
        if s.value.is_empty() {
            errors.push(format!("setting '{}' has empty value", s.key));
        }
    }
    errors
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_setting(key: &str, value: &str, category: &str) -> RenderSetting {
    RenderSetting {
        key: key.to_string(),
        value: value.to_string(),
        category: category.to_string(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_render_settings_export_config();
        assert!(cfg.pretty);
        assert!(cfg.validate);
    }

    #[test]
    fn new_export_is_empty() {
        let e = new_render_settings_export();
        assert_eq!(rse_setting_count(&e), 0);
        assert_eq!(rse_total_bytes(&e), 0);
    }

    #[test]
    fn add_setting_increments_count() {
        let mut e = new_render_settings_export();
        rse_add_setting(&mut e, make_setting("resolution_width", "1920", "output"));
        assert_eq!(rse_setting_count(&e), 1);
    }

    #[test]
    fn add_duplicate_key_overwrites() {
        let mut e = new_render_settings_export();
        rse_add_setting(&mut e, make_setting("aa_samples", "4", "anti-aliasing"));
        rse_add_setting(&mut e, make_setting("aa_samples", "8", "anti-aliasing"));
        assert_eq!(rse_setting_count(&e), 1);
        assert_eq!(rse_get_setting(&e, "aa_samples").unwrap().value, "8");
    }

    #[test]
    fn get_setting_returns_correct_value() {
        let mut e = new_render_settings_export();
        rse_add_setting(&mut e, make_setting("shadow_res", "2048", "shadow"));
        let s = rse_get_setting(&e, "shadow_res");
        assert!(s.is_some());
        assert_eq!(s.unwrap().value, "2048");
    }

    #[test]
    fn get_setting_missing_returns_none() {
        let e = new_render_settings_export();
        assert!(rse_get_setting(&e, "nonexistent").is_none());
    }

    #[test]
    fn json_contains_settings_key() {
        let mut e = new_render_settings_export();
        rse_add_setting(&mut e, make_setting("ao_radius", "0.5", "ambient-occlusion"));
        let cfg = default_render_settings_export_config();
        let json = rse_to_json(&mut e, &cfg);
        assert!(json.contains("\"settings\""));
        assert!(json.contains("ao_radius"));
        assert!(json.contains("ambient-occlusion"));
    }

    #[test]
    fn validate_catches_empty_value() {
        let mut e = new_render_settings_export();
        rse_add_setting(&mut e, make_setting("shadow_res", "", "shadow"));
        let cfg = default_render_settings_export_config();
        let errs = rse_validate(&e, &cfg);
        assert!(!errs.is_empty());
    }

    #[test]
    fn validate_ok_when_all_valid() {
        let mut e = new_render_settings_export();
        rse_add_setting(&mut e, make_setting("resolution_width", "1920", "output"));
        let cfg = default_render_settings_export_config();
        let errs = rse_validate(&e, &cfg);
        assert!(errs.is_empty());
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut e = new_render_settings_export();
        rse_add_setting(&mut e, make_setting("resolution_width", "1920", "output"));
        let cfg = default_render_settings_export_config();
        let n = rse_write_to_file(&mut e, &cfg, "/tmp/render_settings.json");
        assert!(n > 0);
        assert_eq!(rse_total_bytes(&e), n);
    }

    #[test]
    fn clear_resets_state() {
        let mut e = new_render_settings_export();
        rse_add_setting(&mut e, make_setting("resolution_width", "1920", "output"));
        let cfg = default_render_settings_export_config();
        rse_write_to_file(&mut e, &cfg, "/tmp/render_settings.json");
        rse_clear(&mut e);
        assert_eq!(rse_setting_count(&e), 0);
        assert_eq!(rse_total_bytes(&e), 0);
    }
}
