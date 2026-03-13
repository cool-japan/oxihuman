// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export lightmap UV channel and baked irradiance samples as JSON or compact binary.

#![allow(dead_code)]

/// Configuration for lightmap-UV export.
#[derive(Debug, Clone)]
pub struct LightmapUvExportConfig {
    /// Pretty-print JSON output.
    pub pretty: bool,
    /// Scale factor applied to UV coordinates before export.
    pub uv_scale: f32,
    /// Include irradiance samples in the export.
    pub include_irradiance: bool,
}

/// A single vertex in the lightmap UV channel.
#[derive(Debug, Clone)]
pub struct LightmapUvVertex {
    /// Lightmap U coordinate (0.0–1.0).
    pub u: f32,
    /// Lightmap V coordinate (0.0–1.0).
    pub v: f32,
    /// Baked irradiance value (R, G, B) at this texel.
    pub irradiance: [f32; 3],
}

/// Container holding all lightmap UV data for export.
#[derive(Debug, Clone)]
pub struct LightmapUvExportResult {
    /// All lightmap UV vertices.
    pub vertices: Vec<LightmapUvVertex>,
    /// Byte count of the last serialised output.
    pub total_bytes: usize,
}

/// Returns the default [`LightmapUvExportConfig`].
pub fn default_lightmap_uv_config() -> LightmapUvExportConfig {
    LightmapUvExportConfig {
        pretty: true,
        uv_scale: 1.0,
        include_irradiance: true,
    }
}

/// Creates a new, empty [`LightmapUvExportResult`].
pub fn new_lightmap_uv_export() -> LightmapUvExportResult {
    LightmapUvExportResult {
        vertices: Vec::new(),
        total_bytes: 0,
    }
}

/// Appends a vertex to the export container.
pub fn lmuv_add_vertex(result: &mut LightmapUvExportResult, vertex: LightmapUvVertex) {
    result.vertices.push(vertex);
}

