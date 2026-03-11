#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphExportConfig {
    format: String,
    precision: u32,
}

#[allow(dead_code)]
pub fn new_morph_export_config() -> MorphExportConfig {
    MorphExportConfig { format: "json".to_string(), precision: 6 }
}

#[allow(dead_code)]
pub fn set_export_format(cfg: &mut MorphExportConfig, fmt: &str) {
    cfg.format = fmt.to_string();
}

#[allow(dead_code)]
pub fn export_format(cfg: &MorphExportConfig) -> &str { &cfg.format }

#[allow(dead_code)]
pub fn set_export_precision(cfg: &mut MorphExportConfig, p: u32) {
    cfg.precision = p.clamp(1, 15);
}

#[allow(dead_code)]
pub fn export_precision(cfg: &MorphExportConfig) -> u32 { cfg.precision }

#[allow(dead_code)]
pub fn config_to_json_mec(cfg: &MorphExportConfig) -> String {
    format!("{{\"format\":\"{}\",\"precision\":{}}}", cfg.format, cfg.precision)
}

#[allow(dead_code)]
pub fn validate_export_config(cfg: &MorphExportConfig) -> bool {
    let valid_formats = ["json", "csv", "bin", "obj"];
    valid_formats.contains(&cfg.format.as_str()) && (1..=15).contains(&cfg.precision)
}

#[allow(dead_code)]
pub fn default_export_config() -> MorphExportConfig {
    new_morph_export_config()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let c = new_morph_export_config(); assert_eq!(export_format(&c), "json"); }
    #[test] fn test_set_format() { let mut c = new_morph_export_config(); set_export_format(&mut c, "csv"); assert_eq!(export_format(&c), "csv"); }
    #[test] fn test_precision() { let c = new_morph_export_config(); assert_eq!(export_precision(&c), 6); }
    #[test] fn test_set_precision() { let mut c = new_morph_export_config(); set_export_precision(&mut c, 3); assert_eq!(export_precision(&c), 3); }
    #[test] fn test_clamp_precision() { let mut c = new_morph_export_config(); set_export_precision(&mut c, 100); assert_eq!(export_precision(&c), 15); }
    #[test] fn test_json() { let c = new_morph_export_config(); assert!(config_to_json_mec(&c).contains("format")); }
    #[test] fn test_validate_ok() { let c = new_morph_export_config(); assert!(validate_export_config(&c)); }
    #[test] fn test_validate_bad_fmt() { let mut c = new_morph_export_config(); set_export_format(&mut c, "xyz"); assert!(!validate_export_config(&c)); }
    #[test] fn test_default() { let c = default_export_config(); assert_eq!(export_format(&c), "json"); }
    #[test] fn test_precision_min() { let mut c = new_morph_export_config(); set_export_precision(&mut c, 0); assert_eq!(export_precision(&c), 1); }
}
