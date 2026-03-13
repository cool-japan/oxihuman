// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pivot point export: object pivot/origin point data.

/// A named pivot point.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PivotPoint {
    pub name: String,
    pub position: [f32; 3],
    pub orientation: [f32; 4],
}

/// Pivot point collection export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PivotPointExport {
    pub pivots: Vec<PivotPoint>,
}

#[allow(dead_code)]
pub fn new_pivot_point_export() -> PivotPointExport {
    PivotPointExport { pivots: Vec::new() }
}

#[allow(dead_code)]
pub fn pp_add(e: &mut PivotPointExport, name: &str, pos: [f32; 3]) {
    e.pivots.push(PivotPoint {
        name: name.to_string(),
        position: pos,
        orientation: [0.0, 0.0, 0.0, 1.0],
    });
}

#[allow(dead_code)]
pub fn pp_add_with_orientation(
    e: &mut PivotPointExport,
    name: &str,
    pos: [f32; 3],
    orient: [f32; 4],
) {
    e.pivots.push(PivotPoint {
        name: name.to_string(),
        position: pos,
        orientation: orient,
    });
}

#[allow(dead_code)]
pub fn pp_count(e: &PivotPointExport) -> usize {
    e.pivots.len()
}

#[allow(dead_code)]
pub fn pp_get(e: &PivotPointExport, idx: usize) -> Option<&PivotPoint> {
    e.pivots.get(idx)
}

#[allow(dead_code)]
pub fn pp_find_by_name(e: &PivotPointExport, name: &str) -> Option<usize> {
    e.pivots.iter().position(|p| p.name == name)
}

#[allow(dead_code)]
pub fn pp_set_position(e: &mut PivotPointExport, idx: usize, pos: [f32; 3]) {
    if let Some(p) = e.pivots.get_mut(idx) {
        p.position = pos;
    }
}

#[allow(dead_code)]
pub fn pp_distance(a: &PivotPoint, b: &PivotPoint) -> f32 {
    let dx = a.position[0] - b.position[0];
    let dy = a.position[1] - b.position[1];
    let dz = a.position[2] - b.position[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn pp_centroid(e: &PivotPointExport) -> [f32; 3] {
    if e.pivots.is_empty() {
        return [0.0; 3];
    }
    let n = e.pivots.len() as f32;
    let mut sum = [0.0_f32; 3];
    for p in &e.pivots {
        for (k, s) in sum.iter_mut().enumerate() {
            *s += p.position[k];
        }
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

#[allow(dead_code)]
pub fn pp_validate(e: &PivotPointExport) -> bool {
    e.pivots.iter().all(|p| {
        !p.name.is_empty()
            && p.position.iter().all(|v| v.is_finite())
            && p.orientation.iter().all(|v| v.is_finite())
    })
}

#[allow(dead_code)]
pub fn pivot_point_to_json(e: &PivotPointExport) -> String {
    format!("{{\"pivots\":{}}}", e.pivots.len())
}

#[allow(dead_code)]
pub fn pp_clear(e: &mut PivotPointExport) {
    e.pivots.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(pp_count(&new_pivot_point_export()), 0);
    }

    #[test]
    fn test_add() {
        let mut e = new_pivot_point_export();
        pp_add(&mut e, "root", [0.0, 0.0, 0.0]);
        assert_eq!(pp_count(&e), 1);
    }

    #[test]
    fn test_add_with_orientation() {
        let mut e = new_pivot_point_export();
        pp_add_with_orientation(&mut e, "hand", [1.0, 0.0, 0.0], [0.0, 0.707, 0.0, 0.707]);
        let p = pp_get(&e, 0).expect("should succeed");
        assert!((p.orientation[1] - 0.707).abs() < 1e-3);
    }

    #[test]
    fn test_get() {
        let mut e = new_pivot_point_export();
        pp_add(&mut e, "hip", [0.0, 1.0, 0.0]);
        let p = pp_get(&e, 0).expect("should succeed");
        assert_eq!(p.name, "hip");
    }

    #[test]
    fn test_get_oob() {
        assert!(pp_get(&new_pivot_point_export(), 0).is_none());
    }

    #[test]
    fn test_find_by_name() {
        let mut e = new_pivot_point_export();
        pp_add(&mut e, "a", [0.0; 3]);
        pp_add(&mut e, "b", [1.0; 3]);
        assert_eq!(pp_find_by_name(&e, "b"), Some(1));
        assert!(pp_find_by_name(&e, "c").is_none());
    }

    #[test]
    fn test_set_position() {
        let mut e = new_pivot_point_export();
        pp_add(&mut e, "root", [0.0; 3]);
        pp_set_position(&mut e, 0, [5.0, 6.0, 7.0]);
        assert!((pp_get(&e, 0).expect("should succeed").position[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_distance() {
        let a = PivotPoint {
            name: "a".into(),
            position: [0.0, 0.0, 0.0],
            orientation: [0.0; 4],
        };
        let b = PivotPoint {
            name: "b".into(),
            position: [3.0, 4.0, 0.0],
            orientation: [0.0; 4],
        };
        assert!((pp_distance(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_centroid() {
        let mut e = new_pivot_point_export();
        pp_add(&mut e, "a", [0.0, 0.0, 0.0]);
        pp_add(&mut e, "b", [2.0, 4.0, 6.0]);
        let c = pp_centroid(&e);
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_validate() {
        let mut e = new_pivot_point_export();
        pp_add(&mut e, "ok", [1.0, 2.0, 3.0]);
        assert!(pp_validate(&e));
    }

    #[test]
    fn test_to_json() {
        let e = new_pivot_point_export();
        assert!(pivot_point_to_json(&e).contains("\"pivots\":0"));
    }
}
