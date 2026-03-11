// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A hair clump entry.
#[allow(dead_code)]
pub struct HairClump {
    pub id: usize,
    pub root: [f32; 3],
    pub strand_count: usize,
    pub clump_factor: f32,
}

/// A hair clump export bundle.
#[allow(dead_code)]
#[derive(Default)]
pub struct HairClumpExport {
    pub clumps: Vec<HairClump>,
}

/// Create a new hair clump export.
#[allow(dead_code)]
pub fn new_hair_clump_export() -> HairClumpExport {
    HairClumpExport::default()
}

/// Add a clump.
#[allow(dead_code)]
pub fn add_hair_clump(
    export: &mut HairClumpExport,
    root: [f32; 3],
    strand_count: usize,
    clump_factor: f32,
) {
    let id = export.clumps.len();
    export.clumps.push(HairClump {
        id,
        root,
        strand_count,
        clump_factor,
    });
}

/// Count clumps.
#[allow(dead_code)]
pub fn hair_clump_count(export: &HairClumpExport) -> usize {
    export.clumps.len()
}

/// Total strand count across all clumps.
#[allow(dead_code)]
pub fn total_strand_count_hc(export: &HairClumpExport) -> usize {
    export.clumps.iter().map(|c| c.strand_count).sum()
}

/// Average clump factor.
#[allow(dead_code)]
pub fn avg_clump_factor(export: &HairClumpExport) -> f32 {
    if export.clumps.is_empty() {
        return 0.0;
    }
    export.clumps.iter().map(|c| c.clump_factor).sum::<f32>() / export.clumps.len() as f32
}

/// Find the largest clump by strand count.
#[allow(dead_code)]
pub fn largest_clump(export: &HairClumpExport) -> Option<&HairClump> {
    export.clumps.iter().max_by_key(|c| c.strand_count)
}

/// Validate clump factors are in [0, 1].
#[allow(dead_code)]
pub fn validate_clump_factors(export: &HairClumpExport) -> bool {
    export
        .clumps
        .iter()
        .all(|c| (0.0..=1.0).contains(&c.clump_factor))
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn hair_clump_to_json(export: &HairClumpExport) -> String {
    format!(
        r#"{{"clumps":{},"total_strands":{}}}"#,
        export.clumps.len(),
        total_strand_count_hc(export)
    )
}

/// Compute bounding box of clump roots.
#[allow(dead_code)]
pub fn clump_roots_bounds(export: &HairClumpExport) -> ([f32; 3], [f32; 3]) {
    if export.clumps.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = export.clumps[0].root;
    let mut mx = export.clumps[0].root;
    for c in &export.clumps {
        for i in 0..3 {
            mn[i] = mn[i].min(c.root[i]);
            mx[i] = mx[i].max(c.root[i]);
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_count() {
        let mut e = new_hair_clump_export();
        add_hair_clump(&mut e, [0.0; 3], 10, 0.5);
        assert_eq!(hair_clump_count(&e), 1);
    }

    #[test]
    fn total_strands() {
        let mut e = new_hair_clump_export();
        add_hair_clump(&mut e, [0.0; 3], 10, 0.5);
        add_hair_clump(&mut e, [1.0, 0.0, 0.0], 5, 0.5);
        assert_eq!(total_strand_count_hc(&e), 15);
    }

    #[test]
    fn avg_factor() {
        let mut e = new_hair_clump_export();
        add_hair_clump(&mut e, [0.0; 3], 5, 0.4);
        add_hair_clump(&mut e, [0.0; 3], 5, 0.6);
        assert!((avg_clump_factor(&e) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn largest_clump_found() {
        let mut e = new_hair_clump_export();
        add_hair_clump(&mut e, [0.0; 3], 5, 0.5);
        add_hair_clump(&mut e, [0.0; 3], 20, 0.5);
        assert_eq!(largest_clump(&e).unwrap().strand_count, 20);
    }

    #[test]
    fn validate_factors_valid() {
        let mut e = new_hair_clump_export();
        add_hair_clump(&mut e, [0.0; 3], 5, 0.5);
        assert!(validate_clump_factors(&e));
    }

    #[test]
    fn validate_factor_out_of_range() {
        let mut e = new_hair_clump_export();
        add_hair_clump(&mut e, [0.0; 3], 5, 1.5);
        assert!(!validate_clump_factors(&e));
    }

    #[test]
    fn json_has_clumps() {
        let e = new_hair_clump_export();
        let j = hair_clump_to_json(&e);
        assert!(j.contains("\"clumps\":0"));
    }

    #[test]
    fn bounds_single() {
        let mut e = new_hair_clump_export();
        add_hair_clump(&mut e, [1.0, 2.0, 3.0], 1, 0.5);
        let (mn, mx) = clump_roots_bounds(&e);
        assert_eq!(mn, mx);
    }

    #[test]
    fn empty_avg_factor() {
        let e = new_hair_clump_export();
        assert!((avg_clump_factor(&e) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn largest_none_empty() {
        let e = new_hair_clump_export();
        assert!(largest_clump(&e).is_none());
    }
}
