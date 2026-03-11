#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export texture atlas layout.

#[allow(dead_code)]
pub struct AtlasEntry {
    pub name: String,
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
    pub page: u32,
}

#[allow(dead_code)]
pub struct AtlasExport {
    pub entries: Vec<AtlasEntry>,
    pub width: u32,
    pub height: u32,
    pub pages: u32,
}

#[allow(dead_code)]
pub fn new_atlas_export(w: u32, h: u32) -> AtlasExport {
    AtlasExport {
        entries: Vec::new(),
        width: w,
        height: h,
        pages: 1,
    }
}

#[allow(dead_code)]
pub fn add_atlas_entry(
    exp: &mut AtlasExport,
    name: &str,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    page: u32,
) {
    if page >= exp.pages {
        exp.pages = page + 1;
    }
    exp.entries.push(AtlasEntry {
        name: name.to_string(),
        u_min: u0,
        v_min: v0,
        u_max: u1,
        v_max: v1,
        page,
    });
}

#[allow(dead_code)]
pub fn export_atlas_to_json(exp: &AtlasExport) -> String {
    let mut s = format!(
        "{{\"width\":{},\"height\":{},\"pages\":{},\"entries\":[",
        exp.width, exp.height, exp.pages
    );
    for (i, e) in exp.entries.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"u_min\":{},\"v_min\":{},\"u_max\":{},\"v_max\":{},\"page\":{}}}",
            e.name, e.u_min, e.v_min, e.u_max, e.v_max, e.page
        ));
    }
    s.push_str("]}");
    s
}

#[allow(dead_code)]
pub fn entry_count(exp: &AtlasExport) -> usize {
    exp.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AtlasExport {
        let mut exp = new_atlas_export(1024, 1024);
        add_atlas_entry(&mut exp, "skin", 0.0, 0.0, 0.5, 0.5, 0);
        add_atlas_entry(&mut exp, "eye", 0.5, 0.0, 1.0, 0.5, 0);
        exp
    }

    #[test]
    fn new_export_empty() {
        let exp = new_atlas_export(512, 512);
        assert_eq!(entry_count(&exp), 0);
    }

    #[test]
    fn add_entry_increases_count() {
        let mut exp = new_atlas_export(512, 512);
        add_atlas_entry(&mut exp, "a", 0.0, 0.0, 1.0, 1.0, 0);
        assert_eq!(entry_count(&exp), 1);
    }

    #[test]
    fn dimensions_stored() {
        let exp = new_atlas_export(2048, 1024);
        assert_eq!(exp.width, 2048);
        assert_eq!(exp.height, 1024);
    }

    #[test]
    fn entry_name_preserved() {
        let exp = sample();
        assert_eq!(exp.entries[0].name, "skin");
    }

    #[test]
    fn uv_bounds_stored() {
        let exp = sample();
        assert!((exp.entries[0].u_max - 0.5).abs() < 1e-6);
        assert!((exp.entries[1].u_min - 0.5).abs() < 1e-6);
    }

    #[test]
    fn page_count_auto_updated() {
        let mut exp = new_atlas_export(512, 512);
        add_atlas_entry(&mut exp, "a", 0.0, 0.0, 1.0, 1.0, 2);
        assert_eq!(exp.pages, 3);
    }

    #[test]
    fn json_contains_width() {
        let exp = sample();
        let json = export_atlas_to_json(&exp);
        assert!(json.contains("1024"));
    }

    #[test]
    fn json_contains_entry_name() {
        let exp = sample();
        let json = export_atlas_to_json(&exp);
        assert!(json.contains("skin"));
    }

    #[test]
    fn entry_count_two() {
        let exp = sample();
        assert_eq!(entry_count(&exp), 2);
    }

    #[test]
    fn json_valid_brackets() {
        let exp = sample();
        let json = export_atlas_to_json(&exp);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }
}