/// Serialises the lightmap UV data as JSON.
pub fn lmuv_export_json(
    result: &mut LightmapUvExportResult,
    cfg: &LightmapUvExportConfig,
) -> String {
    let indent = if cfg.pretty { "  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };
    let scale = cfg.uv_scale;

    let mut out = format!("{{{nl}{indent}\"vertices\":[{nl}");
    let len = result.vertices.len();
    for (i, v) in result.vertices.iter().enumerate() {
        let comma = if i + 1 < len { "," } else { "" };
        let u = v.u * scale;
        let vu = v.v * scale;
        if cfg.include_irradiance {
            out.push_str(&format!(
                "{indent}{indent}{{\"u\":{u:.6},\"v\":{vu:.6},\
                 \"irradiance\":[{:.6},{:.6},{:.6}]}}{comma}{nl}",
                v.irradiance[0], v.irradiance[1], v.irradiance[2]
            ));
        } else {
            out.push_str(&format!(
                "{indent}{indent}{{\"u\":{u:.6},\"v\":{vu:.6}}}{comma}{nl}"
            ));
        }
    }
    out.push_str(&format!("{indent}]{nl}}}"));
    result.total_bytes = out.len();
    out
}

/// Serialises the lightmap UV data as a compact binary blob.
/// Format: `[u32 count][f32 u][f32 v][f32 r][f32 g][f32 b]` per vertex.
pub fn lmuv_export_binary(
    result: &mut LightmapUvExportResult,
    cfg: &LightmapUvExportConfig,
) -> Vec<u8> {
    let scale = cfg.uv_scale;
    let count = result.vertices.len() as u32;
    let mut buf: Vec<u8> = Vec::with_capacity(4 + result.vertices.len() * 20);
    buf.extend_from_slice(&count.to_le_bytes());
    for v in &result.vertices {
        buf.extend_from_slice(&(v.u * scale).to_le_bytes());
        buf.extend_from_slice(&(v.v * scale).to_le_bytes());
        buf.extend_from_slice(&v.irradiance[0].to_le_bytes());
        buf.extend_from_slice(&v.irradiance[1].to_le_bytes());
        buf.extend_from_slice(&v.irradiance[2].to_le_bytes());
    }
    result.total_bytes = buf.len();
    buf
}

/// Returns the number of vertices stored.
pub fn lmuv_vertex_count(result: &LightmapUvExportResult) -> usize {
    result.vertices.len()
}

/// Writes JSON to a file path (stub — returns byte count).
pub fn lmuv_write_to_file(
    result: &mut LightmapUvExportResult,
    cfg: &LightmapUvExportConfig,
    _path: &str,
) -> usize {
    let json = lmuv_export_json(result, cfg);
    result.total_bytes = json.len();
    result.total_bytes
}

/// Returns the byte count of the last serialised output.
pub fn lmuv_total_bytes(result: &LightmapUvExportResult) -> usize {
    result.total_bytes
}

/// Clears all vertices and resets state.
pub fn lmuv_clear(result: &mut LightmapUvExportResult) {
    result.vertices.clear();
    result.total_bytes = 0;
}

/// Returns the fraction of lightmap area covered by UV islands (naive: max extent product).
/// Returns 0.0 for empty exports.
pub fn lmuv_coverage(result: &LightmapUvExportResult, cfg: &LightmapUvExportConfig) -> f32 {
    if result.vertices.is_empty() {
        return 0.0;
    }
    let scale = cfg.uv_scale;
    let mut min_u = f32::MAX;
    let mut max_u = f32::MIN;
    let mut min_v = f32::MAX;
    let mut max_v = f32::MIN;
    for v in &result.vertices {
        let u = v.u * scale;
        let vv = v.v * scale;
        if u < min_u {
            min_u = u;
        }
        if u > max_u {
            max_u = u;
        }
        if vv < min_v {
            min_v = vv;
        }
        if vv > max_v {
            max_v = vv;
        }
    }
    let w = (max_u - min_u).clamp(0.0, 1.0);
    let h = (max_v - min_v).clamp(0.0, 1.0);
    w * h
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_vertex(u: f32, v: f32, irr: [f32; 3]) -> LightmapUvVertex {
    LightmapUvVertex { u, v, irradiance: irr }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_lightmap_uv_config();
        assert!(cfg.pretty);
        assert!((cfg.uv_scale - 1.0).abs() < 1e-6);
        assert!(cfg.include_irradiance);
    }

    #[test]
    fn new_export_is_empty() {
        let r = new_lightmap_uv_export();
        assert_eq!(lmuv_vertex_count(&r), 0);
        assert_eq!(lmuv_total_bytes(&r), 0);
    }

    #[test]
    fn add_vertex_increments_count() {
        let mut r = new_lightmap_uv_export();
        lmuv_add_vertex(&mut r, make_vertex(0.1, 0.2, [1.0, 0.8, 0.6]));
        assert_eq!(lmuv_vertex_count(&r), 1);
    }

    #[test]
    fn json_contains_vertices_key() {
        let mut r = new_lightmap_uv_export();
        lmuv_add_vertex(&mut r, make_vertex(0.25, 0.75, [0.5, 0.5, 0.5]));
        let cfg = default_lightmap_uv_config();
        let json = lmuv_export_json(&mut r, &cfg);
        assert!(json.contains("\"vertices\""));
        assert!(json.contains("\"u\""));
        assert!(json.contains("irradiance"));
    }

    #[test]
    fn json_without_irradiance() {
        let mut r = new_lightmap_uv_export();
        lmuv_add_vertex(&mut r, make_vertex(0.1, 0.1, [0.0, 0.0, 0.0]));
        let mut cfg = default_lightmap_uv_config();
        cfg.include_irradiance = false;
        let json = lmuv_export_json(&mut r, &cfg);
        assert!(!json.contains("irradiance"));
        assert!(json.contains("\"u\""));
    }

    #[test]
    fn binary_export_has_correct_length() {
        let mut r = new_lightmap_uv_export();
        lmuv_add_vertex(&mut r, make_vertex(0.0, 0.0, [0.0, 0.0, 0.0]));
        lmuv_add_vertex(&mut r, make_vertex(1.0, 1.0, [1.0, 1.0, 1.0]));
        let cfg = default_lightmap_uv_config();
        let bin = lmuv_export_binary(&mut r, &cfg);
        // 4 bytes header + 2 * 20 bytes per vertex
        assert_eq!(bin.len(), 4 + 2 * 20);
    }

    #[test]
    fn binary_export_count_field() {
        let mut r = new_lightmap_uv_export();
        for _ in 0..3 {
            lmuv_add_vertex(&mut r, make_vertex(0.5, 0.5, [0.5, 0.5, 0.5]));
        }
        let cfg = default_lightmap_uv_config();
        let bin = lmuv_export_binary(&mut r, &cfg);
        let count = u32::from_le_bytes(bin[0..4].try_into().expect("should succeed"));
        assert_eq!(count, 3);
    }

    #[test]
    fn coverage_empty_is_zero() {
        let r = new_lightmap_uv_export();
        let cfg = default_lightmap_uv_config();
        assert!((lmuv_coverage(&r, &cfg) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn coverage_nonzero_for_spread_vertices() {
        let mut r = new_lightmap_uv_export();
        lmuv_add_vertex(&mut r, make_vertex(0.0, 0.0, [0.0, 0.0, 0.0]));
        lmuv_add_vertex(&mut r, make_vertex(0.5, 0.5, [0.0, 0.0, 0.0]));
        let cfg = default_lightmap_uv_config();
        let cov = lmuv_coverage(&r, &cfg);
        assert!(cov > 0.0);
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut r = new_lightmap_uv_export();
        lmuv_add_vertex(&mut r, make_vertex(0.3, 0.7, [0.9, 0.8, 0.7]));
        let cfg = default_lightmap_uv_config();
        let n = lmuv_write_to_file(&mut r, &cfg, "/tmp/lmuv.json");
        assert!(n > 0);
        assert_eq!(lmuv_total_bytes(&r), n);
    }

    #[test]
    fn clear_resets_state() {
        let mut r = new_lightmap_uv_export();
        lmuv_add_vertex(&mut r, make_vertex(0.1, 0.2, [0.3, 0.4, 0.5]));
        let cfg = default_lightmap_uv_config();
        lmuv_write_to_file(&mut r, &cfg, "/tmp/lmuv.json");
        lmuv_clear(&mut r);
        assert_eq!(lmuv_vertex_count(&r), 0);
        assert_eq!(lmuv_total_bytes(&r), 0);
    }

    #[test]
    fn uv_scale_applied_in_json() {
        let mut r = new_lightmap_uv_export();
        lmuv_add_vertex(&mut r, make_vertex(0.5, 0.5, [0.0, 0.0, 0.0]));
        let mut cfg = default_lightmap_uv_config();
        cfg.uv_scale = 2.0;
        let json = lmuv_export_json(&mut r, &cfg);
        // scaled u = 1.0
        assert!(json.contains("1.000000"));
    }
}
