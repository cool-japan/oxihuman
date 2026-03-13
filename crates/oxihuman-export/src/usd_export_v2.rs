// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! USD export v2 with composition arc support.

#[allow(dead_code)]
pub struct UsdPrimV2 {
    pub path: String,
    pub kind: String,
    pub attributes: Vec<(String, String)>,
}

#[allow(dead_code)]
pub struct UsdExportV2 {
    pub prims: Vec<UsdPrimV2>,
    pub up_axis: char,
}

#[allow(dead_code)]
pub fn new_usd_export_v2(up_axis: char) -> UsdExportV2 {
    UsdExportV2 { prims: Vec::new(), up_axis }
}

#[allow(dead_code)]
pub fn usd2_add_prim(e: &mut UsdExportV2, path: &str, kind: &str) -> usize {
    let idx = e.prims.len();
    e.prims.push(UsdPrimV2 { path: path.to_string(), kind: kind.to_string(), attributes: Vec::new() });
    idx
}

#[allow(dead_code)]
pub fn usd2_set_attr(e: &mut UsdExportV2, prim_idx: usize, key: &str, value: &str) {
    if prim_idx < e.prims.len() {
        e.prims[prim_idx].attributes.push((key.to_string(), value.to_string()));
    }
}

#[allow(dead_code)]
pub fn usd2_prim_count(e: &UsdExportV2) -> usize {
    e.prims.len()
}

#[allow(dead_code)]
pub fn usd2_to_usda(e: &UsdExportV2) -> String {
    let mut out = format!("#usda 1.0\n(\n    upAxis = \"{}\"\n)\n\n", e.up_axis);
    for prim in &e.prims {
        out.push_str(&format!("def {} \"{}\"\n{{\n", prim.kind, prim.path));
        for (k, v) in &prim.attributes {
            out.push_str(&format!("    {} = {}\n", k, v));
        }
        out.push_str("}\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = new_usd_export_v2('Y');
        assert_eq!(e.up_axis, 'Y');
        assert_eq!(usd2_prim_count(&e), 0);
    }

    #[test]
    fn test_add_prim() {
        let mut e = new_usd_export_v2('Y');
        let idx = usd2_add_prim(&mut e, "/World/Mesh", "Mesh");
        assert_eq!(idx, 0);
        assert_eq!(usd2_prim_count(&e), 1);
    }

    #[test]
    fn test_set_attr() {
        let mut e = new_usd_export_v2('Y');
        usd2_add_prim(&mut e, "/World/Mesh", "Mesh");
        usd2_set_attr(&mut e, 0, "faceVertexCounts", "[]");
        assert_eq!(e.prims[0].attributes.len(), 1);
    }

    #[test]
    fn test_prim_count() {
        let mut e = new_usd_export_v2('Y');
        for i in 0..3 {
            usd2_add_prim(&mut e, &format!("/p{}", i), "Xform");
        }
        assert_eq!(usd2_prim_count(&e), 3);
    }

    #[test]
    fn test_to_usda_contains_magic() {
        let e = new_usd_export_v2('Y');
        let s = usd2_to_usda(&e);
        assert!(s.contains("#usda"));
    }

    #[test]
    fn test_to_usda_contains_prim() {
        let mut e = new_usd_export_v2('Y');
        usd2_add_prim(&mut e, "/World", "Xform");
        let s = usd2_to_usda(&e);
        assert!(s.contains("/World"));
    }

    #[test]
    fn test_to_usda_up_axis() {
        let e = new_usd_export_v2('Z');
        let s = usd2_to_usda(&e);
        assert!(s.contains("upAxis"));
    }

    #[test]
    fn test_attr_in_usda() {
        let mut e = new_usd_export_v2('Y');
        usd2_add_prim(&mut e, "/Mesh", "Mesh");
        usd2_set_attr(&mut e, 0, "extent", "[(0,0,0),(1,1,1)]");
        let s = usd2_to_usda(&e);
        assert!(s.contains("extent"));
    }
}
