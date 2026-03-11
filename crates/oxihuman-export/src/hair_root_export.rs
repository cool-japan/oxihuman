// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Export hair root point data for groom systems.
#[allow(dead_code)]
pub struct HairRoot {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub length: f32,
    pub group_id: u32,
}

#[allow(dead_code)]
pub struct HairRootExport {
    pub roots: Vec<HairRoot>,
    pub group_names: Vec<String>,
}

#[allow(dead_code)]
pub fn new_hair_root_export() -> HairRootExport {
    HairRootExport {
        roots: vec![],
        group_names: vec![],
    }
}

#[allow(dead_code)]
pub fn add_hair_root(export: &mut HairRootExport, root: HairRoot) {
    export.roots.push(root);
}

#[allow(dead_code)]
pub fn add_group(export: &mut HairRootExport, name: &str) {
    export.group_names.push(name.to_string());
}

#[allow(dead_code)]
pub fn root_count(export: &HairRootExport) -> usize {
    export.roots.len()
}

#[allow(dead_code)]
pub fn group_count(export: &HairRootExport) -> usize {
    export.group_names.len()
}

#[allow(dead_code)]
pub fn roots_in_group(export: &HairRootExport, group_id: u32) -> Vec<&HairRoot> {
    export
        .roots
        .iter()
        .filter(|r| r.group_id == group_id)
        .collect()
}

#[allow(dead_code)]
pub fn avg_hair_length(export: &HairRootExport) -> f32 {
    if export.roots.is_empty() {
        return 0.0;
    }
    export.roots.iter().map(|r| r.length).sum::<f32>() / export.roots.len() as f32
}

#[allow(dead_code)]
pub fn max_hair_length(export: &HairRootExport) -> f32 {
    export.roots.iter().map(|r| r.length).fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn min_hair_length(export: &HairRootExport) -> f32 {
    if export.roots.is_empty() {
        return 0.0;
    }
    export
        .roots
        .iter()
        .map(|r| r.length)
        .fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn hair_root_bounds(export: &HairRootExport) -> ([f32; 3], [f32; 3]) {
    if export.roots.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = export.roots[0].position;
    let mut mx = export.roots[0].position;
    for r in &export.roots {
        for k in 0..3 {
            if r.position[k] < mn[k] {
                mn[k] = r.position[k];
            }
            if r.position[k] > mx[k] {
                mx[k] = r.position[k];
            }
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn validate_hair_roots(export: &HairRootExport) -> bool {
    export
        .roots
        .iter()
        .all(|r| r.length >= 0.0 && r.position.iter().all(|&v| v.is_finite()))
}

#[allow(dead_code)]
pub fn hair_root_to_json(export: &HairRootExport) -> String {
    format!(
        "{{\"root_count\":{},\"group_count\":{},\"avg_length\":{}}}",
        export.roots.len(),
        export.group_names.len(),
        avg_hair_length(export)
    )
}

#[allow(dead_code)]
pub fn hair_root_to_csv(export: &HairRootExport) -> String {
    let mut s = "pos_x,pos_y,pos_z,length,group_id\n".to_string();
    for r in &export.roots {
        s.push_str(&format!(
            "{},{},{},{},{}\n",
            r.position[0], r.position[1], r.position[2], r.length, r.group_id
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_roots() -> HairRootExport {
        let mut e = new_hair_root_export();
        add_group(&mut e, "scalp");
        add_group(&mut e, "eyebrows");
        add_hair_root(
            &mut e,
            HairRoot {
                position: [0.0, 1.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                uv: [0.5, 0.5],
                length: 0.15,
                group_id: 0,
            },
        );
        add_hair_root(
            &mut e,
            HairRoot {
                position: [0.1, 1.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                uv: [0.6, 0.5],
                length: 0.20,
                group_id: 0,
            },
        );
        add_hair_root(
            &mut e,
            HairRoot {
                position: [0.0, 1.0, 0.1],
                normal: [0.0, 1.0, 0.0],
                uv: [0.5, 0.4],
                length: 0.03,
                group_id: 1,
            },
        );
        e
    }

    #[test]
    fn test_root_count() {
        let e = sample_roots();
        assert_eq!(root_count(&e), 3);
    }

    #[test]
    fn test_group_count() {
        let e = sample_roots();
        assert_eq!(group_count(&e), 2);
    }

    #[test]
    fn test_roots_in_group() {
        let e = sample_roots();
        assert_eq!(roots_in_group(&e, 0).len(), 2);
    }

    #[test]
    fn test_avg_length() {
        let e = sample_roots();
        let avg = avg_hair_length(&e);
        assert!((avg - (0.15 + 0.20 + 0.03) / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_max_length() {
        let e = sample_roots();
        assert!((max_hair_length(&e) - 0.20).abs() < 1e-5);
    }

    #[test]
    fn test_min_length() {
        let e = sample_roots();
        assert!((min_hair_length(&e) - 0.03).abs() < 1e-5);
    }

    #[test]
    fn test_validate() {
        let e = sample_roots();
        assert!(validate_hair_roots(&e));
    }

    #[test]
    fn test_to_json() {
        let e = sample_roots();
        let j = hair_root_to_json(&e);
        assert!(j.contains("root_count"));
    }

    #[test]
    fn test_to_csv_header() {
        let e = sample_roots();
        let csv = hair_root_to_csv(&e);
        assert!(csv.starts_with("pos_x"));
    }

    #[test]
    fn test_bounds() {
        let e = sample_roots();
        let (mn, mx) = hair_root_bounds(&e);
        assert!(mx[0] >= mn[0]);
    }
}
