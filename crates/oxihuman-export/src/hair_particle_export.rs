// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export hair particle system data (strands, guides).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairStrand {
    pub points: Vec<[f32; 3]>,
    pub thickness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HairParticleExport {
    pub name: String,
    pub strands: Vec<HairStrand>,
}

#[allow(dead_code)]
pub fn new_hair_particle_export(name: &str) -> HairParticleExport {
    HairParticleExport { name: name.to_string(), strands: Vec::new() }
}

#[allow(dead_code)]
pub fn hpe_add_strand(hpe: &mut HairParticleExport, points: Vec<[f32; 3]>, thickness: f32) {
    hpe.strands.push(HairStrand { points, thickness: thickness.max(0.0) });
}

#[allow(dead_code)]
pub fn hpe_strand_count(hpe: &HairParticleExport) -> usize { hpe.strands.len() }

#[allow(dead_code)]
pub fn hpe_total_points(hpe: &HairParticleExport) -> usize {
    hpe.strands.iter().map(|s| s.points.len()).sum()
}

#[allow(dead_code)]
pub fn hpe_strand_length(strand: &HairStrand) -> f32 {
    if strand.points.len() < 2 { return 0.0; }
    strand.points.windows(2).map(|w| {
        let d = [w[1][0]-w[0][0], w[1][1]-w[0][1], w[1][2]-w[0][2]];
        (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt()
    }).sum()
}

#[allow(dead_code)]
pub fn hpe_avg_length(hpe: &HairParticleExport) -> f32 {
    if hpe.strands.is_empty() { return 0.0; }
    hpe.strands.iter().map(hpe_strand_length).sum::<f32>() / hpe.strands.len() as f32
}

#[allow(dead_code)]
pub fn hpe_validate(hpe: &HairParticleExport) -> bool {
    !hpe.strands.is_empty() && hpe.strands.iter().all(|s| s.points.len() >= 2 && s.thickness > 0.0)
}

#[allow(dead_code)]
pub fn hpe_to_json(hpe: &HairParticleExport) -> String {
    format!("{{\"name\":\"{}\",\"strands\":{},\"avg_length\":{:.4}}}", hpe.name, hpe.strands.len(), hpe_avg_length(hpe))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> HairParticleExport {
        let mut h = new_hair_particle_export("head_hair");
        hpe_add_strand(&mut h, vec![[0.0,0.0,0.0],[0.0,1.0,0.0],[0.0,2.0,0.0]], 0.01);
        hpe_add_strand(&mut h, vec![[1.0,0.0,0.0],[1.0,1.0,0.0]], 0.01);
        h
    }

    #[test] fn test_new() { let h = new_hair_particle_export("test"); assert!(h.strands.is_empty()); }
    #[test] fn test_add_strand() { assert_eq!(hpe_strand_count(&sample()), 2); }
    #[test] fn test_total_points() { assert_eq!(hpe_total_points(&sample()), 5); }
    #[test] fn test_strand_length() { let s = &sample().strands[0]; assert!((hpe_strand_length(s) - 2.0).abs() < 1e-5); }
    #[test] fn test_avg_length() { assert!(hpe_avg_length(&sample()) > 0.0); }
    #[test] fn test_validate() { assert!(hpe_validate(&sample())); }
    #[test] fn test_to_json() { assert!(hpe_to_json(&sample()).contains("head_hair")); }
    #[test] fn test_name() { assert_eq!(sample().name, "head_hair"); }
    #[test] fn test_empty_avg() { let h = new_hair_particle_export("e"); assert!((hpe_avg_length(&h)).abs() < 1e-6); }
    #[test] fn test_single_point() {
        let s = HairStrand { points: vec![[0.0,0.0,0.0]], thickness: 0.01 };
        assert!((hpe_strand_length(&s)).abs() < 1e-6);
    }
}
