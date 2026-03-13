// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OxiHuman-specific metadata to embed in GLTF asset.extras.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxiHumanMeta {
    /// Generator identifier.
    pub generator: String,
    /// Semantic version string.
    pub version: String,
    /// ISO 8601 timestamp (UTC) when exported.
    pub exported_at: String,
    /// Body measurements at export time.
    pub measurements: Option<MeasurementsMeta>,
    /// Active preset name, if any.
    pub preset: Option<String>,
    /// Key morph parameters (height/weight/muscle/age).
    pub params: Option<ParamsMeta>,
    /// Active expression preset name.
    pub expression: Option<String>,
    /// Number of morph targets applied.
    pub target_count: Option<usize>,
    /// Policy profile used ("Strict" or "Standard").
    pub policy: Option<String>,
    /// Arbitrary extra key-value pairs.
    pub extra: std::collections::HashMap<String, Value>,
}

/// Body measurements at export time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementsMeta {
    /// Height in centimetres.
    pub height_cm: Option<f32>,
    /// Weight in kilograms.
    pub weight_kg: Option<f32>,
    /// Chest circumference in centimetres.
    pub chest_cm: Option<f32>,
    /// Waist circumference in centimetres.
    pub waist_cm: Option<f32>,
    /// Hips circumference in centimetres.
    pub hips_cm: Option<f32>,
}

/// Key morph parameters snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamsMeta {
    /// Height blend factor [0, 1].
    pub height: f32,
    /// Weight blend factor [0, 1].
    pub weight: f32,
    /// Muscle blend factor [0, 1].
    pub muscle: f32,
    /// Age blend factor [0, 1].
    pub age: f32,
}

impl OxiHumanMeta {
    /// Create a minimal metadata block with just the generator info.
    pub fn minimal() -> Self {
        Self {
            generator: "oxihuman-export".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            exported_at: current_timestamp(),
            measurements: None,
            preset: None,
            params: None,
            expression: None,
            target_count: None,
            policy: None,
            extra: Default::default(),
        }
    }

    /// Convert to a `serde_json::Value` for embedding in GLTF.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// Parse from a `serde_json::Value`.
    pub fn from_json(v: &Value) -> Option<Self> {
        serde_json::from_value(v.clone()).ok()
    }

    /// Set a parameter block from raw f32 values.
    pub fn with_params(mut self, height: f32, weight: f32, muscle: f32, age: f32) -> Self {
        self.params = Some(ParamsMeta {
            height,
            weight,
            muscle,
            age,
        });
        self
    }

    /// Set measurements.
    pub fn with_measurements(mut self, m: MeasurementsMeta) -> Self {
        self.measurements = Some(m);
        self
    }
}

/// Returns current UTC timestamp in ISO 8601 format without pulling in chrono.
/// Uses `std::time::SystemTime`.
fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, sec) = unix_secs_to_datetime(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, mi, sec)
}

fn unix_secs_to_datetime(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let sec = (secs % 60) as u32;
    let min = ((secs / 60) % 60) as u32;
    let hour = ((secs / 3600) % 24) as u32;
    let days = secs / 86400;

    // Walk forward year by year from the Unix epoch (1970).
    let mut y = 1970u32;
    let mut d = days;
    loop {
        let dy = if is_leap(y) { 366u64 } else { 365u64 };
        if d < dy {
            break;
        }
        d -= dy;
        y += 1;
    }

    let months = [
        31u32,
        if is_leap(y) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut mo = 1u32;
    for &ml in &months {
        if d < ml as u64 {
            break;
        }
        d -= ml as u64;
        mo += 1;
    }

    (y, mo, d as u32 + 1, hour, min, sec)
}

fn is_leap(y: u32) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_has_generator() {
        assert_eq!(OxiHumanMeta::minimal().generator, "oxihuman-export");
    }

    #[test]
    fn to_json_has_generator_key() {
        let meta = OxiHumanMeta::minimal();
        assert!(meta.to_json()["generator"].as_str().is_some());
    }

    #[test]
    fn from_json_roundtrip() {
        let meta = OxiHumanMeta::minimal();
        let json = meta.to_json();
        let back = OxiHumanMeta::from_json(&json).expect("deserialization failed");
        assert_eq!(back.generator, "oxihuman-export");
    }

    #[test]
    fn with_params_sets_params() {
        let meta = OxiHumanMeta::minimal().with_params(0.5, 0.4, 0.3, 0.2);
        assert!(meta.params.is_some());
    }

    #[test]
    fn timestamp_is_nonempty() {
        assert!(!OxiHumanMeta::minimal().exported_at.is_empty());
    }

    #[test]
    fn timestamp_contains_t() {
        assert!(OxiHumanMeta::minimal().exported_at.contains('T'));
    }
}
