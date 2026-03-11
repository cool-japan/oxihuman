// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export shape key (blend shape) definitions with basis mesh and delta arrays.

#![allow(dead_code)]

/// Configuration for shape-key export.
#[derive(Debug, Clone)]
pub struct ShapeKeyExportConfig {
    /// Pretty-print JSON output.
    pub pretty: bool,
    /// Include basis positions in export.
    pub include_basis: bool,
}

/// A single shape key entry.
#[derive(Debug, Clone)]
pub struct ShapeKeyEntry {
    /// Unique name for this shape key.
    pub name: String,
    /// Per-vertex position deltas (xyz) relative to the basis mesh.
    pub deltas: Vec<[f32; 3]>,
    /// Minimum blend weight (usually 0.0).
    pub weight_min: f32,
    /// Maximum blend weight (usually 1.0).
    pub weight_max: f32,
}

/// Container for shape key export data.
#[derive(Debug, Clone)]
pub struct ShapeKeyExport {
    /// Basis mesh vertex positions (xyz per vertex).
    pub basis: Vec<[f32; 3]>,
    /// All shape key entries.
    pub keys: Vec<ShapeKeyEntry>,
    /// Byte count of the last serialised output.
    pub total_bytes: usize,
}

/// Returns the default [`ShapeKeyExportConfig`].
pub fn default_shape_key_export_config() -> ShapeKeyExportConfig {
    ShapeKeyExportConfig { pretty: true, include_basis: true }
}

