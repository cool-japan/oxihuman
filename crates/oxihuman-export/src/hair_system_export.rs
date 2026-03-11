// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairStrand {
    pub root: [f32; 3],
    pub segments: Vec<[f32; 3]>,
    pub width: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairSystemExport {
    pub name: String,
    pub strands: Vec<HairStrand>,
    pub strand_count: usize,
}

#[allow(dead_code)]
pub fn new_hair_system_export(name: &str) -> HairSystemExport {
    HairSystemExport {
        name: name.to_string(),
        strands: Vec::new(),
        strand_count: 0,
    }
}

#[allow(dead_code)]
pub fn hs_add_strand(exp: &mut HairSystemExport, strand: HairStrand) {
    exp.strand_count += 1;
    exp.strands.push(strand);
}

#[allow(dead_code)]
pub fn hs_strand_count(exp: &HairSystemExport) -> usize {
    exp.strands.len()
}

#[allow(dead_code)]
pub fn hs_total_segments(exp: &HairSystemExport) -> usize {
    exp.strands.iter().map(|s| s.segments.len()).sum()
}

#[allow(dead_code)]
pub fn hs_avg_length(exp: &HairSystemExport) -> f32 {
    if exp.strands.is_empty() {
        return 0.0;
    }
    let total: f32 = exp.strands.iter().map(|s| {
        let mut len = 0.0f32;
        let pts: Vec<[f32; 3]> = std::iter::once(s.root).chain(s.segments.iter().copied()).collect();
        for i in 0..pts.len().saturating_sub(1) {
            let dx = pts[i + 1][0] - pts[i][0];
            let dy = pts[i + 1][1] - pts[i][1];
            let dz = pts[i + 1][2] - pts[i][2];
            len += (dx * dx + dy * dy + dz * dz).sqrt();
        }
        len
    }).sum();
    total / exp.strands.len() as f32
}

#[allow(dead_code)]
pub fn hs_to_json(exp: &HairSystemExport) -> String {
    format!(
        r#"{{"name":"{}","strand_count":{},"total_segments":{}}}"#,
        exp.name,
        exp.strands.len(),
        hs_total_segments(exp)
    )
}

#[allow(dead_code)]
pub fn hs_validate(exp: &HairSystemExport) -> bool {
    !exp.name.is_empty()
}

#[allow(dead_code)]
pub fn hs_get_strand(exp: &HairSystemExport, idx: usize) -> Option<&HairStrand> {
    exp.strands.get(idx)
}

#[allow(dead_code)]
pub fn hs_clear(exp: &mut HairSystemExport) {
    exp.strands.clear();
    exp.strand_count = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_strand(root: [f32; 3], tip: [f32; 3]) -> HairStrand {
        HairStrand {
            root,
            segments: vec![tip],
            width: 0.01,
        }
    }

    #[test]
    fn test_new_export() {
        let e = new_hair_system_export("Hair");
        assert_eq!(e.name, "Hair");
        assert_eq!(hs_strand_count(&e), 0);
    }

    #[test]
    fn test_add_strand() {
        let mut e = new_hair_system_export("Hair");
        hs_add_strand(&mut e, make_strand([0.0; 3], [0.0, 1.0, 0.0]));
        assert_eq!(hs_strand_count(&e), 1);
    }

    #[test]
    fn test_total_segments() {
        let mut e = new_hair_system_export("Hair");
        hs_add_strand(&mut e, make_strand([0.0; 3], [0.0, 1.0, 0.0]));
        hs_add_strand(&mut e, make_strand([1.0, 0.0, 0.0], [1.0, 1.0, 0.0]));
        assert_eq!(hs_total_segments(&e), 2);
    }

    #[test]
    fn test_avg_length_empty() {
        let e = new_hair_system_export("Hair");
        assert!((hs_avg_length(&e)).abs() < 1e-6);
    }

    #[test]
    fn test_avg_length() {
        let mut e = new_hair_system_export("Hair");
        hs_add_strand(&mut e, make_strand([0.0; 3], [0.0, 2.0, 0.0]));
        let l = hs_avg_length(&e);
        assert!((l - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_get_strand() {
        let mut e = new_hair_system_export("Hair");
        hs_add_strand(&mut e, make_strand([0.0; 3], [0.0, 1.0, 0.0]));
        assert!(hs_get_strand(&e, 0).is_some());
        assert!(hs_get_strand(&e, 99).is_none());
    }

    #[test]
    fn test_clear() {
        let mut e = new_hair_system_export("Hair");
        hs_add_strand(&mut e, make_strand([0.0; 3], [0.0, 1.0, 0.0]));
        hs_clear(&mut e);
        assert_eq!(hs_strand_count(&e), 0);
    }

    #[test]
    fn test_to_json() {
        let mut e = new_hair_system_export("Hair");
        hs_add_strand(&mut e, make_strand([0.0; 3], [0.0, 1.0, 0.0]));
        let j = hs_to_json(&e);
        assert!(j.contains("strand_count"));
        assert!(j.contains("Hair"));
    }
}
