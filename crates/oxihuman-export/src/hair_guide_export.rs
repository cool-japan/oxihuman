// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hair guide curve export for groom data.

/// A single hair guide curve.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairGuideExport {
    pub points: Vec<[f32; 3]>,
    pub width: f32,
}

/// Collection of hair guides.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairGuideBundle {
    pub guides: Vec<HairGuideExport>,
}

/// Create new bundle.
#[allow(dead_code)]
pub fn new_hair_guide_bundle() -> HairGuideBundle {
    HairGuideBundle { guides: vec![] }
}

/// Add a guide.
#[allow(dead_code)]
pub fn add_guide(b: &mut HairGuideBundle, points: &[[f32; 3]], width: f32) {
    b.guides.push(HairGuideExport {
        points: points.to_vec(),
        width,
    });
}

/// Guide count.
#[allow(dead_code)]
pub fn hg_count(b: &HairGuideBundle) -> usize {
    b.guides.len()
}

/// Total point count.
#[allow(dead_code)]
pub fn hg_total_points(b: &HairGuideBundle) -> usize {
    b.guides.iter().map(|g| g.points.len()).sum()
}

/// Guide length (sum of segment lengths).
#[allow(dead_code)]
pub fn guide_length(g: &HairGuideExport) -> f32 {
    let mut len = 0.0f32;
    for i in 1..g.points.len() {
        let dx = g.points[i][0] - g.points[i - 1][0];
        let dy = g.points[i][1] - g.points[i - 1][1];
        let dz = g.points[i][2] - g.points[i - 1][2];
        len += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    len
}

/// Average guide length.
#[allow(dead_code)]
pub fn avg_guide_length(b: &HairGuideBundle) -> f32 {
    if b.guides.is_empty() {
        return 0.0;
    }
    b.guides.iter().map(guide_length).sum::<f32>() / b.guides.len() as f32
}

/// Validate.
#[allow(dead_code)]
pub fn hg_validate(b: &HairGuideBundle) -> bool {
    b.guides
        .iter()
        .all(|g| g.points.len() >= 2 && g.width > 0.0)
}

/// Export to JSON.
#[allow(dead_code)]
pub fn hair_guide_to_json(b: &HairGuideBundle) -> String {
    format!(
        "{{\"guides\":{},\"total_points\":{},\"avg_length\":{:.6}}}",
        hg_count(b),
        hg_total_points(b),
        avg_guide_length(b)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_guide() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]]
    }
    #[test]
    fn test_new() {
        let b = new_hair_guide_bundle();
        assert_eq!(hg_count(&b), 0);
    }
    #[test]
    fn test_add() {
        let mut b = new_hair_guide_bundle();
        add_guide(&mut b, &sample_guide(), 0.01);
        assert_eq!(hg_count(&b), 1);
    }
    #[test]
    fn test_total_points() {
        let mut b = new_hair_guide_bundle();
        add_guide(&mut b, &sample_guide(), 0.01);
        assert_eq!(hg_total_points(&b), 3);
    }
    #[test]
    fn test_guide_length() {
        let g = HairGuideExport {
            points: sample_guide(),
            width: 0.01,
        };
        assert!((guide_length(&g) - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_avg_length() {
        let mut b = new_hair_guide_bundle();
        add_guide(&mut b, &sample_guide(), 0.01);
        assert!((avg_guide_length(&b) - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_avg_empty() {
        let b = new_hair_guide_bundle();
        assert!((avg_guide_length(&b)).abs() < 1e-9);
    }
    #[test]
    fn test_validate() {
        let mut b = new_hair_guide_bundle();
        add_guide(&mut b, &sample_guide(), 0.01);
        assert!(hg_validate(&b));
    }
    #[test]
    fn test_validate_bad() {
        let mut b = new_hair_guide_bundle();
        add_guide(&mut b, &[[0.0; 3]], 0.01);
        assert!(!hg_validate(&b));
    }
    #[test]
    fn test_to_json() {
        let b = new_hair_guide_bundle();
        assert!(hair_guide_to_json(&b).contains("\"guides\":0"));
    }
    #[test]
    fn test_width() {
        let mut b = new_hair_guide_bundle();
        add_guide(&mut b, &sample_guide(), 0.05);
        assert!((b.guides[0].width - 0.05).abs() < 1e-6);
    }
}
