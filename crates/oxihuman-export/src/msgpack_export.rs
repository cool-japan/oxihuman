//! MessagePack-compatible binary export stub — encodes simple key-value maps to msgpack bytes.

use std::fs;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MsgpackConfig {
    pub compact: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum MsgpackValue {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Bytes(Vec<u8>),
    Array(Vec<MsgpackValue>),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MsgpackMap {
    pub config: MsgpackConfig,
    pub entries: Vec<(String, MsgpackValue)>,
}

#[allow(dead_code)]
pub fn default_msgpack_config() -> MsgpackConfig {
    MsgpackConfig { compact: true }
}

#[allow(dead_code)]
pub fn new_msgpack_map() -> MsgpackMap {
    MsgpackMap {
        config: default_msgpack_config(),
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn msgpack_set(map: &mut MsgpackMap, key: &str, value: MsgpackValue) {
    if let Some(entry) = map.entries.iter_mut().find(|(k, _)| k == key) {
        entry.1 = value;
    } else {
        map.entries.push((key.to_string(), value));
    }
}

#[allow(dead_code)]
pub fn msgpack_get<'a>(map: &'a MsgpackMap, key: &str) -> Option<&'a MsgpackValue> {
    map.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// Encodes a `MsgpackValue` to bytes using a minimal msgpack-compatible encoding.
fn encode_value(val: &MsgpackValue) -> Vec<u8> {
    match val {
        MsgpackValue::Nil => vec![0xc0],
        MsgpackValue::Bool(b) => vec![if *b { 0xc3 } else { 0xc2 }],
        MsgpackValue::Int(n) => {
            if *n >= 0 && *n <= 127 {
                vec![*n as u8]
            } else if *n < 0 && *n >= -32 {
                vec![(*n as i8) as u8]
            } else {
                let mut out = vec![0xd3];
                out.extend_from_slice(&n.to_be_bytes());
                out
            }
        }
        MsgpackValue::Float(f) => {
            let mut out = vec![0xcb];
            out.extend_from_slice(&f.to_bits().to_be_bytes());
            out
        }
        MsgpackValue::Str(s) => {
            let bytes = s.as_bytes();
            let len = bytes.len();
            let mut out = Vec::new();
            if len <= 31 {
                out.push(0xa0 | len as u8);
            } else {
                out.push(0xd9);
                out.push(len as u8);
            }
            out.extend_from_slice(bytes);
            out
        }
        MsgpackValue::Bytes(b) => {
            let len = b.len();
            let mut out = vec![0xc4, len as u8];
            out.extend_from_slice(b);
            out
        }
        MsgpackValue::Array(arr) => {
            let len = arr.len();
            let mut out = Vec::new();
            if len <= 15 {
                out.push(0x90 | len as u8);
            } else {
                out.push(0xdc);
                out.extend_from_slice(&(len as u16).to_be_bytes());
            }
            for item in arr {
                out.extend(encode_value(item));
            }
            out
        }
    }
}

#[allow(dead_code)]
pub fn msgpack_to_bytes(map: &MsgpackMap) -> Vec<u8> {
    let len = map.entries.len();
    let mut out = Vec::new();
    // fixmap header
    if len <= 15 {
        out.push(0x80 | len as u8);
    } else {
        out.push(0xde);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    }
    for (key, val) in &map.entries {
        out.extend(encode_value(&MsgpackValue::Str(key.clone())));
        out.extend(encode_value(val));
    }
    out
}

#[allow(dead_code)]
pub fn msgpack_write_to_file(map: &MsgpackMap, path: &str) -> Result<(), String> {
    let bytes = msgpack_to_bytes(map);
    fs::write(path, bytes).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub fn msgpack_entry_count(map: &MsgpackMap) -> usize {
    map.entries.len()
}

#[allow(dead_code)]
pub fn msgpack_value_type_name(val: &MsgpackValue) -> &'static str {
    match val {
        MsgpackValue::Nil => "nil",
        MsgpackValue::Bool(_) => "bool",
        MsgpackValue::Int(_) => "int",
        MsgpackValue::Float(_) => "float",
        MsgpackValue::Str(_) => "str",
        MsgpackValue::Bytes(_) => "bytes",
        MsgpackValue::Array(_) => "array",
    }
}

#[allow(dead_code)]
pub fn msgpack_from_morph_weights(names: &[&str], weights: &[f32]) -> MsgpackMap {
    let mut map = new_msgpack_map();
    for (name, weight) in names.iter().zip(weights.iter()) {
        msgpack_set(&mut map, name, MsgpackValue::Float(*weight as f64));
    }
    map
}

#[allow(dead_code)]
pub fn msgpack_map_clear(map: &mut MsgpackMap) {
    map.entries.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_msgpack_config();
        assert!(cfg.compact);
    }

    #[test]
    fn test_new_map_empty() {
        let m = new_msgpack_map();
        assert_eq!(msgpack_entry_count(&m), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut m = new_msgpack_map();
        msgpack_set(&mut m, "x", MsgpackValue::Int(42));
        let v = msgpack_get(&m, "x").expect("should succeed");
        assert_eq!(msgpack_value_type_name(v), "int");
    }

    #[test]
    fn test_set_overwrite() {
        let mut m = new_msgpack_map();
        msgpack_set(&mut m, "k", MsgpackValue::Bool(true));
        msgpack_set(&mut m, "k", MsgpackValue::Bool(false));
        assert_eq!(msgpack_entry_count(&m), 1);
        if let MsgpackValue::Bool(b) = msgpack_get(&m, "k").expect("should succeed") {
            assert!(!b);
        }
    }

    #[test]
    fn test_value_type_names() {
        assert_eq!(msgpack_value_type_name(&MsgpackValue::Nil), "nil");
        assert_eq!(msgpack_value_type_name(&MsgpackValue::Bool(true)), "bool");
        assert_eq!(msgpack_value_type_name(&MsgpackValue::Int(0)), "int");
        assert_eq!(msgpack_value_type_name(&MsgpackValue::Float(0.0)), "float");
        assert_eq!(msgpack_value_type_name(&MsgpackValue::Str("".into())), "str");
        assert_eq!(msgpack_value_type_name(&MsgpackValue::Bytes(vec![])), "bytes");
        assert_eq!(msgpack_value_type_name(&MsgpackValue::Array(vec![])), "array");
    }

    #[test]
    fn test_to_bytes_nonempty() {
        let mut m = new_msgpack_map();
        msgpack_set(&mut m, "n", MsgpackValue::Int(1));
        let b = msgpack_to_bytes(&m);
        assert!(!b.is_empty());
        // fixmap with 1 entry has header 0x81
        assert_eq!(b[0], 0x81);
    }

    #[test]
    fn test_from_morph_weights() {
        let names = ["jaw", "brow"];
        let weights = [0.5_f32, 0.8];
        let m = msgpack_from_morph_weights(&names, &weights);
        assert_eq!(msgpack_entry_count(&m), 2);
        let v = msgpack_get(&m, "jaw").expect("should succeed");
        assert_eq!(msgpack_value_type_name(v), "float");
    }

    #[test]
    fn test_map_clear() {
        let mut m = new_msgpack_map();
        msgpack_set(&mut m, "a", MsgpackValue::Nil);
        msgpack_map_clear(&mut m);
        assert_eq!(msgpack_entry_count(&m), 0);
    }

    #[test]
    fn test_get_missing_returns_none() {
        let m = new_msgpack_map();
        assert!(msgpack_get(&m, "nope").is_none());
    }
}
