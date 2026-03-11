// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ParamMap — bidirectional parameter remapping.

#![allow(dead_code)]

/// A single remapping entry from a source range to a target range.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ParamRemapping {
    pub src_lo: f32,
    pub src_hi: f32,
    pub dst_lo: f32,
    pub dst_hi: f32,
    pub name: String,
}

/// A named collection of parameter remappings.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ParamMap {
    pub mappings: Vec<ParamRemapping>,
}

/// Create an empty `ParamMap`.
#[allow(dead_code)]
pub fn new_param_map() -> ParamMap {
    ParamMap { mappings: Vec::new() }
}

/// Append a remapping entry to the map.
#[allow(dead_code)]
pub fn add_mapping(map: &mut ParamMap, name: &str, src_lo: f32, src_hi: f32, dst_lo: f32, dst_hi: f32) {
    map.mappings.push(ParamRemapping {
        src_lo,
        src_hi,
        dst_lo,
        dst_hi,
        name: name.to_owned(),
    });
}

/// Remap a single value using the first mapping whose name matches.
#[allow(dead_code)]
pub fn remap_value(map: &ParamMap, name: &str, value: f32) -> f32 {
    for m in &map.mappings {
        if m.name == name && (m.src_lo..=m.src_hi).contains(&value) {
            let t = (value - m.src_lo) / (m.src_hi - m.src_lo).max(f32::EPSILON);
            return m.dst_lo + t * (m.dst_hi - m.dst_lo);
        }
    }
    value
}

/// Remap every value in `values` using the mapping at `index`.
#[allow(dead_code)]
pub fn remap_all(map: &ParamMap, index: usize, values: &[f32]) -> Vec<f32> {
    if index >= map.mappings.len() {
        return values.to_vec();
    }
    let m = &map.mappings[index];
    values
        .iter()
        .map(|&v| {
            let t = (v - m.src_lo) / (m.src_hi - m.src_lo).max(f32::EPSILON);
            m.dst_lo + t * (m.dst_hi - m.dst_lo)
        })
        .collect()
}

/// Return the number of mappings in the map.
#[allow(dead_code)]
pub fn param_map_count(map: &ParamMap) -> usize {
    map.mappings.len()
}

/// Return an inverted version of the mapping at `index`.
#[allow(dead_code)]
pub fn invert_mapping(map: &ParamMap, index: usize) -> Option<ParamRemapping> {
    map.mappings.get(index).map(|m| ParamRemapping {
        src_lo: m.dst_lo,
        src_hi: m.dst_hi,
        dst_lo: m.src_lo,
        dst_hi: m.src_hi,
        name: format!("{}_inv", m.name),
    })
}

/// Compose two mappings (first apply `a`, then `b`).
#[allow(dead_code)]
pub fn compose_mappings(a: &ParamRemapping, b: &ParamRemapping) -> ParamRemapping {
    let b_src_range = (b.src_hi - b.src_lo).max(f32::EPSILON);
    let b_dst_range = b.dst_hi - b.dst_lo;
    // Output at a.src_lo: b applied to a.dst_lo
    let out_lo = b.dst_lo + (a.dst_lo - b.src_lo) / b_src_range * b_dst_range;
    // Output at a.src_hi: b applied to a.dst_hi
    let out_hi = b.dst_lo + (a.dst_hi - b.src_lo) / b_src_range * b_dst_range;
    ParamRemapping {
        src_lo: a.src_lo,
        src_hi: a.src_hi,
        dst_lo: out_lo,
        dst_hi: out_hi,
        name: format!("{}_{}", a.name, b.name),
    }
}

/// Return the output range `[dst_lo, dst_hi]` for the mapping at `index`.
#[allow(dead_code)]
pub fn mapping_range(map: &ParamMap, index: usize) -> Option<(f32, f32)> {
    map.mappings.get(index).map(|m| (m.dst_lo, m.dst_hi))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_param_map_empty() {
        let m = new_param_map();
        assert_eq!(param_map_count(&m), 0);
    }

    #[test]
    fn test_add_mapping_increments_count() {
        let mut m = new_param_map();
        add_mapping(&mut m, "test", 0.0, 1.0, 0.0, 10.0);
        assert_eq!(param_map_count(&m), 1);
    }

    #[test]
    fn test_remap_value_midpoint() {
        let mut m = new_param_map();
        add_mapping(&mut m, "x", 0.0, 1.0, 0.0, 100.0);
        let v = remap_value(&m, "x", 0.5);
        assert!((v - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_remap_value_unknown_name_passthrough() {
        use std::f32::consts::PI;
        let m = new_param_map();
        assert!((remap_value(&m, "nope", PI) - PI).abs() < 1e-6);
    }

    #[test]
    fn test_remap_all_doubles_values() {
        let mut m = new_param_map();
        add_mapping(&mut m, "a", 0.0, 1.0, 0.0, 2.0);
        let out = remap_all(&m, 0, &[0.0, 0.5, 1.0]);
        assert!((out[0]).abs() < 1e-4);
        assert!((out[1] - 1.0).abs() < 1e-4);
        assert!((out[2] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_invert_mapping() {
        let mut m = new_param_map();
        add_mapping(&mut m, "fwd", 0.0, 1.0, 10.0, 20.0);
        let inv = invert_mapping(&m, 0).unwrap();
        assert!((inv.src_lo - 10.0).abs() < 1e-5);
        assert!((inv.dst_hi - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compose_mappings() {
        let a = ParamRemapping { src_lo: 0.0, src_hi: 1.0, dst_lo: 0.0, dst_hi: 1.0, name: "a".into() };
        let b = ParamRemapping { src_lo: 0.0, src_hi: 1.0, dst_lo: 0.0, dst_hi: 2.0, name: "b".into() };
        let c = compose_mappings(&a, &b);
        assert_eq!(c.name, "a_b");
        assert!((c.dst_hi - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_mapping_range_some() {
        let mut m = new_param_map();
        add_mapping(&mut m, "r", 0.0, 1.0, 5.0, 9.0);
        let (lo, hi) = mapping_range(&m, 0).unwrap();
        assert!((lo - 5.0).abs() < 1e-5);
        assert!((hi - 9.0).abs() < 1e-5);
    }

    #[test]
    fn test_mapping_range_none() {
        let m = new_param_map();
        assert!(mapping_range(&m, 99).is_none());
    }

    #[test]
    fn test_remap_all_out_of_bounds_index() {
        let m = new_param_map();
        let out = remap_all(&m, 5, &[1.0, 2.0]);
        assert_eq!(out, vec![1.0, 2.0]);
    }
}
