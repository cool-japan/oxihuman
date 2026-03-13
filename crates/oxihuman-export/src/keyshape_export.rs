// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A key shape (blend shape target with a name and weight).
#[allow(dead_code)]
#[derive(Clone)]
pub struct KeyShape {
    pub name: String,
    pub weight: f32,
    pub deltas: Vec<[f32; 3]>,
}

/// Key shape export bundle.
#[allow(dead_code)]
#[derive(Default)]
pub struct KeyShapeExport {
    pub shapes: Vec<KeyShape>,
}

/// Create a new key shape export.
#[allow(dead_code)]
pub fn new_keyshape_export() -> KeyShapeExport {
    KeyShapeExport::default()
}

/// Add a key shape.
#[allow(dead_code)]
pub fn add_keyshape(export: &mut KeyShapeExport, name: &str, weight: f32, deltas: Vec<[f32; 3]>) {
    export.shapes.push(KeyShape {
        name: name.to_string(),
        weight,
        deltas,
    });
}

/// Count shapes.
#[allow(dead_code)]
pub fn keyshape_count(export: &KeyShapeExport) -> usize {
    export.shapes.len()
}

/// Find shape by name.
#[allow(dead_code)]
pub fn find_keyshape<'a>(export: &'a KeyShapeExport, name: &str) -> Option<&'a KeyShape> {
    export.shapes.iter().find(|s| s.name == name)
}

/// Total deltas across all shapes.
#[allow(dead_code)]
pub fn total_keyshape_deltas(export: &KeyShapeExport) -> usize {
    export.shapes.iter().map(|s| s.deltas.len()).sum()
}

/// Weighted blend of two key shapes (returns new delta list).
#[allow(dead_code)]
pub fn blend_keyshapes(a: &KeyShape, b: &KeyShape, t: f32) -> Vec<[f32; 3]> {
    let n = a.deltas.len().min(b.deltas.len());
    (0..n)
        .map(|i| {
            let da = a.deltas[i];
            let db = b.deltas[i];
            [
                da[0] + t * (db[0] - da[0]),
                da[1] + t * (db[1] - da[1]),
                da[2] + t * (db[2] - da[2]),
            ]
        })
        .collect()
}

/// Validate weights are in [0, 1].
#[allow(dead_code)]
pub fn validate_keyshape_weights(export: &KeyShapeExport) -> bool {
    export
        .shapes
        .iter()
        .all(|s| (0.0..=1.0).contains(&s.weight))
}

/// Normalize delta magnitude for a shape.
#[allow(dead_code)]
pub fn max_keyshape_delta(shape: &KeyShape) -> f32 {
    shape
        .deltas
        .iter()
        .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .fold(0.0_f32, f32::max)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn keyshape_to_json(export: &KeyShapeExport) -> String {
    format!(
        r#"{{"keyshapes":{},"total_deltas":{}}}"#,
        export.shapes.len(),
        total_keyshape_deltas(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut e = new_keyshape_export();
        add_keyshape(&mut e, "smile", 0.5, vec![[0.1, 0.0, 0.0]; 5]);
        assert_eq!(keyshape_count(&e), 1);
    }

    #[test]
    fn find_shape() {
        let mut e = new_keyshape_export();
        add_keyshape(&mut e, "blink", 1.0, vec![]);
        assert!(find_keyshape(&e, "blink").is_some());
    }

    #[test]
    fn find_missing() {
        let e = new_keyshape_export();
        assert!(find_keyshape(&e, "x").is_none());
    }

    #[test]
    fn total_deltas() {
        let mut e = new_keyshape_export();
        add_keyshape(&mut e, "a", 0.5, vec![[0.0; 3]; 3]);
        add_keyshape(&mut e, "b", 0.5, vec![[0.0; 3]; 4]);
        assert_eq!(total_keyshape_deltas(&e), 7);
    }

    #[test]
    fn blend_midpoint() {
        let a = KeyShape {
            name: "a".to_string(),
            weight: 0.0,
            deltas: vec![[0.0, 0.0, 0.0]],
        };
        let b = KeyShape {
            name: "b".to_string(),
            weight: 1.0,
            deltas: vec![[2.0, 0.0, 0.0]],
        };
        let blended = blend_keyshapes(&a, &b, 0.5);
        assert!((blended[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn validate_valid() {
        let mut e = new_keyshape_export();
        add_keyshape(&mut e, "x", 0.7, vec![]);
        assert!(validate_keyshape_weights(&e));
    }

    #[test]
    fn validate_invalid() {
        let mut e = new_keyshape_export();
        add_keyshape(&mut e, "x", 1.5, vec![]);
        assert!(!validate_keyshape_weights(&e));
    }

    #[test]
    fn max_delta() {
        let s = KeyShape {
            name: "x".to_string(),
            weight: 1.0,
            deltas: vec![[3.0, 4.0, 0.0]],
        };
        assert!((max_keyshape_delta(&s) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn json_has_count() {
        let e = new_keyshape_export();
        let j = keyshape_to_json(&e);
        assert!(j.contains("\"keyshapes\":0"));
    }

    #[test]
    fn empty_total_deltas() {
        let e = new_keyshape_export();
        assert_eq!(total_keyshape_deltas(&e), 0);
    }
}
