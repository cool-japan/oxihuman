// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Import and export blend shapes in JSON, OBJ-delta, and CSV formats.

use anyhow::{anyhow, bail, Context};

/// A single named blend shape with per-vertex deltas.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendShapeEntry {
    /// Shape name.
    pub name: String,
    /// Per-vertex `[dx, dy, dz]`.
    pub deltas: Vec<[f32; 3]>,
    /// Vertex count (must equal `deltas.len()`).
    pub vertex_count: usize,
}

/// A library of blend shapes sharing the same base mesh vertex count.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendShapeLibraryFile {
    /// Format version (use 1).
    pub version: u32,
    /// Vertex count of the base mesh.
    pub base_vertex_count: usize,
    /// All shapes in the library.
    pub shapes: Vec<BlendShapeEntry>,
}

// ── JSON ───────────────────────────────────────────────────────────────────

/// Serialize a blend shape library to compact JSON.
///
/// Format: `{"version":1,"vertex_count":N,"shapes":[{"name":"...","deltas":[[dx,dy,dz],...]}]}`
#[allow(dead_code)]
pub fn export_blend_shapes_json(lib: &BlendShapeLibraryFile) -> String {
    let mut buf = String::new();
    buf.push_str(&format!(
        "{{\"version\":{},\"vertex_count\":{},\"shapes\":[",
        lib.version, lib.base_vertex_count
    ));
    for (si, shape) in lib.shapes.iter().enumerate() {
        if si > 0 {
            buf.push(',');
        }
        buf.push_str(&format!(
            "{{\"name\":{},\"deltas\":[",
            json_str(&shape.name)
        ));
        for (di, d) in shape.deltas.iter().enumerate() {
            if di > 0 {
                buf.push(',');
            }
            buf.push_str(&format!("[{},{},{}]", d[0], d[1], d[2]));
        }
        buf.push_str("]}");
    }
    buf.push_str("]}");
    buf
}

/// Parse a blend shape library from compact JSON.
#[allow(dead_code)]
pub fn import_blend_shapes_json(json: &str) -> anyhow::Result<BlendShapeLibraryFile> {
    let v: serde_json::Value = serde_json::from_str(json).context("invalid JSON")?;
    let version = v["version"]
        .as_u64()
        .ok_or_else(|| anyhow!("missing version"))? as u32;
    let base_vertex_count = v["vertex_count"]
        .as_u64()
        .ok_or_else(|| anyhow!("missing vertex_count"))? as usize;
    let shapes_arr = v["shapes"]
        .as_array()
        .ok_or_else(|| anyhow!("missing shapes"))?;

    let mut shapes = Vec::new();
    for s in shapes_arr {
        let name = s["name"]
            .as_str()
            .ok_or_else(|| anyhow!("shape missing name"))?
            .to_string();
        let deltas_arr = s["deltas"]
            .as_array()
            .ok_or_else(|| anyhow!("shape missing deltas"))?;
        let mut deltas: Vec<[f32; 3]> = Vec::with_capacity(deltas_arr.len());
        for d in deltas_arr {
            let arr = d.as_array().ok_or_else(|| anyhow!("delta not array"))?;
            if arr.len() < 3 {
                bail!("delta too short");
            }
            deltas.push([
                arr[0].as_f64().ok_or_else(|| anyhow!("delta not f64"))? as f32,
                arr[1].as_f64().ok_or_else(|| anyhow!("delta not f64"))? as f32,
                arr[2].as_f64().ok_or_else(|| anyhow!("delta not f64"))? as f32,
            ]);
        }
        let vertex_count = deltas.len();
        shapes.push(BlendShapeEntry {
            name,
            deltas,
            vertex_count,
        });
    }

    Ok(BlendShapeLibraryFile {
        version,
        base_vertex_count,
        shapes,
    })
}

// ── OBJ delta ─────────────────────────────────────────────────────────────

/// Export one blend shape as an OBJ file where `v = base + delta`.
#[allow(dead_code)]
pub fn export_blend_shape_obj_delta(
    entry: &BlendShapeEntry,
    base_positions: &[[f32; 3]],
) -> String {
    let mut buf = String::new();
    buf.push_str("# OBJ morph target\n");
    buf.push_str(&format!("# shape: {}\n", entry.name));
    for (bp, d) in base_positions.iter().zip(entry.deltas.iter()) {
        let x = bp[0] + d[0];
        let y = bp[1] + d[1];
        let z = bp[2] + d[2];
        buf.push_str(&format!("v {} {} {}\n", x, y, z));
    }
    buf
}

