// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Simple splat (billboard) export for point visualization.

#[allow(dead_code)]
pub struct Splat {
    pub pos: [f32; 3],
    pub radius: f32,
    pub color: [u8; 4],
}

#[allow(dead_code)]
pub struct SplatExport {
    pub splats: Vec<Splat>,
}

#[allow(dead_code)]
pub fn new_splat_export() -> SplatExport {
    SplatExport { splats: Vec::new() }
}

#[allow(dead_code)]
pub fn splat_add(exp: &mut SplatExport, pos: [f32; 3], radius: f32, color: [u8; 4]) {
    exp.splats.push(Splat { pos, radius, color });
}

#[allow(dead_code)]
pub fn splat_count(exp: &SplatExport) -> usize {
    exp.splats.len()
}

#[allow(dead_code)]
pub fn splat_avg_radius(exp: &SplatExport) -> f32 {
    let n = exp.splats.len();
    if n == 0 { return 0.0; }
    exp.splats.iter().map(|s| s.radius).sum::<f32>() / n as f32
}

#[allow(dead_code)]
pub fn splat_to_csv(exp: &SplatExport) -> String {
    let mut out = String::from("x,y,z,radius,r,g,b,a\n");
    for s in &exp.splats {
        out.push_str(&format!("{},{},{},{},{},{},{},{}\n",
            s.pos[0], s.pos[1], s.pos[2], s.radius,
            s.color[0], s.color[1], s.color[2], s.color[3]));
    }
    out
}

#[allow(dead_code)]
pub fn splat_max_radius(exp: &SplatExport) -> f32 {
    exp.splats.iter().map(|s| s.radius).fold(0.0f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        assert_eq!(splat_count(&new_splat_export()), 0);
    }

    #[test]
    fn test_add() {
        let mut exp = new_splat_export();
        splat_add(&mut exp, [0.0; 3], 1.0, [255; 4]);
        assert_eq!(splat_count(&exp), 1);
    }

    #[test]
    fn test_avg_radius() {
        let mut exp = new_splat_export();
        splat_add(&mut exp, [0.0; 3], 2.0, [0; 4]);
        splat_add(&mut exp, [0.0; 3], 4.0, [0; 4]);
        assert!((splat_avg_radius(&exp) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_avg_radius_empty() {
        assert_eq!(splat_avg_radius(&new_splat_export()), 0.0);
    }

    #[test]
    fn test_to_csv_header() {
        let exp = new_splat_export();
        assert!(splat_to_csv(&exp).contains("x,y,z"));
    }

    #[test]
    fn test_max_radius() {
        let mut exp = new_splat_export();
        splat_add(&mut exp, [0.0; 3], 1.0, [0; 4]);
        splat_add(&mut exp, [0.0; 3], 5.0, [0; 4]);
        splat_add(&mut exp, [0.0; 3], 2.0, [0; 4]);
        assert!((splat_max_radius(&exp) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_csv_contains_data() {
        let mut exp = new_splat_export();
        splat_add(&mut exp, [1.0, 2.0, 3.0], 0.5, [10, 20, 30, 255]);
        let csv = splat_to_csv(&exp);
        assert!(csv.contains("1"));
        assert!(csv.contains("0.5"));
    }

    #[test]
    fn test_max_radius_empty() {
        assert_eq!(splat_max_radius(&new_splat_export()), 0.0);
    }

    #[test]
    fn test_count_multiple() {
        let mut exp = new_splat_export();
        for _ in 0..6 {
            splat_add(&mut exp, [0.0; 3], 1.0, [0; 4]);
        }
        assert_eq!(splat_count(&exp), 6);
    }
}
