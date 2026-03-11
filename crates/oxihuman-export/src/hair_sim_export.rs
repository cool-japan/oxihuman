// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hair simulation export: guide strand dynamics and physics params.

/// A hair strand for simulation export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairSimStrand {
    pub root: [f32; 3],
    pub tip: [f32; 3],
    pub stiffness: f32,
    pub damping: f32,
    pub points: Vec<[f32; 3]>,
}

/// Hair simulation export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairSimExport {
    pub strands: Vec<HairSimStrand>,
    pub gravity: [f32; 3],
    pub wind: [f32; 3],
}

/// Create a new hair sim export.
#[allow(dead_code)]
pub fn new_hair_sim_export() -> HairSimExport {
    HairSimExport {
        strands: Vec::new(),
        gravity: [0.0, -9.81, 0.0],
        wind: [0.0; 3],
    }
}

/// Add a strand.
#[allow(dead_code)]
pub fn add_hair_strand(exp: &mut HairSimExport, strand: HairSimStrand) {
    exp.strands.push(strand);
}

/// Strand count.
#[allow(dead_code)]
pub fn hair_strand_count_sim(exp: &HairSimExport) -> usize {
    exp.strands.len()
}

/// Total point count across all strands.
#[allow(dead_code)]
pub fn total_sim_points(exp: &HairSimExport) -> usize {
    exp.strands.iter().map(|s| s.points.len()).sum()
}

/// Average stiffness.
#[allow(dead_code)]
pub fn avg_stiffness(exp: &HairSimExport) -> f32 {
    if exp.strands.is_empty() {
        return 0.0;
    }
    exp.strands.iter().map(|s| s.stiffness).sum::<f32>() / exp.strands.len() as f32
}

/// Average strand length (root to tip).
#[allow(dead_code)]
pub fn avg_strand_length_sim(exp: &HairSimExport) -> f32 {
    if exp.strands.is_empty() {
        return 0.0;
    }
    let sum: f32 = exp
        .strands
        .iter()
        .map(|s| {
            let d = [
                s.tip[0] - s.root[0],
                s.tip[1] - s.root[1],
                s.tip[2] - s.root[2],
            ];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .sum();
    sum / exp.strands.len() as f32
}

/// Validate: stiffness in `[0,1]`, damping in `[0,1]`.
#[allow(dead_code)]
pub fn validate_hair_sim(exp: &HairSimExport) -> bool {
    exp.strands
        .iter()
        .all(|s| (0.0..=1.0).contains(&s.stiffness) && (0.0..=1.0).contains(&s.damping))
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn hair_sim_to_json(exp: &HairSimExport) -> String {
    format!(
        "{{\"strand_count\":{},\"total_points\":{}}}",
        hair_strand_count_sim(exp),
        total_sim_points(exp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strand(stiff: f32) -> HairSimStrand {
        HairSimStrand {
            root: [0.0; 3],
            tip: [0.0, 0.1, 0.0],
            stiffness: stiff,
            damping: 0.05,
            points: vec![[0.0; 3]; 5],
        }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_hair_sim_export();
        assert_eq!(hair_strand_count_sim(&exp), 0);
    }

    #[test]
    fn add_strand_increments() {
        let mut exp = new_hair_sim_export();
        add_hair_strand(&mut exp, strand(0.8));
        assert_eq!(hair_strand_count_sim(&exp), 1);
    }

    #[test]
    fn total_points_correct() {
        let mut exp = new_hair_sim_export();
        add_hair_strand(&mut exp, strand(0.5));
        add_hair_strand(&mut exp, strand(0.5));
        assert_eq!(total_sim_points(&exp), 10);
    }

    #[test]
    fn avg_stiffness_correct() {
        let mut exp = new_hair_sim_export();
        add_hair_strand(&mut exp, strand(0.2));
        add_hair_strand(&mut exp, strand(0.8));
        assert!((avg_stiffness(&exp) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn avg_length_positive() {
        let mut exp = new_hair_sim_export();
        add_hair_strand(&mut exp, strand(0.5));
        assert!(avg_strand_length_sim(&exp) > 0.0);
    }

    #[test]
    fn validate_valid() {
        let mut exp = new_hair_sim_export();
        add_hair_strand(&mut exp, strand(0.5));
        assert!(validate_hair_sim(&exp));
    }

    #[test]
    fn default_gravity_negative_y() {
        let exp = new_hair_sim_export();
        assert!(exp.gravity[1] < 0.0);
    }

    #[test]
    fn json_contains_strand_count() {
        let exp = new_hair_sim_export();
        let j = hair_sim_to_json(&exp);
        assert!(j.contains("strand_count"));
    }

    #[test]
    fn stiffness_in_range() {
        let s = strand(0.7);
        assert!((0.0..=1.0).contains(&s.stiffness));
    }

    #[test]
    fn empty_avg_stiffness_zero() {
        let exp = new_hair_sim_export();
        assert!((avg_stiffness(&exp)).abs() < 1e-6);
    }
}
