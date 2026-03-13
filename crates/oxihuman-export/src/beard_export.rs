// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct BeardStrand {
    pub root: [f32; 3],
    pub length_mm: f32,
    pub diameter_um: f32,
    pub color_rgb: [f32; 3],
}

pub fn new_beard_strand(root: [f32; 3], length: f32) -> BeardStrand {
    BeardStrand {
        root,
        length_mm: length,
        diameter_um: 80.0,
        color_rgb: [0.15, 0.10, 0.08],
    }
}

pub fn beard_strand_to_csv_line(s: &BeardStrand) -> String {
    format!(
        "{:.4},{:.4},{:.4},{:.3},{:.2},{:.4},{:.4},{:.4}",
        s.root[0],
        s.root[1],
        s.root[2],
        s.length_mm,
        s.diameter_um,
        s.color_rgb[0],
        s.color_rgb[1],
        s.color_rgb[2]
    )
}

pub fn beard_strands_to_csv(strands: &[BeardStrand]) -> String {
    let mut out = String::from("rx,ry,rz,length_mm,diameter_um,cr,cg,cb\n");
    for s in strands {
        out.push_str(&beard_strand_to_csv_line(s));
        out.push('\n');
    }
    out
}

pub fn beard_mean_length(strands: &[BeardStrand]) -> f32 {
    if strands.is_empty() {
        return 0.0;
    }
    strands.iter().map(|s| s.length_mm).sum::<f32>() / strands.len() as f32
}

pub fn beard_coverage_density(strands: &[BeardStrand], area_cm2: f32) -> f32 {
    if area_cm2 <= 0.0 {
        return 0.0;
    }
    strands.len() as f32 / area_cm2
}

pub fn beard_count(strands: &[BeardStrand]) -> usize {
    strands.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_beard_strand() {
        /* construction */
        let s = new_beard_strand([0.0; 3], 10.0);
        assert!((s.length_mm - 10.0).abs() < 1e-6);
        assert!((s.diameter_um - 80.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_csv_line() {
        /* CSV line */
        let s = new_beard_strand([1.0, 2.0, 3.0], 10.0);
        let line = beard_strand_to_csv_line(&s);
        assert!(line.contains("1.0000"));
    }

    #[test]
    fn test_strands_to_csv_header() {
        /* CSV has header */
        let csv = beard_strands_to_csv(&[]);
        assert!(csv.contains("length_mm"));
    }

    #[test]
    fn test_mean_length() {
        /* mean */
        let strands = vec![
            new_beard_strand([0.0; 3], 8.0),
            new_beard_strand([0.0; 3], 12.0),
        ];
        assert!((beard_mean_length(&strands) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_density() {
        /* 10 over 5 cm2 = 2 */
        let strands: Vec<BeardStrand> = (0..10).map(|_| new_beard_strand([0.0; 3], 5.0)).collect();
        assert!((beard_coverage_density(&strands, 5.0) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_count() {
        /* count */
        let strands: Vec<BeardStrand> = (0..7).map(|_| new_beard_strand([0.0; 3], 5.0)).collect();
        assert_eq!(beard_count(&strands), 7);
    }
}