/// Parse an OBJ morph target and compute `delta = parsed_v - base`.
#[allow(dead_code)]
pub fn import_blend_shape_obj_delta(
    obj_src: &str,
    base_positions: &[[f32; 3]],
) -> anyhow::Result<BlendShapeEntry> {
    let mut parsed: Vec<[f32; 3]> = Vec::new();
    for line in obj_src.lines() {
        let line = line.trim();
        if !line.starts_with("v ") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            bail!("malformed v line: {}", line);
        }
        let x: f32 = parts[1].parse().context("x")?;
        let y: f32 = parts[2].parse().context("y")?;
        let z: f32 = parts[3].parse().context("z")?;
        parsed.push([x, y, z]);
    }
    if parsed.len() != base_positions.len() {
        bail!(
            "OBJ vertex count {} != base count {}",
            parsed.len(),
            base_positions.len()
        );
    }
    let deltas: Vec<[f32; 3]> = parsed
        .iter()
        .zip(base_positions.iter())
        .map(|(&p, &b)| [p[0] - b[0], p[1] - b[1], p[2] - b[2]])
        .collect();
    let vertex_count = deltas.len();
    Ok(BlendShapeEntry {
        name: "imported".to_string(),
        deltas,
        vertex_count,
    })
}

// ── CSV ────────────────────────────────────────────────────────────────────

/// Export all blend shapes as CSV: `shape_name,vertex_idx,dx,dy,dz`.
#[allow(dead_code)]
pub fn export_blend_shapes_csv(lib: &BlendShapeLibraryFile) -> String {
    let mut buf = String::from("shape_name,vertex_idx,dx,dy,dz\n");
    for shape in &lib.shapes {
        for (vi, d) in shape.deltas.iter().enumerate() {
            buf.push_str(&format!(
                "{},{},{},{},{}\n",
                shape.name, vi, d[0], d[1], d[2]
            ));
        }
    }
    buf
}

/// Parse blend shapes from CSV.
///
/// Expects header `shape_name,vertex_idx,dx,dy,dz`.
#[allow(dead_code)]
pub fn import_blend_shapes_csv(
    csv: &str,
    vertex_count: usize,
) -> anyhow::Result<BlendShapeLibraryFile> {
    use std::collections::BTreeMap;

    let mut lines = csv.lines();
    // Skip header
    let header = lines.next().unwrap_or("").trim();
    if !header.starts_with("shape_name") {
        bail!("missing CSV header, got: {}", header);
    }

    // name → (vertex_idx → delta)
    let mut map: BTreeMap<String, BTreeMap<usize, [f32; 3]>> = BTreeMap::new();

    for (ln, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 5 {
            bail!("line {}: expected 5 columns, got {}", ln + 2, parts.len());
        }
        let name = parts[0].to_string();
        let vi: usize = parts[1]
            .parse()
            .with_context(|| format!("vertex_idx line {}", ln + 2))?;
        let dx: f32 = parts[2]
            .parse()
            .with_context(|| format!("dx line {}", ln + 2))?;
        let dy: f32 = parts[3]
            .parse()
            .with_context(|| format!("dy line {}", ln + 2))?;
        let dz: f32 = parts[4]
            .parse()
            .with_context(|| format!("dz line {}", ln + 2))?;
        map.entry(name).or_default().insert(vi, [dx, dy, dz]);
    }

    let mut shapes: Vec<BlendShapeEntry> = Vec::new();
    for (name, vmap) in map {
        let mut deltas = vec![[0.0f32; 3]; vertex_count];
        for (vi, d) in vmap {
            if vi < vertex_count {
                deltas[vi] = d;
            }
        }
        shapes.push(BlendShapeEntry {
            name,
            vertex_count,
            deltas,
        });
    }

    Ok(BlendShapeLibraryFile {
        version: 1,
        base_vertex_count: vertex_count,
        shapes,
    })
}

// ── Utilities ──────────────────────────────────────────────────────────────

/// Return min/max/mean delta magnitude statistics as a formatted string.
#[allow(dead_code)]
pub fn blend_shape_stats(entry: &BlendShapeEntry) -> String {
    if entry.deltas.is_empty() {
        return "empty".to_string();
    }
    let mags: Vec<f32> = entry
        .deltas
        .iter()
        .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .collect();
    let min = mags.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = mags.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mean = mags.iter().sum::<f32>() / mags.len() as f32;
    format!("min={:.6} max={:.6} mean={:.6}", min, max, mean)
}