/// Creates a new, empty [`ShapeKeyExport`] with no basis and no keys.
pub fn new_shape_key_export() -> ShapeKeyExport {
    ShapeKeyExport {
        basis: Vec::new(),
        keys: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds or replaces a shape key (matched by name).
pub fn ske_add_key(export: &mut ShapeKeyExport, key: ShapeKeyEntry) {
    if let Some(existing) = export.keys.iter_mut().find(|k| k.name == key.name) {
        *existing = key;
    } else {
        export.keys.push(key);
    }
}

/// Serialises shape keys as JSON.
pub fn ske_to_json(export: &mut ShapeKeyExport, cfg: &ShapeKeyExportConfig) -> String {
    let indent = if cfg.pretty { "  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };

    let mut out = format!("{{{nl}");

    // basis
    if cfg.include_basis {
        out.push_str(&format!("{indent}\"basis\":[{nl}"));
        let blen = export.basis.len();
        for (i, p) in export.basis.iter().enumerate() {
            let comma = if i + 1 < blen { "," } else { "" };
            out.push_str(&format!(
                "{indent}{indent}[{:.6},{:.6},{:.6}]{comma}{nl}",
                p[0], p[1], p[2]
            ));
        }
        out.push_str(&format!("{indent}],{nl}"));
    }

    // keys
    out.push_str(&format!("{indent}\"keys\":[{nl}"));
    let klen = export.keys.len();
    for (i, k) in export.keys.iter().enumerate() {
        let comma = if i + 1 < klen { "," } else { "" };
        out.push_str(&format!(
            "{indent}{indent}{{\"name\":\"{}\",\"weight_min\":{:.4},\"weight_max\":{:.4},\
             \"delta_count\":{}}}{comma}{nl}",
            k.name,
            k.weight_min,
            k.weight_max,
            k.deltas.len()
        ));
    }
    out.push_str(&format!("{indent}]{nl}}}"));

    export.total_bytes = out.len();
    out
}

/// Serialises shape keys as compact binary.
/// Format per key: `[u16 name_len][name bytes][u32 delta_count][f32 x][f32 y][f32 z]...`
pub fn ske_to_binary(export: &mut ShapeKeyExport, _cfg: &ShapeKeyExportConfig) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    let key_count = export.keys.len() as u32;
    buf.extend_from_slice(&key_count.to_le_bytes());
    for k in &export.keys {
        let name_bytes = k.name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        buf.extend_from_slice(&(k.deltas.len() as u32).to_le_bytes());
        for d in &k.deltas {
            buf.extend_from_slice(&d[0].to_le_bytes());
            buf.extend_from_slice(&d[1].to_le_bytes());
            buf.extend_from_slice(&d[2].to_le_bytes());
        }
    }
    export.total_bytes = buf.len();
    buf
}

/// Returns the number of shape keys stored.
pub fn ske_key_count(export: &ShapeKeyExport) -> usize {
    export.keys.len()
}

/// Returns the number of basis vertices.
pub fn ske_vertex_count(export: &ShapeKeyExport) -> usize {
    export.basis.len()
}

/// Writes JSON to a file path (stub — returns byte count).
pub fn ske_write_to_file(
    export: &mut ShapeKeyExport,
    cfg: &ShapeKeyExportConfig,
    _path: &str,
) -> usize {
    let json = ske_to_json(export, cfg);
    export.total_bytes = json.len();
    export.total_bytes
}

/// Clears all keys and basis vertices.
pub fn ske_clear(export: &mut ShapeKeyExport) {
    export.basis.clear();
    export.keys.clear();
    export.total_bytes = 0;
}

/// Returns the byte count of the last serialised output.
pub fn ske_total_bytes(export: &ShapeKeyExport) -> usize {
    export.total_bytes
}

// ── internal helpers ───────────────────────────────────────────────────────────

fn make_key(name: &str, deltas: Vec<[f32; 3]>) -> ShapeKeyEntry {
    ShapeKeyEntry {
        name: name.to_string(),
        deltas,
        weight_min: 0.0,
        weight_max: 1.0,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = default_shape_key_export_config();
        assert!(cfg.pretty);
        assert!(cfg.include_basis);
    }

    #[test]
    fn new_export_is_empty() {
        let e = new_shape_key_export();
        assert_eq!(ske_key_count(&e), 0);
        assert_eq!(ske_vertex_count(&e), 0);
        assert_eq!(ske_total_bytes(&e), 0);
    }

    #[test]
    fn add_key_increments_count() {
        let mut e = new_shape_key_export();
        ske_add_key(&mut e, make_key("smile", vec![[0.1, 0.0, 0.0]]));
        assert_eq!(ske_key_count(&e), 1);
    }

    #[test]
    fn duplicate_key_name_overwrites() {
        let mut e = new_shape_key_export();
        ske_add_key(&mut e, make_key("smile", vec![[0.1, 0.0, 0.0]]));
        ske_add_key(&mut e, make_key("smile", vec![[0.2, 0.1, 0.0], [0.0, 0.0, 0.0]]));
        assert_eq!(ske_key_count(&e), 1);
        assert_eq!(e.keys[0].deltas.len(), 2);
    }

    #[test]
    fn json_contains_keys_section() {
        let mut e = new_shape_key_export();
        ske_add_key(&mut e, make_key("brow_up", vec![[0.0, 0.05, 0.0]]));
        let cfg = default_shape_key_export_config();
        let json = ske_to_json(&mut e, &cfg);
        assert!(json.contains("\"keys\""));
        assert!(json.contains("brow_up"));
    }

    #[test]
    fn json_contains_basis_when_requested() {
        let mut e = new_shape_key_export();
        e.basis.push([0.0, 1.0, 2.0]);
        let cfg = default_shape_key_export_config();
        let json = ske_to_json(&mut e, &cfg);
        assert!(json.contains("\"basis\""));
    }

    #[test]
    fn json_no_basis_when_disabled() {
        let mut e = new_shape_key_export();
        e.basis.push([0.0, 1.0, 2.0]);
        let mut cfg = default_shape_key_export_config();
        cfg.include_basis = false;
        let json = ske_to_json(&mut e, &cfg);
        assert!(!json.contains("\"basis\""));
    }

    #[test]
    fn binary_header_key_count() {
        let mut e = new_shape_key_export();
        ske_add_key(&mut e, make_key("a", vec![[1.0, 0.0, 0.0]]));
        ske_add_key(&mut e, make_key("b", vec![[0.0, 1.0, 0.0]]));
        let cfg = default_shape_key_export_config();
        let bin = ske_to_binary(&mut e, &cfg);
        let count = u32::from_le_bytes(bin[0..4].try_into().unwrap());
        assert_eq!(count, 2);
    }

    #[test]
    fn write_to_file_sets_total_bytes() {
        let mut e = new_shape_key_export();
        ske_add_key(&mut e, make_key("jaw_open", vec![[0.0, -0.1, 0.0]]));
        let cfg = default_shape_key_export_config();
        let n = ske_write_to_file(&mut e, &cfg, "/tmp/ske.json");
        assert!(n > 0);
        assert_eq!(ske_total_bytes(&e), n);
    }

    #[test]
    fn clear_resets_state() {
        let mut e = new_shape_key_export();
        e.basis.push([0.0, 0.0, 0.0]);
        ske_add_key(&mut e, make_key("smile", vec![[0.1, 0.0, 0.0]]));
        let cfg = default_shape_key_export_config();
        ske_write_to_file(&mut e, &cfg, "/tmp/ske.json");
        ske_clear(&mut e);
        assert_eq!(ske_key_count(&e), 0);
        assert_eq!(ske_vertex_count(&e), 0);
        assert_eq!(ske_total_bytes(&e), 0);
    }
}
