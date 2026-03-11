// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Skin melanin density layer.
pub struct MelaninLayer {
    pub eumelanin: f32,
    pub pheomelanin: f32,
    pub melanocyte_density: f32,
}

impl MelaninLayer {
    pub fn new(melanocyte_density: f32) -> Self {
        MelaninLayer {
            eumelanin: melanocyte_density * 0.6,
            pheomelanin: melanocyte_density * 0.1,
            melanocyte_density,
        }
    }
}

pub fn new_melanin_layer(melanocyte_density: f32) -> MelaninLayer {
    MelaninLayer::new(melanocyte_density)
}

pub fn melanin_total(m: &MelaninLayer) -> f32 {
    m.eumelanin + m.pheomelanin
}

/// ITA-based skin tone index: 0 = lightest (low melanin), 1 = darkest.
/// Simplified: index ≈ total / (total + baseline), baseline=0.5
pub fn melanin_skin_tone_index(m: &MelaninLayer) -> f32 {
    let total = melanin_total(m);
    (total / (total + 0.5)).clamp(0.0, 1.0)
}

/// UV protection factor correlates with eumelanin content.
pub fn melanin_uv_protection_factor(m: &MelaninLayer) -> f32 {
    (1.0 + m.eumelanin * 10.0).clamp(1.0, 50.0)
}

pub fn melanin_set_eumelanin(m: &mut MelaninLayer, v: f32) {
    m.eumelanin = v;
}

pub fn melanin_set_pheomelanin(m: &mut MelaninLayer, v: f32) {
    m.pheomelanin = v;
}

/// eu / total ratio (0 if no melanin).
pub fn melanin_ratio(m: &MelaninLayer) -> f32 {
    let total = melanin_total(m);
    if total < 1e-9 {
        0.0
    } else {
        m.eumelanin / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new layer initialises based on density */
        let m = new_melanin_layer(1.0);
        assert!(m.eumelanin > 0.0);
        assert!(m.pheomelanin > 0.0);
    }

    #[test]
    fn test_total_positive() {
        /* total melanin is positive */
        let m = new_melanin_layer(1.0);
        assert!(melanin_total(&m) > 0.0);
    }

    #[test]
    fn test_skin_tone_index_range() {
        /* skin tone index in [0,1] */
        let m = new_melanin_layer(1.0);
        let idx = melanin_skin_tone_index(&m);
        assert!((0.0..=1.0).contains(&idx));
    }

    #[test]
    fn test_uv_protection_at_least_one() {
        /* UV protection factor >= 1 */
        let m = new_melanin_layer(0.5);
        assert!(melanin_uv_protection_factor(&m) >= 1.0);
    }

    #[test]
    fn test_set_eumelanin() {
        /* set_eumelanin updates the value */
        let mut m = new_melanin_layer(1.0);
        melanin_set_eumelanin(&mut m, 5.0);
        assert!((m.eumelanin - 5.0).abs() < 1e-7);
    }

    #[test]
    fn test_set_pheomelanin() {
        /* set_pheomelanin updates the value */
        let mut m = new_melanin_layer(1.0);
        melanin_set_pheomelanin(&mut m, 2.0);
        assert!((m.pheomelanin - 2.0).abs() < 1e-7);
    }

    #[test]
    fn test_ratio() {
        /* ratio is eumelanin fraction of total */
        let mut m = new_melanin_layer(0.0);
        melanin_set_eumelanin(&mut m, 3.0);
        melanin_set_pheomelanin(&mut m, 1.0);
        assert!((melanin_ratio(&m) - 0.75).abs() < 1e-6);
    }
}
