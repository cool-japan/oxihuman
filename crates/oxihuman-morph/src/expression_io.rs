// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! ExpressionIO — serialise / deserialise expression presets.

#![allow(dead_code)]

/// Serialised representation of a saved expression.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionSave {
    pub name: String,
    pub version: u32,
    pub weights: Vec<f32>,
}

/// Loaded representation of an expression preset.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionLoad {
    pub name: String,
    pub version: u32,
    pub weights: Vec<f32>,
    pub checksum: u32,
}

/// Serialise an `ExpressionSave` to a simple JSON string.
#[allow(dead_code)]
pub fn expression_to_json(save: &ExpressionSave) -> String {
    let weights_str: Vec<String> = save.weights.iter().map(|w| format!("{w:.6}")).collect();
    format!(
        r#"{{"name":"{name}","version":{ver},"weights":[{w}]}}"#,
        name = save.name,
        ver = save.version,
        w = weights_str.join(",")
    )
}

/// Parse a JSON string back into an `ExpressionLoad`.
///
/// Extracts the `name` and `version` fields together with the full
/// `weights` array. Returns `None` if the string is not a recognised format.
#[allow(dead_code)]
pub fn expression_from_json(json: &str) -> Option<ExpressionLoad> {
    let name = extract_json_string(json, "name")?;
    let version_str = extract_json_number(json, "version")?;
    let version: u32 = version_str.parse().ok()?;
    let weights = extract_json_f32_array(json, "weights").unwrap_or_default();
    let checksum = simple_checksum(json.as_bytes());
    Some(ExpressionLoad {
        name,
        version,
        weights,
        checksum,
    })
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{key}\":");
    let pos = json.find(&search)?;
    let after = &json[pos + search.len()..];
    let start = after.find('"')? + 1;
    let end = after[start + 1..].find('"')? + start + 1;
    Some(after[start..end].to_owned())
}

fn extract_json_number(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{key}\":");
    let pos = json.find(&search)?;
    let after = json[pos + search.len()..].trim_start();
    let end = after
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len());
    Some(after[..end].to_owned())
}

/// Extract a JSON array of `f32` values associated with `key`.
///
/// Locates `"<key>":`, then the next `[` and its matching `]`, and parses each
/// comma-separated, non-empty token as `f32`. An empty array yields
/// `Some(vec![])`; a missing key or missing brackets yields `None`.
fn extract_json_f32_array(json: &str, key: &str) -> Option<Vec<f32>> {
    let search = format!("\"{key}\":");
    let pos = json.find(&search)?;
    let after = &json[pos + search.len()..];
    let lb = after.find('[')?;
    let rb = after[lb + 1..].find(']')? + lb + 1;
    let inner = &after[lb + 1..rb];
    let mut out = Vec::new();
    for tok in inner.split(',') {
        let t = tok.trim();
        if t.is_empty() {
            continue;
        }
        out.push(t.parse::<f32>().ok()?);
    }
    Some(out)
}

/// Stub: write expression data to a byte buffer (simulates file I/O).
#[allow(dead_code)]
pub fn save_expression_stub(save: &ExpressionSave) -> Vec<u8> {
    expression_to_json(save).into_bytes()
}

/// Stub: read an expression from a byte buffer.
#[allow(dead_code)]
pub fn load_expression_stub(data: &[u8]) -> Option<ExpressionLoad> {
    let json = std::str::from_utf8(data).ok()?;
    expression_from_json(json)
}

/// Serialise to raw bytes (JSON-encoded).
#[allow(dead_code)]
pub fn expression_to_bytes(save: &ExpressionSave) -> Vec<u8> {
    save_expression_stub(save)
}

/// Compute a simple additive checksum over the byte representation.
#[allow(dead_code)]
pub fn expression_checksum(save: &ExpressionSave) -> u32 {
    simple_checksum(&save_expression_stub(save))
}

fn simple_checksum(data: &[u8]) -> u32 {
    data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32))
}

/// Return the expression format version.
#[allow(dead_code)]
pub fn expression_version(save: &ExpressionSave) -> u32 {
    save.version
}

/// Extract the expression name from a JSON string.
#[allow(dead_code)]
pub fn expression_name_from_json(json: &str) -> Option<String> {
    extract_json_string(json, "name")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_save() -> ExpressionSave {
        ExpressionSave {
            name: "smile".to_owned(),
            version: 1,
            weights: vec![0.5, 0.3],
        }
    }

    #[test]
    fn test_expression_to_json_contains_name() {
        let s = make_save();
        let j = expression_to_json(&s);
        assert!(j.contains("smile"));
    }

    #[test]
    fn test_expression_to_json_contains_version() {
        let s = make_save();
        let j = expression_to_json(&s);
        assert!(j.contains("\"version\":1"));
    }

    #[test]
    fn test_expression_from_json_roundtrip_name() {
        let s = make_save();
        let j = expression_to_json(&s);
        let loaded = expression_from_json(&j).expect("should succeed");
        assert_eq!(loaded.name, "smile");
    }

    #[test]
    fn test_expression_from_json_roundtrip_version() {
        let s = make_save();
        let j = expression_to_json(&s);
        let loaded = expression_from_json(&j).expect("should succeed");
        assert_eq!(loaded.version, 1);
    }

    #[test]
    fn test_save_load_stub_roundtrip() {
        let s = make_save();
        let bytes = save_expression_stub(&s);
        let loaded = load_expression_stub(&bytes).expect("should succeed");
        assert_eq!(loaded.name, "smile");
    }

    #[test]
    fn test_expression_checksum_nonzero() {
        let s = make_save();
        assert!(expression_checksum(&s) > 0);
    }

    #[test]
    fn test_expression_version() {
        let s = make_save();
        assert_eq!(expression_version(&s), 1);
    }

    #[test]
    fn test_expression_name_from_json() {
        let s = make_save();
        let j = expression_to_json(&s);
        assert_eq!(
            expression_name_from_json(&j).expect("should succeed"),
            "smile"
        );
    }

    #[test]
    fn test_expression_to_bytes_nonempty() {
        let s = make_save();
        assert!(!expression_to_bytes(&s).is_empty());
    }

    #[test]
    fn test_expression_from_json_roundtrip_weights() {
        let s = ExpressionSave {
            name: "smile".to_owned(),
            version: 2,
            weights: vec![0.5, 0.3, 0.125, 1.0],
        };
        let j = expression_to_json(&s);
        let loaded = expression_from_json(&j).expect("should parse");
        assert_eq!(loaded.weights.len(), 4);
        for (a, b) in loaded.weights.iter().zip(s.weights.iter()) {
            assert!((a - b).abs() < 1e-4, "weight mismatch: {a} vs {b}");
        }
    }

    #[test]
    fn test_expression_from_json_empty_weights() {
        let s = ExpressionSave {
            name: "neutral".to_owned(),
            version: 1,
            weights: vec![],
        };
        let j = expression_to_json(&s);
        let loaded = expression_from_json(&j).expect("should parse");
        assert!(loaded.weights.is_empty());
    }
}