/// Zero out deltas whose magnitude is below `threshold`.
#[allow(dead_code)]
pub fn filter_zero_deltas(entry: &BlendShapeEntry, threshold: f32) -> BlendShapeEntry {
    let deltas = entry
        .deltas
        .iter()
        .map(|&d| {
            let mag = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if mag < threshold {
                [0.0, 0.0, 0.0]
            } else {
                d
            }
        })
        .collect::<Vec<_>>();
    let vertex_count = deltas.len();
    BlendShapeEntry {
        name: entry.name.clone(),
        deltas,
        vertex_count,
    }
}

/// Merge two blend shape libraries (must have the same `base_vertex_count`).
#[allow(dead_code)]
pub fn merge_blend_shape_libraries(
    a: BlendShapeLibraryFile,
    b: BlendShapeLibraryFile,
) -> anyhow::Result<BlendShapeLibraryFile> {
    if a.base_vertex_count != b.base_vertex_count {
        bail!(
            "vertex count mismatch: {} vs {}",
            a.base_vertex_count,
            b.base_vertex_count
        );
    }
    let mut shapes = a.shapes;
    shapes.extend(b.shapes);
    Ok(BlendShapeLibraryFile {
        version: a.version.max(b.version),
        base_vertex_count: a.base_vertex_count,
        shapes,
    })
}

// ── Private helpers ────────────────────────────────────────────────────────

