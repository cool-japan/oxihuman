// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct SpleenModel {
    pub blood_flow_ml_per_min: f32,
    pub red_pulp_volume_ml: f32,
    pub white_pulp_volume_ml: f32,
    pub filtration_efficiency: f32,
}

pub fn new_spleen_model() -> SpleenModel {
    SpleenModel {
        blood_flow_ml_per_min: 200.0,
        red_pulp_volume_ml: 120.0,
        white_pulp_volume_ml: 30.0,
        filtration_efficiency: 0.9,
    }
}

pub fn spleen_cells_filtered_per_min(s: &SpleenModel) -> f32 {
    s.blood_flow_ml_per_min * s.filtration_efficiency
}

pub fn spleen_is_splenomegaly(s: &SpleenModel) -> bool {
    spleen_total_volume(s) > 250.0
}

pub fn spleen_total_volume(s: &SpleenModel) -> f32 {
    s.red_pulp_volume_ml + s.white_pulp_volume_ml
}

pub fn spleen_platelet_pool_fraction(s: &SpleenModel) -> f32 {
    /* red pulp sequesters platelets; normal ~30% of total platelet pool */
    (s.red_pulp_volume_ml / spleen_total_volume(s).max(1.0) * 0.3).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_volume_sum() {
        /* total volume = red + white pulp */
        let s = new_spleen_model();
        assert!((spleen_total_volume(&s) - 150.0).abs() < 1e-5);
    }

    #[test]
    fn not_splenomegaly_normally() {
        /* normal spleen is not enlarged */
        let s = new_spleen_model();
        assert!(!spleen_is_splenomegaly(&s));
    }

    #[test]
    fn splenomegaly_when_large() {
        /* enlarged spleen triggers splenomegaly */
        let mut s = new_spleen_model();
        s.red_pulp_volume_ml = 200.0;
        s.white_pulp_volume_ml = 100.0;
        assert!(spleen_is_splenomegaly(&s));
    }

    #[test]
    fn cells_filtered_positive() {
        /* filtration rate is positive */
        let s = new_spleen_model();
        assert!(spleen_cells_filtered_per_min(&s) > 0.0);
    }

    #[test]
    fn platelet_pool_fraction_in_range() {
        /* platelet pool fraction is between 0 and 1 */
        let s = new_spleen_model();
        let f = spleen_platelet_pool_fraction(&s);
        assert!((0.0..=1.0).contains(&f));
    }

    #[test]
    fn filtration_efficiency_used() {
        /* filtration rate scales with efficiency */
        let mut s = new_spleen_model();
        s.filtration_efficiency = 0.5;
        let half = spleen_cells_filtered_per_min(&s);
        s.filtration_efficiency = 1.0;
        let full = spleen_cells_filtered_per_min(&s);
        assert!((full - 2.0 * half).abs() < 1e-4);
    }

    #[test]
    fn blood_flow_positive() {
        /* blood flow is positive */
        let s = new_spleen_model();
        assert!(s.blood_flow_ml_per_min > 0.0);
    }
}
