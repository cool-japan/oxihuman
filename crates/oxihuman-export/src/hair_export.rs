// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Hair strand data export to various formats (CSV, JSON, custom binary).

/// Supported output formats for hair export.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum HairExportFormat {
    Csv,
    Json,
    BinaryCustom,
}

/// Configuration controlling how hair strands are exported.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairExportConfig {
    pub format: HairExportFormat,
    pub strand_count_limit: usize,
    pub include_width: bool,
    pub include_color: bool,
}

/// Per-strand data: control points, per-point widths, and a root color.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairStrandData {
    pub control_points: Vec<[f32; 3]>,
    pub width: Vec<f32>,
    pub color: [f32; 4],
}

/// Result of a hair export operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairExportResult {
    pub data: String,
    pub strand_count: usize,
    pub total_points: usize,
}

/// Returns a default hair export configuration (JSON, no limit).
#[allow(dead_code)]
pub fn default_hair_export_config() -> HairExportConfig {
    HairExportConfig {
        format: HairExportFormat::Json,
        strand_count_limit: 0,
        include_width: true,
        include_color: true,
    }
}

/// Creates a new `HairStrandData` from control points and a root color.
/// Width is initialised to 0.05 per point.
#[allow(dead_code)]
pub fn new_hair_strand_data(cps: Vec<[f32; 3]>, color: [f32; 4]) -> HairStrandData {
    let width = vec![0.05; cps.len()];
    HairStrandData { control_points: cps, width, color }
}

/// Exports hair strands according to the provided configuration.
#[allow(dead_code)]
pub fn export_hair_strands(strands: &[HairStrandData], cfg: &HairExportConfig) -> HairExportResult {
    let limited: &[HairStrandData] = if cfg.strand_count_limit > 0 {
        &strands[..strands.len().min(cfg.strand_count_limit)]
    } else {
        strands
    };

    let data = match cfg.format {
        HairExportFormat::Csv => export_to_csv(limited),
        HairExportFormat::Json => export_to_json_hair(limited),
        HairExportFormat::BinaryCustom => {
            format!("OXHAIR_BIN strands={}", limited.len())
        }
    };

    HairExportResult {
        data,
        strand_count: limited.len(),
        total_points: total_hair_points(limited),
    }
}

/// Exports hair strands as CSV (one row per control point).
#[allow(dead_code)]
pub fn export_to_csv(strands: &[HairStrandData]) -> String {
    let mut out = String::from("strand_id,point_id,x,y,z,width,r,g,b,a\n");
    for (si, s) in strands.iter().enumerate() {
        for (pi, cp) in s.control_points.iter().enumerate() {
            let w = s.width.get(pi).copied().unwrap_or(0.05);
            let [r, g, b, a] = s.color;
            out.push_str(&format!(
                "{si},{pi},{x:.4},{y:.4},{z:.4},{w:.4},{r:.4},{g:.4},{b:.4},{a:.4}\n",
                x = cp[0], y = cp[1], z = cp[2],
            ));
        }
    }
    out
}

/// Exports hair strands as a JSON string.
#[allow(dead_code)]
pub fn export_to_json_hair(strands: &[HairStrandData]) -> String {
    let mut items = Vec::new();
    for s in strands {
        let pts: Vec<String> = s.control_points
            .iter()
            .map(|p| format!("[{:.4},{:.4},{:.4}]", p[0], p[1], p[2]))
            .collect();
        let [r, g, b, a] = s.color;
        items.push(format!(
            r#"{{"points":[{}],"color":[{r:.4},{g:.4},{b:.4},{a:.4}]}}"#,
            pts.join(","),
        ));
    }
    format!("[{}]", items.join(","))
}

/// Returns the number of control points in a single strand.
#[allow(dead_code)]
pub fn strand_point_count(strand: &HairStrandData) -> usize {
    strand.control_points.len()
}

/// Returns the total number of control points across all strands.
#[allow(dead_code)]
pub fn total_hair_points(strands: &[HairStrandData]) -> usize {
    strands.iter().map(strand_point_count).sum()
}

/// Returns a human-readable format name for the export config.
#[allow(dead_code)]
pub fn hair_format_name(cfg: &HairExportConfig) -> &'static str {
    match cfg.format {
        HairExportFormat::Csv => "csv",
        HairExportFormat::Json => "json",
        HairExportFormat::BinaryCustom => "binary_custom",
    }
}

/// Serialises an `HairExportResult` to a JSON string.
#[allow(dead_code)]
pub fn hair_export_result_to_json(r: &HairExportResult) -> String {
    format!(
        r#"{{"strand_count":{s},"total_points":{p},"data_len":{d}}}"#,
        s = r.strand_count,
        p = r.total_points,
        d = r.data.len(),
    )
}

/// Returns `true` when all strands have at least one control point.
#[allow(dead_code)]
pub fn validate_hair_strands(strands: &[HairStrandData]) -> bool {
    !strands.is_empty() && strands.iter().all(|s| !s.control_points.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_strand(n: usize) -> HairStrandData {
        let cps: Vec<[f32; 3]> = (0..n).map(|i| [i as f32, 0.0, 0.0]).collect();
        new_hair_strand_data(cps, [1.0, 1.0, 1.0, 1.0])
    }

    #[test]
    fn default_config_is_json() {
        let cfg = default_hair_export_config();
        assert_eq!(cfg.format, HairExportFormat::Json);
        assert!(cfg.include_width);
        assert!(cfg.include_color);
    }

    #[test]
    fn strand_point_count_correct() {
        let s = make_strand(5);
        assert_eq!(strand_point_count(&s), 5);
    }

    #[test]
    fn total_hair_points_sums_all() {
        let strands = vec![make_strand(3), make_strand(4)];
        assert_eq!(total_hair_points(&strands), 7);
    }

    #[test]
    fn export_csv_has_header() {
        let strands = vec![make_strand(2)];
        let csv = export_to_csv(&strands);
        assert!(csv.starts_with("strand_id,"));
        assert!(csv.contains("0,0,"));
    }

    #[test]
    fn export_json_hair_is_array() {
        let strands = vec![make_strand(2)];
        let json = export_to_json_hair(&strands);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }

    #[test]
    fn validate_hair_strands_ok() {
        let strands = vec![make_strand(2)];
        assert!(validate_hair_strands(&strands));
    }

    #[test]
    fn validate_hair_strands_empty_fails() {
        assert!(!validate_hair_strands(&[]));
    }

    #[test]
    fn export_hair_strands_respects_limit() {
        let strands = vec![make_strand(3), make_strand(3), make_strand(3)];
        let mut cfg = default_hair_export_config();
        cfg.strand_count_limit = 2;
        let result = export_hair_strands(&strands, &cfg);
        assert_eq!(result.strand_count, 2);
    }

    #[test]
    fn hair_format_name_values() {
        let mut cfg = default_hair_export_config();
        cfg.format = HairExportFormat::Csv;
        assert_eq!(hair_format_name(&cfg), "csv");
        cfg.format = HairExportFormat::BinaryCustom;
        assert_eq!(hair_format_name(&cfg), "binary_custom");
    }

    #[test]
    fn result_to_json_contains_fields() {
        let r = HairExportResult { data: "x".to_string(), strand_count: 3, total_points: 9 };
        let json = hair_export_result_to_json(&r);
        assert!(json.contains("\"strand_count\":3"));
        assert!(json.contains("\"total_points\":9"));
    }
}