fn json_str(s: &str) -> String {
    // Minimal JSON string escaping
    let mut out = String::from('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lib() -> BlendShapeLibraryFile {
        BlendShapeLibraryFile {
            version: 1,
            base_vertex_count: 2,
            shapes: vec![BlendShapeEntry {
                name: "smile".to_string(),
                deltas: vec![[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]],
                vertex_count: 2,
            }],
        }
    }

    // 1. export_blend_shapes_json round-trip
    #[test]
    fn test_json_roundtrip() {
        let lib = sample_lib();
        let json = export_blend_shapes_json(&lib);
        let imported = import_blend_shapes_json(&json).expect("should succeed");
        assert_eq!(imported.shapes.len(), 1);
        assert_eq!(imported.shapes[0].name, "smile");
        assert_eq!(imported.shapes[0].deltas.len(), 2);
        assert!((imported.shapes[0].deltas[0][0] - 0.1).abs() < 1e-5);
    }

    // 2. JSON contains version field
    #[test]
    fn test_json_contains_version() {
        let lib = sample_lib();
        let json = export_blend_shapes_json(&lib);
        assert!(json.contains("\"version\":1"));
    }

    // 3. import_blend_shapes_json parses name and deltas
    #[test]
    fn test_json_import_name_deltas() {
        let json =
            r#"{"version":1,"vertex_count":1,"shapes":[{"name":"brow","deltas":[[0.5,0.0,0.0]]}]}"#;
        let lib = import_blend_shapes_json(json).expect("should succeed");
        assert_eq!(lib.shapes[0].name, "brow");
        assert!((lib.shapes[0].deltas[0][0] - 0.5).abs() < 1e-5);
    }

    // 4. export_blend_shape_obj_delta has v lines
    #[test]
    fn test_obj_export_has_v_lines() {
        let entry = BlendShapeEntry {
            name: "test".to_string(),
            deltas: vec![[0.1, 0.2, 0.3]],
            vertex_count: 1,
        };
        let base = vec![[1.0f32, 2.0, 3.0]];
        let obj = export_blend_shape_obj_delta(&entry, &base);
        assert!(obj.contains("v "));
    }

    // 5. import_blend_shape_obj_delta recovers deltas
    #[test]
    fn test_obj_import_recovers_deltas() {
        let base = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let entry = BlendShapeEntry {
            name: "test".to_string(),
            deltas: vec![[0.5, -0.5, 0.1], [0.0, 0.2, -0.1]],
            vertex_count: 2,
        };
        let obj = export_blend_shape_obj_delta(&entry, &base);
        let imported = import_blend_shape_obj_delta(&obj, &base).expect("should succeed");
        for (a, b) in entry.deltas.iter().zip(imported.deltas.iter()) {
            assert!((a[0] - b[0]).abs() < 1e-4);
            assert!((a[1] - b[1]).abs() < 1e-4);
            assert!((a[2] - b[2]).abs() < 1e-4);
        }
    }

    // 6. export_csv has correct columns header
    #[test]
    fn test_csv_header_columns() {
        let lib = sample_lib();
        let csv = export_blend_shapes_csv(&lib);
        assert!(csv.starts_with("shape_name,vertex_idx,dx,dy,dz"));
    }

    // 7. import_blend_shapes_csv round-trip
    #[test]
    fn test_csv_roundtrip() {
        let lib = sample_lib();
        let csv = export_blend_shapes_csv(&lib);
        let imported = import_blend_shapes_csv(&csv, 2).expect("should succeed");
        assert_eq!(imported.shapes.len(), 1);
        assert_eq!(imported.shapes[0].name, "smile");
        assert!((imported.shapes[0].deltas[0][0] - 0.1).abs() < 1e-4);
    }

    // 8. blend_shape_stats non-empty returns stats string
    #[test]
    fn test_blend_shape_stats_nonempty() {
        let entry = BlendShapeEntry {
            name: "t".to_string(),
            deltas: vec![[3.0, 4.0, 0.0]],
            vertex_count: 1,
        };
        let s = blend_shape_stats(&entry);
        assert!(s.contains("min="));
        assert!(s.contains("max="));
        assert!(s.contains("mean="));
    }

    // 9. blend_shape_stats empty returns "empty"
    #[test]
    fn test_blend_shape_stats_empty() {
        let entry = BlendShapeEntry {
            name: "e".to_string(),
            deltas: vec![],
            vertex_count: 0,
        };
        assert_eq!(blend_shape_stats(&entry), "empty");
    }

    // 10. filter_zero_deltas removes near-zero
    #[test]
    fn test_filter_zero_deltas_removes() {
        let entry = BlendShapeEntry {
            name: "t".to_string(),
            deltas: vec![[0.0001, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vertex_count: 2,
        };
        let filtered = filter_zero_deltas(&entry, 0.01);
        let mag0 = (filtered.deltas[0][0].powi(2)
            + filtered.deltas[0][1].powi(2)
            + filtered.deltas[0][2].powi(2))
        .sqrt();
        assert!(mag0 < 1e-6);
        assert!((filtered.deltas[1][0] - 1.0).abs() < 1e-6);
    }

    // 11. merge_blend_shape_libraries success
    #[test]
    fn test_merge_success() {
        let a = sample_lib();
        let b = BlendShapeLibraryFile {
            version: 1,
            base_vertex_count: 2,
            shapes: vec![BlendShapeEntry {
                name: "frown".to_string(),
                deltas: vec![[0.0, -0.1, 0.0], [0.0, -0.1, 0.0]],
                vertex_count: 2,
            }],
        };
        let merged = merge_blend_shape_libraries(a, b).expect("should succeed");
        assert_eq!(merged.shapes.len(), 2);
    }

    // 12. merge_blend_shape_libraries fails on vertex count mismatch
    #[test]
    fn test_merge_mismatch_fails() {
        let a = sample_lib();
        let b = BlendShapeLibraryFile {
            version: 1,
            base_vertex_count: 999,
            shapes: vec![],
        };
        assert!(merge_blend_shape_libraries(a, b).is_err());
    }

    // 13. empty library JSON export is valid and importable
    #[test]
    fn test_empty_library_json_export() {
        let lib = BlendShapeLibraryFile {
            version: 1,
            base_vertex_count: 0,
            shapes: vec![],
        };
        let json = export_blend_shapes_json(&lib);
        let imported = import_blend_shapes_json(&json).expect("should succeed");
        assert_eq!(imported.shapes.len(), 0);
    }

    // 14. single shape round-trip through JSON with accurate delta values
    #[test]
    fn test_single_shape_json_roundtrip() {
        let lib = BlendShapeLibraryFile {
            version: 1,
            base_vertex_count: 1,
            shapes: vec![BlendShapeEntry {
                name: "single".to_string(),
                deltas: vec![[1.5, -2.5, 3.77]],
                vertex_count: 1,
            }],
        };
        let json = export_blend_shapes_json(&lib);
        let imported = import_blend_shapes_json(&json).expect("should succeed");
        let d = &imported.shapes[0].deltas[0];
        assert!((d[0] - 1.5).abs() < 1e-4);
        assert!((d[1] - (-2.5)).abs() < 1e-4);
        assert!((d[2] - 3.77).abs() < 1e-3);
    }

    // 15. vertex_count field in JSON parsed correctly
    #[test]
    fn test_json_vertex_count_field() {
        let lib = sample_lib();
        let json = export_blend_shapes_json(&lib);
        let imported = import_blend_shapes_json(&json).expect("should succeed");
        assert_eq!(imported.base_vertex_count, 2);
    }
}
