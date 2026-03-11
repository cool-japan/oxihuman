// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Skin-level hair follicle root data with biometric detail.
pub struct SkinHairRoot {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub diameter_um: f32,
    pub growth_dir: [f32; 3],
    pub density: f32,
}

pub fn new_skin_hair_root(pos: [f32; 3], normal: [f32; 3]) -> SkinHairRoot {
    SkinHairRoot {
        pos,
        normal,
        diameter_um: 70.0,
        growth_dir: normal,
        density: 1.0,
    }
}

pub fn skin_hair_root_to_csv_line(h: &SkinHairRoot) -> String {
    format!(
        "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.2},{:.4},{:.4},{:.4},{:.4}",
        h.pos[0],
        h.pos[1],
        h.pos[2],
        h.normal[0],
        h.normal[1],
        h.normal[2],
        h.diameter_um,
        h.growth_dir[0],
        h.growth_dir[1],
        h.growth_dir[2],
        h.density
    )
}

pub fn skin_hair_roots_to_csv(roots: &[SkinHairRoot]) -> String {
    let mut out = String::from("pos_x,pos_y,pos_z,nx,ny,nz,diameter_um,gx,gy,gz,density\n");
    for r in roots {
        out.push_str(&skin_hair_root_to_csv_line(r));
        out.push('\n');
    }
    out
}

pub fn skin_hair_root_density_per_cm2(roots: &[SkinHairRoot], area_cm2: f32) -> f32 {
    if area_cm2 <= 0.0 {
        return 0.0;
    }
    roots.len() as f32 / area_cm2
}

pub fn skin_hair_root_mean_diameter(roots: &[SkinHairRoot]) -> f32 {
    if roots.is_empty() {
        return 0.0;
    }
    roots.iter().map(|r| r.diameter_um).sum::<f32>() / roots.len() as f32
}

pub fn skin_hair_root_count(roots: &[SkinHairRoot]) -> usize {
    roots.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skin_hair_root() {
        /* construction */
        let r = new_skin_hair_root([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((r.diameter_um - 70.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_csv_line() {
        /* CSV line format */
        let r = new_skin_hair_root([1.0, 2.0, 3.0], [0.0, 1.0, 0.0]);
        let line = skin_hair_root_to_csv_line(&r);
        assert!(line.contains("1.0000"));
    }

    #[test]
    fn test_roots_to_csv_header() {
        /* CSV has header */
        let roots: Vec<SkinHairRoot> = vec![];
        let csv = skin_hair_roots_to_csv(&roots);
        assert!(csv.contains("pos_x"));
    }

    #[test]
    fn test_density_per_cm2() {
        /* 10 roots over 2 cm2 = 5 */
        let roots: Vec<SkinHairRoot> = (0..10)
            .map(|_| new_skin_hair_root([0.0; 3], [0.0, 1.0, 0.0]))
            .collect();
        assert!((skin_hair_root_density_per_cm2(&roots, 2.0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_mean_diameter() {
        /* default diameter is 70 */
        let roots = vec![new_skin_hair_root([0.0; 3], [0.0, 1.0, 0.0])];
        assert!((skin_hair_root_mean_diameter(&roots) - 70.0).abs() < 1e-6);
    }

    #[test]
    fn test_count() {
        /* count */
        let roots: Vec<SkinHairRoot> = (0..5)
            .map(|_| new_skin_hair_root([0.0; 3], [0.0, 1.0, 0.0]))
            .collect();
        assert_eq!(skin_hair_root_count(&roots), 5);
    }
}
