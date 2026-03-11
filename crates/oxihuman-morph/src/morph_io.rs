// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Morph I/O: serialize/deserialize morph weights to/from simple format.

#[allow(dead_code)]
pub struct MorphIoRecord {
    pub name: String,
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
pub fn mio_serialize(records: &[MorphIoRecord]) -> String {
    records.iter().map(|r| {
        let mut parts = vec![r.name.clone()];
        parts.extend(r.weights.iter().map(|w| format!("{}", w)));
        parts.join(",")
    }).collect::<Vec<_>>().join("\n")
}

#[allow(dead_code)]
pub fn mio_parse_line(line: &str) -> Option<MorphIoRecord> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.is_empty() || parts[0].is_empty() {
        return None;
    }
    let name = parts[0].to_string();
    let weights: Vec<f32> = parts[1..].iter().filter_map(|s| s.parse().ok()).collect();
    Some(MorphIoRecord { name, weights })
}

#[allow(dead_code)]
pub fn mio_deserialize(data: &str) -> Vec<MorphIoRecord> {
    data.lines().filter(|l| !l.trim().is_empty()).filter_map(mio_parse_line).collect()
}

#[allow(dead_code)]
pub fn mio_record_count(data: &str) -> usize {
    data.lines().filter(|l| !l.trim().is_empty()).count()
}

#[allow(dead_code)]
pub fn mio_weight_count(record: &MorphIoRecord) -> usize {
    record.weights.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let records = vec![MorphIoRecord { name: "smile".to_string(), weights: vec![1.0, 0.5] }];
        let s = mio_serialize(&records);
        assert!(s.contains("smile"));
    }

    #[test]
    fn test_parse_line() {
        let r = mio_parse_line("jaw,0.5,1.0").unwrap();
        assert_eq!(r.name, "jaw");
        assert_eq!(r.weights.len(), 2);
    }

    #[test]
    fn test_parse_empty_returns_none() {
        assert!(mio_parse_line("").is_none());
    }

    #[test]
    fn test_deserialize() {
        let data = "smile,1.0\njaw,0.5";
        let records = mio_deserialize(data);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_deserialize_skips_empty_lines() {
        let data = "smile,1.0\n\njaw,0.5";
        let records = mio_deserialize(data);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_record_count() {
        let data = "a,1.0\nb,0.5\nc,0.2";
        assert_eq!(mio_record_count(data), 3);
    }

    #[test]
    fn test_weight_count() {
        let r = MorphIoRecord { name: "x".to_string(), weights: vec![0.1, 0.2, 0.3] };
        assert_eq!(mio_weight_count(&r), 3);
    }

    #[test]
    fn test_roundtrip() {
        let records = vec![MorphIoRecord { name: "test".to_string(), weights: vec![0.5] }];
        let s = mio_serialize(&records);
        let back = mio_deserialize(&s);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].name, "test");
    }
}
