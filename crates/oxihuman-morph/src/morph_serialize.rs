#![allow(dead_code)]
//! Morph serialization: serialize and deserialize morph state to/from bytes and JSON.

use std::collections::HashMap;

/// A serializable morph state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphSerialize {
    weights: HashMap<String, f32>,
    version: u32,
}

/// Serialize a morph state (weight map) to a string representation.
#[allow(dead_code)]
pub fn serialize_morph_state(weights: &HashMap<String, f32>) -> String {
    let mut pairs: Vec<(&String, &f32)> = weights.iter().collect();
    pairs.sort_by_key(|(k, _)| (*k).clone());
    let entries: Vec<String> = pairs.iter().map(|(k, v)| format!("{k}={v}")).collect();
    entries.join(";")
}

/// Deserialize a morph state from a string representation.
#[allow(dead_code)]
pub fn deserialize_morph_state(data: &str) -> HashMap<String, f32> {
    let mut result = HashMap::new();
    if data.is_empty() {
        return result;
    }
    for pair in data.split(';') {
        if let Some((key, val_str)) = pair.split_once('=') {
            if let Ok(val) = val_str.parse::<f32>() {
                result.insert(key.to_string(), val);
            }
        }
    }
    result
}

/// Serialize to a byte vector (simple format: 4-byte version + key-value pairs).
#[allow(dead_code)]
pub fn serialize_to_bytes(weights: &HashMap<String, f32>) -> Vec<u8> {
    let s = serialize_morph_state(weights);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&1u32.to_le_bytes()); // version
    bytes.extend_from_slice(s.as_bytes());
    bytes
}

/// Deserialize from a byte vector.
#[allow(dead_code)]
pub fn deserialize_from_bytes(data: &[u8]) -> HashMap<String, f32> {
    if data.len() < 4 {
        return HashMap::new();
    }
    let s = std::str::from_utf8(&data[4..]).unwrap_or("");
    deserialize_morph_state(s)
}

/// Return the serialized size in bytes.
#[allow(dead_code)]
pub fn serialized_size(weights: &HashMap<String, f32>) -> usize {
    4 + serialize_morph_state(weights).len()
}

/// Compute a simple checksum of the serialized data.
#[allow(dead_code)]
pub fn serialized_checksum(weights: &HashMap<String, f32>) -> u32 {
    let bytes = serialize_to_bytes(weights);
    bytes.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32))
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn serialize_to_json(weights: &HashMap<String, f32>) -> String {
    let mut pairs: Vec<(&String, &f32)> = weights.iter().collect();
    pairs.sort_by_key(|(k, _)| (*k).clone());
    let entries: Vec<String> = pairs.iter().map(|(k, v)| format!("\"{}\":{}", k, v)).collect();
    format!("{{{}}}", entries.join(","))
}

/// Return the serialization version.
#[allow(dead_code)]
pub fn serialize_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_empty() {
        let weights = HashMap::new();
        assert_eq!(serialize_morph_state(&weights), "");
    }

    #[test]
    fn test_serialize_single() {
        let mut weights = HashMap::new();
        weights.insert("smile".to_string(), 0.5);
        let s = serialize_morph_state(&weights);
        assert!(s.contains("smile=0.5"));
    }

    #[test]
    fn test_deserialize() {
        let data = "smile=0.5;frown=0.3";
        let result = deserialize_morph_state(data);
        assert!((result["smile"] - 0.5).abs() < 1e-6);
        assert!((result["frown"] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_deserialize_empty() {
        let result = deserialize_morph_state("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_roundtrip_bytes() {
        let mut weights = HashMap::new();
        weights.insert("x".to_string(), 0.7);
        let bytes = serialize_to_bytes(&weights);
        let result = deserialize_from_bytes(&bytes);
        assert!((result["x"] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_deserialize_bytes_short() {
        let result = deserialize_from_bytes(&[0, 1]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_serialized_size() {
        let mut weights = HashMap::new();
        weights.insert("a".to_string(), 1.0);
        let size = serialized_size(&weights);
        assert!(size > 4);
    }

    #[test]
    fn test_checksum_deterministic() {
        let mut weights = HashMap::new();
        weights.insert("a".to_string(), 1.0);
        let c1 = serialized_checksum(&weights);
        let c2 = serialized_checksum(&weights);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_to_json() {
        let mut weights = HashMap::new();
        weights.insert("x".to_string(), 0.5);
        let json = serialize_to_json(&weights);
        assert!(json.contains("\"x\":0.5"));
    }

    #[test]
    fn test_version() {
        assert_eq!(serialize_version(), 1);
    }
}
