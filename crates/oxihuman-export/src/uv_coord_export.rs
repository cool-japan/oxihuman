// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! UV coordinate export utilities.

/// UV coordinate export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvCoordExport {
    pub uvs: Vec<[f32; 2]>,
    pub channel: u32,
}

#[allow(dead_code)]
pub fn new_uv_coord_export(channel: u32) -> UvCoordExport {
    UvCoordExport {
        uvs: Vec::new(),
        channel,
    }
}

#[allow(dead_code)]
pub fn uvc_add(e: &mut UvCoordExport, uv: [f32; 2]) {
    e.uvs.push(uv);
}

#[allow(dead_code)]
pub fn uvc_count(e: &UvCoordExport) -> usize {
    e.uvs.len()
}

#[allow(dead_code)]
pub fn uvc_get(e: &UvCoordExport, idx: usize) -> Option<[f32; 2]> {
    e.uvs.get(idx).copied()
}

#[allow(dead_code)]
pub fn uvc_bounds(e: &UvCoordExport) -> ([f32; 2], [f32; 2]) {
    if e.uvs.is_empty() {
        return ([0.0; 2], [0.0; 2]);
    }
    let mut mn = e.uvs[0];
    let mut mx = e.uvs[0];
    for uv in &e.uvs {
        mn[0] = mn[0].min(uv[0]);
        mn[1] = mn[1].min(uv[1]);
        mx[0] = mx[0].max(uv[0]);
        mx[1] = mx[1].max(uv[1]);
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn uvc_flip_v(e: &mut UvCoordExport) {
    for uv in &mut e.uvs {
        uv[1] = 1.0 - uv[1];
    }
}

#[allow(dead_code)]
pub fn uvc_normalize(e: &mut UvCoordExport) {
    let (mn, mx) = uvc_bounds(e);
    let range = [mx[0] - mn[0], mx[1] - mn[1]];
    for uv in &mut e.uvs {
        uv[0] = if range[0] > 1e-12 {
            (uv[0] - mn[0]) / range[0]
        } else {
            0.0
        };
        uv[1] = if range[1] > 1e-12 {
            (uv[1] - mn[1]) / range[1]
        } else {
            0.0
        };
    }
}

#[allow(dead_code)]
pub fn uvc_validate(e: &UvCoordExport) -> bool {
    e.uvs
        .iter()
        .all(|uv| (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]))
}

#[allow(dead_code)]
pub fn uv_coord_to_json(e: &UvCoordExport) -> String {
    format!("{{\"channel\":{},\"count\":{}}}", e.channel, e.uvs.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(uvc_count(&new_uv_coord_export(0)), 0);
    }

    #[test]
    fn test_add() {
        let mut e = new_uv_coord_export(0);
        uvc_add(&mut e, [0.5, 0.5]);
        assert_eq!(uvc_count(&e), 1);
    }

    #[test]
    fn test_get() {
        let mut e = new_uv_coord_export(0);
        uvc_add(&mut e, [0.1, 0.9]);
        let uv = uvc_get(&e, 0).unwrap();
        assert!((uv[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_get_oob() {
        assert!(uvc_get(&new_uv_coord_export(0), 0).is_none());
    }

    #[test]
    fn test_bounds() {
        let mut e = new_uv_coord_export(0);
        uvc_add(&mut e, [0.1, 0.2]);
        uvc_add(&mut e, [0.8, 0.9]);
        let (mn, mx) = uvc_bounds(&e);
        assert!((mn[0] - 0.1).abs() < 1e-6);
        assert!((mx[1] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_flip_v() {
        let mut e = new_uv_coord_export(0);
        uvc_add(&mut e, [0.5, 0.2]);
        uvc_flip_v(&mut e);
        assert!((e.uvs[0][1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_normalize() {
        let mut e = new_uv_coord_export(0);
        uvc_add(&mut e, [2.0, 4.0]);
        uvc_add(&mut e, [4.0, 8.0]);
        uvc_normalize(&mut e);
        assert!((e.uvs[0][0]).abs() < 1e-6);
        assert!((e.uvs[1][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_validate_ok() {
        let mut e = new_uv_coord_export(0);
        uvc_add(&mut e, [0.0, 1.0]);
        assert!(uvc_validate(&e));
    }

    #[test]
    fn test_validate_bad() {
        let mut e = new_uv_coord_export(0);
        uvc_add(&mut e, [-0.1, 0.5]);
        assert!(!uvc_validate(&e));
    }

    #[test]
    fn test_to_json() {
        assert!(uv_coord_to_json(&new_uv_coord_export(2)).contains("\"channel\":2"));
    }
}
