// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct EyebrowHair {
    pub pos: [f32; 3],
    pub direction: [f32; 3],
    pub length_mm: f32,
    pub thickness_um: f32,
}

pub fn new_eyebrow_hair(pos: [f32; 3], dir: [f32; 3], length: f32) -> EyebrowHair {
    EyebrowHair {
        pos,
        direction: dir,
        length_mm: length,
        thickness_um: 60.0,
    }
}

pub fn eyebrow_hair_to_csv_line(h: &EyebrowHair) -> String {
    format!(
        "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.3},{:.2}",
        h.pos[0],
        h.pos[1],
        h.pos[2],
        h.direction[0],
        h.direction[1],
        h.direction[2],
        h.length_mm,
        h.thickness_um
    )
}

pub fn eyebrow_hairs_to_csv(hairs: &[EyebrowHair]) -> String {
    let mut out = String::from("px,py,pz,dx,dy,dz,length_mm,thickness_um\n");
    for h in hairs {
        out.push_str(&eyebrow_hair_to_csv_line(h));
        out.push('\n');
    }
    out
}

pub fn eyebrow_mean_length(hairs: &[EyebrowHair]) -> f32 {
    if hairs.is_empty() {
        return 0.0;
    }
    hairs.iter().map(|h| h.length_mm).sum::<f32>() / hairs.len() as f32
}

pub fn eyebrow_density(hairs: &[EyebrowHair], area_cm2: f32) -> f32 {
    if area_cm2 <= 0.0 {
        return 0.0;
    }
    hairs.len() as f32 / area_cm2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_eyebrow_hair() {
        /* construction */
        let h = new_eyebrow_hair([0.0; 3], [1.0, 0.0, 0.0], 5.0);
        assert!((h.length_mm - 5.0).abs() < 1e-6);
        assert!((h.thickness_um - 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_csv_line() {
        /* CSV line has data */
        let h = new_eyebrow_hair([1.0, 2.0, 3.0], [1.0, 0.0, 0.0], 5.0);
        let line = eyebrow_hair_to_csv_line(&h);
        assert!(line.contains("1.0000"));
    }

    #[test]
    fn test_hairs_to_csv_header() {
        /* CSV has header */
        let hairs: Vec<EyebrowHair> = vec![];
        let csv = eyebrow_hairs_to_csv(&hairs);
        assert!(csv.contains("length_mm"));
    }

    #[test]
    fn test_mean_length() {
        /* mean length */
        let hairs = vec![
            new_eyebrow_hair([0.0; 3], [1.0, 0.0, 0.0], 4.0),
            new_eyebrow_hair([0.0; 3], [1.0, 0.0, 0.0], 6.0),
        ];
        assert!((eyebrow_mean_length(&hairs) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_density() {
        /* 10 hairs over 2 cm2 = 5/cm2 */
        let hairs: Vec<EyebrowHair> = (0..10)
            .map(|_| new_eyebrow_hair([0.0; 3], [1.0, 0.0, 0.0], 5.0))
            .collect();
        assert!((eyebrow_density(&hairs, 2.0) - 5.0).abs() < 1e-6);
    }
}
