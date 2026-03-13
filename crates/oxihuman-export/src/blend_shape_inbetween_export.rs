// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// An inbetween blend shape key.
#[allow(dead_code)]
pub struct InbetweenKey {
    pub name: String,
    pub target_weight: f32,
    pub deltas: Vec<[f32; 3]>,
}

/// Export bundle for blend shape inbetweens.
#[allow(dead_code)]
pub struct BlendShapeInbetweenExport {
    pub base_name: String,
    pub keys: Vec<InbetweenKey>,
}

/// Create a new inbetween export.
#[allow(dead_code)]
pub fn new_bsi_export(base_name: &str) -> BlendShapeInbetweenExport {
    BlendShapeInbetweenExport {
        base_name: base_name.to_string(),
        keys: Vec::new(),
    }
}

/// Add an inbetween key.
#[allow(dead_code)]
pub fn add_inbetween_key(
    export: &mut BlendShapeInbetweenExport,
    name: &str,
    target_weight: f32,
    deltas: Vec<[f32; 3]>,
) {
    export.keys.push(InbetweenKey {
        name: name.to_string(),
        target_weight,
        deltas,
    });
}

/// Count inbetween keys.
#[allow(dead_code)]
pub fn inbetween_key_count(export: &BlendShapeInbetweenExport) -> usize {
    export.keys.len()
}

/// Total delta vertices across all keys.
#[allow(dead_code)]
pub fn total_inbetween_deltas(export: &BlendShapeInbetweenExport) -> usize {
    export.keys.iter().map(|k| k.deltas.len()).sum()
}

/// Find key by name.
#[allow(dead_code)]
pub fn find_inbetween<'a>(
    export: &'a BlendShapeInbetweenExport,
    name: &str,
) -> Option<&'a InbetweenKey> {
    export.keys.iter().find(|k| k.name == name)
}

/// Maximum magnitude delta in a key.
#[allow(dead_code)]
pub fn max_delta_magnitude_bsi(key: &InbetweenKey) -> f32 {
    key.deltas
        .iter()
        .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .fold(0.0_f32, f32::max)
}

/// Validate that keys are sorted by target weight.
#[allow(dead_code)]
pub fn keys_sorted_by_weight(export: &BlendShapeInbetweenExport) -> bool {
    export
        .keys
        .windows(2)
        .all(|w| w[0].target_weight <= w[1].target_weight)
}

/// Interpolate between two inbetween keys.
#[allow(dead_code)]
pub fn interpolate_inbetween(k0: &InbetweenKey, k1: &InbetweenKey, t: f32) -> Vec<[f32; 3]> {
    let n = k0.deltas.len().min(k1.deltas.len());
    (0..n)
        .map(|i| {
            let d0 = k0.deltas[i];
            let d1 = k1.deltas[i];
            [
                d0[0] + t * (d1[0] - d0[0]),
                d0[1] + t * (d1[1] - d0[1]),
                d0[2] + t * (d1[2] - d0[2]),
            ]
        })
        .collect()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn bsi_to_json(export: &BlendShapeInbetweenExport) -> String {
    format!(
        r#"{{"base":"{}","keys":{}}}"#,
        export.base_name,
        export.keys.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_add() {
        let mut e = new_bsi_export("mouth_open");
        add_inbetween_key(&mut e, "half", 0.5, vec![[0.0, 0.1, 0.0]; 4]);
        assert_eq!(inbetween_key_count(&e), 1);
    }

    #[test]
    fn total_deltas() {
        let mut e = new_bsi_export("eye");
        add_inbetween_key(&mut e, "k1", 0.5, vec![[0.0; 3]; 3]);
        add_inbetween_key(&mut e, "k2", 1.0, vec![[0.0; 3]; 3]);
        assert_eq!(total_inbetween_deltas(&e), 6);
    }

    #[test]
    fn find_key() {
        let mut e = new_bsi_export("brow");
        add_inbetween_key(&mut e, "mid", 0.5, vec![]);
        assert!(find_inbetween(&e, "mid").is_some());
    }

    #[test]
    fn max_delta_mag() {
        let k = InbetweenKey {
            name: "k".to_string(),
            target_weight: 1.0,
            deltas: vec![[3.0, 4.0, 0.0]],
        };
        assert!((max_delta_magnitude_bsi(&k) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn sorted_by_weight() {
        let mut e = new_bsi_export("test");
        add_inbetween_key(&mut e, "a", 0.25, vec![]);
        add_inbetween_key(&mut e, "b", 0.75, vec![]);
        assert!(keys_sorted_by_weight(&e));
    }

    #[test]
    fn not_sorted() {
        let mut e = new_bsi_export("test");
        add_inbetween_key(&mut e, "a", 0.75, vec![]);
        add_inbetween_key(&mut e, "b", 0.25, vec![]);
        assert!(!keys_sorted_by_weight(&e));
    }

    #[test]
    fn interpolate_midpoint() {
        let k0 = InbetweenKey {
            name: "k0".to_string(),
            target_weight: 0.0,
            deltas: vec![[0.0; 3]],
        };
        let k1 = InbetweenKey {
            name: "k1".to_string(),
            target_weight: 1.0,
            deltas: vec![[2.0, 0.0, 0.0]],
        };
        let mid = interpolate_inbetween(&k0, &k1, 0.5);
        assert!((mid[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn json_has_base() {
        let e = new_bsi_export("cheek");
        let j = bsi_to_json(&e);
        assert!(j.contains("\"base\":\"cheek\""));
    }

    #[test]
    fn empty_keys() {
        let e = new_bsi_export("x");
        assert_eq!(inbetween_key_count(&e), 0);
    }

    #[test]
    fn find_missing() {
        let e = new_bsi_export("x");
        assert!(find_inbetween(&e, "y").is_none());
    }
}
