// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Houdini BGEO format stub export.

/// BGEO file version.
pub const BGEO_VERSION: u32 = 5;

/// BGEO attribute type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BgeoAttrType {
    Float,
    Int,
    String,
    Vector3,
}

/// A BGEO attribute definition.
#[derive(Debug, Clone)]
pub struct BgeoAttr {
    pub name: String,
    pub attr_type: BgeoAttrType,
    pub default_float: f32,
}

/// A BGEO export container.
#[derive(Debug, Clone)]
pub struct BgeoExport {
    pub version: u32,
    pub point_count: usize,
    pub prim_count: usize,
    pub positions: Vec<[f32; 3]>,
    pub attrs: Vec<BgeoAttr>,
}

/// Create a new empty BGEO export.
pub fn new_bgeo_export() -> BgeoExport {
    BgeoExport {
        version: BGEO_VERSION,
        point_count: 0,
        prim_count: 0,
        positions: Vec::new(),
        attrs: Vec::new(),
    }
}

/// Add a position to the export.
pub fn bgeo_add_point(export: &mut BgeoExport, pos: [f32; 3]) {
    export.positions.push(pos);
    export.point_count += 1;
}

/// Add an attribute definition.
pub fn bgeo_add_attr(export: &mut BgeoExport, name: &str, attr_type: BgeoAttrType, default: f32) {
    export.attrs.push(BgeoAttr {
        name: name.to_string(),
        attr_type,
        default_float: default,
    });
}

/// Set the primitive count.
pub fn bgeo_set_prim_count(export: &mut BgeoExport, count: usize) {
    export.prim_count = count;
}

/// Validate the export.
pub fn validate_bgeo(export: &BgeoExport) -> bool {
    export.point_count == export.positions.len()
}

/// Estimate the binary size of the BGEO file (stub).
pub fn bgeo_size_estimate(export: &BgeoExport) -> usize {
    /* 4 bytes version + 4 bytes point_count + 4 bytes prim_count + 12 per position */
    12 + export.positions.len() * 12
}

/// Generate a stub BGEO header as bytes.
pub fn bgeo_header_bytes(export: &BgeoExport) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"BGEO");
    out.extend_from_slice(&export.version.to_le_bytes());
    out.extend_from_slice(&(export.point_count as u32).to_le_bytes());
    out
}

/// Return the attribute count.
pub fn bgeo_attr_count(export: &BgeoExport) -> usize {
    export.attrs.len()
}

/// Find an attribute by name.
pub fn bgeo_find_attr<'a>(export: &'a BgeoExport, name: &str) -> Option<&'a BgeoAttr> {
    export.attrs.iter().find(|a| a.name == name)
}

/// Compute bounding box of points.
pub fn bgeo_bounds(export: &BgeoExport) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for &p in &export.positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bgeo() {
        let exp = new_bgeo_export();
        assert_eq!(exp.version, BGEO_VERSION);
        assert_eq!(exp.point_count, 0);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_bgeo_export();
        bgeo_add_point(&mut exp, [1.0, 2.0, 3.0]);
        assert_eq!(exp.point_count, 1);
    }

    #[test]
    fn test_validate_valid() {
        let mut exp = new_bgeo_export();
        bgeo_add_point(&mut exp, [0.0, 0.0, 0.0]);
        assert!(validate_bgeo(&exp));
    }

    #[test]
    fn test_bgeo_size_estimate() {
        let mut exp = new_bgeo_export();
        bgeo_add_point(&mut exp, [0.0, 0.0, 0.0]);
        assert!(bgeo_size_estimate(&exp) > 12);
    }

    #[test]
    fn test_bgeo_header_bytes() {
        let exp = new_bgeo_export();
        let hdr = bgeo_header_bytes(&exp);
        assert_eq!(&hdr[0..4], b"BGEO");
    }

    #[test]
    fn test_add_attr() {
        let mut exp = new_bgeo_export();
        bgeo_add_attr(&mut exp, "pscale", BgeoAttrType::Float, 0.1);
        assert_eq!(bgeo_attr_count(&exp), 1);
    }

    #[test]
    fn test_find_attr() {
        let mut exp = new_bgeo_export();
        bgeo_add_attr(&mut exp, "pscale", BgeoAttrType::Float, 0.1);
        let found = bgeo_find_attr(&exp, "pscale");
        assert!(found.is_some());
    }

    #[test]
    fn test_bgeo_bounds() {
        let mut exp = new_bgeo_export();
        bgeo_add_point(&mut exp, [-1.0, 0.0, 0.0]);
        bgeo_add_point(&mut exp, [1.0, 0.0, 0.0]);
        let (mn, mx) = bgeo_bounds(&exp);
        assert!(mn[0] < mx[0]);
    }

    #[test]
    fn test_set_prim_count() {
        let mut exp = new_bgeo_export();
        bgeo_set_prim_count(&mut exp, 42);
        assert_eq!(exp.prim_count, 42);
    }
}
