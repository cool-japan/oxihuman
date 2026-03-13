// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use oxihuman_mesh::measurements::{compute_measurements, BodyMeasurements};
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_morph::params::ParamState;
use serde_json::{json, Value};

/// Serialize ParamState to a JSON value for profile export/import.
pub fn export_params(params: &ParamState) -> Value {
    let mut obj = json!({
        "height": params.height,
        "weight": params.weight,
        "muscle": params.muscle,
        "age":    params.age,
    });

    if !params.extra.is_empty() {
        let map = match obj.as_object_mut() {
            Some(m) => m,
            None => return obj,
        };
        for (k, v) in &params.extra {
            map.insert(k.clone(), json!(v));
        }
    }

    obj
}

/// Deserialize ParamState from a JSON value.
pub fn import_params(val: &Value) -> anyhow::Result<ParamState> {
    use anyhow::Context;
    let height = val["height"].as_f64().context("missing height")? as f32;
    let weight = val["weight"].as_f64().context("missing weight")? as f32;
    let muscle = val["muscle"].as_f64().context("missing muscle")? as f32;
    let age = val["age"].as_f64().context("missing age")? as f32;

    let mut extra = std::collections::HashMap::new();
    if let Some(map) = val.as_object() {
        for (k, v) in map {
            if !matches!(k.as_str(), "height" | "weight" | "muscle" | "age") {
                if let Some(f) = v.as_f64() {
                    extra.insert(k.clone(), f as f32);
                }
            }
        }
    }

    Ok(ParamState {
        height,
        weight,
        muscle,
        age,
        extra,
    })
}

/// Serialize BodyMeasurements to JSON.
pub fn export_measurements(meas: &BodyMeasurements) -> Value {
    serde_json::json!({
        "total_height":    meas.total_height,
        "max_width":       meas.max_width,
        "max_depth":       meas.max_depth,
        "torso_height":    meas.torso_height,
        "shoulder_width":  meas.shoulder_width,
        "waist_width":     meas.waist_width,
        "hip_width":       meas.hip_width,
    })
}

/// Compute and export measurements from a mesh to JSON.
pub fn export_mesh_measurements(mesh: &MeshBuffers) -> Value {
    match compute_measurements(mesh) {
        Some(m) => export_measurements(&m),
        None => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::params::ParamState;

    #[test]
    fn round_trip_params() {
        let p = ParamState::new(0.7, 0.3, 0.5, 0.2);
        let val = export_params(&p);
        let p2 = import_params(&val).expect("should succeed");
        assert!((p.height - p2.height).abs() < 1e-5);
        assert!((p.weight - p2.weight).abs() < 1e-5);
        assert!((p.muscle - p2.muscle).abs() < 1e-5);
        assert!((p.age - p2.age).abs() < 1e-5);
    }

    #[test]
    fn export_has_all_fields() {
        let p = ParamState::default();
        let val = export_params(&p);
        assert!(val["height"].is_number());
        assert!(val["weight"].is_number());
        assert!(val["muscle"].is_number());
        assert!(val["age"].is_number());
    }

    #[test]
    fn measurements_json_has_all_fields() {
        use oxihuman_mesh::measurements::BodyMeasurements;
        let m = BodyMeasurements {
            total_height: 1.7,
            max_width: 0.5,
            max_depth: 0.3,
            torso_height: 0.5,
            shoulder_width: 0.45,
            waist_width: 0.35,
            hip_width: 0.4,
        };
        let v = super::export_measurements(&m);
        assert!(v["total_height"].is_number());
        assert!(v["shoulder_width"].is_number());
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn params_json_round_trip(
            h in 0.0f32..=1.0f32,
            w in 0.0f32..=1.0f32,
            m in 0.0f32..=1.0f32,
            a in 0.0f32..=1.0f32,
        ) {
            use oxihuman_morph::params::ParamState;
            let p = ParamState::new(h, w, m, a);
            let val = super::export_params(&p);
            let p2 = super::import_params(&val).expect("should succeed");
            prop_assert!((p.height - p2.height).abs() < 1e-4);
            prop_assert!((p.weight - p2.weight).abs() < 1e-4);
            prop_assert!((p.muscle - p2.muscle).abs() < 1e-4);
            prop_assert!((p.age    - p2.age).abs()    < 1e-4);
        }
    }
}
