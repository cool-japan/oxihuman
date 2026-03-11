// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-strand and per-point hair width export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairWidthStrand {
    pub strand_id: u32,
    pub widths: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairWidthExport {
    pub strands: Vec<HairWidthStrand>,
}

#[allow(dead_code)]
pub fn new_hair_width_export() -> HairWidthExport {
    HairWidthExport {
        strands: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_strand_widths(exp: &mut HairWidthExport, strand_id: u32, widths: Vec<f32>) {
    exp.strands.push(HairWidthStrand { strand_id, widths });
}

#[allow(dead_code)]
pub fn strand_count_hw(exp: &HairWidthExport) -> usize {
    exp.strands.len()
}

#[allow(dead_code)]
pub fn total_width_points(exp: &HairWidthExport) -> usize {
    exp.strands.iter().map(|s| s.widths.len()).sum()
}

#[allow(dead_code)]
pub fn avg_width(exp: &HairWidthExport) -> f32 {
    let total = total_width_points(exp);
    if total == 0 {
        return 0.0;
    }
    let sum: f32 = exp
        .strands
        .iter()
        .flat_map(|s| s.widths.iter().copied())
        .sum();
    sum / total as f32
}

#[allow(dead_code)]
pub fn max_width(exp: &HairWidthExport) -> f32 {
    exp.strands
        .iter()
        .flat_map(|s| s.widths.iter().copied())
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn min_width(exp: &HairWidthExport) -> f32 {
    if exp.strands.is_empty() {
        return 0.0;
    }
    exp.strands
        .iter()
        .flat_map(|s| s.widths.iter().copied())
        .fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn scale_widths(exp: &mut HairWidthExport, factor: f32) {
    for s in &mut exp.strands {
        for w in &mut s.widths {
            *w *= factor;
        }
    }
}

#[allow(dead_code)]
pub fn hair_width_to_json(exp: &HairWidthExport) -> String {
    format!(
        "{{\"strand_count\":{},\"avg_width\":{}}}",
        strand_count_hw(exp),
        avg_width(exp)
    )
}

#[allow(dead_code)]
pub fn widths_positive(exp: &HairWidthExport) -> bool {
    exp.strands
        .iter()
        .all(|s| s.widths.iter().all(|&w| w >= 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_hair_width_export();
        assert_eq!(strand_count_hw(&exp), 0);
    }

    #[test]
    fn test_add_strand() {
        let mut exp = new_hair_width_export();
        add_strand_widths(&mut exp, 0, vec![0.01, 0.008, 0.005]);
        assert_eq!(strand_count_hw(&exp), 1);
    }

    #[test]
    fn test_total_points() {
        let mut exp = new_hair_width_export();
        add_strand_widths(&mut exp, 0, vec![0.01; 3]);
        add_strand_widths(&mut exp, 1, vec![0.01; 2]);
        assert_eq!(total_width_points(&exp), 5);
    }

    #[test]
    fn test_avg_width() {
        let mut exp = new_hair_width_export();
        add_strand_widths(&mut exp, 0, vec![1.0, 3.0]);
        assert!((avg_width(&exp) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_width() {
        let mut exp = new_hair_width_export();
        add_strand_widths(&mut exp, 0, vec![0.5, 1.0, 0.2]);
        assert!((max_width(&exp) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_min_width() {
        let mut exp = new_hair_width_export();
        add_strand_widths(&mut exp, 0, vec![0.5, 1.0, 0.2]);
        assert!((min_width(&exp) - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_scale_widths() {
        let mut exp = new_hair_width_export();
        add_strand_widths(&mut exp, 0, vec![1.0, 2.0]);
        scale_widths(&mut exp, 0.5);
        assert!((exp.strands[0].widths[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_hair_width_export();
        let j = hair_width_to_json(&exp);
        assert!(j.contains("strand_count"));
    }

    #[test]
    fn test_widths_positive() {
        let mut exp = new_hair_width_export();
        add_strand_widths(&mut exp, 0, vec![0.01, 0.02]);
        assert!(widths_positive(&exp));
    }

    #[test]
    fn test_empty_avg_zero() {
        let exp = new_hair_width_export();
        assert!((avg_width(&exp)).abs() < 1e-6);
    }
}
