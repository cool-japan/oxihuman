// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct TubularReabsorption {
    pub max_transport_rate_mg_per_min: f32,
    pub plasma_concentration_mg_per_dl: f32,
    pub gfr_ml_per_min: f32,
}

pub fn new_tubular_reabsorption(tm: f32) -> TubularReabsorption {
    TubularReabsorption {
        max_transport_rate_mg_per_min: tm,
        plasma_concentration_mg_per_dl: 100.0,
        gfr_ml_per_min: 125.0,
    }
}

pub fn tubular_filtered_load(r: &TubularReabsorption) -> f32 {
    r.gfr_ml_per_min * r.plasma_concentration_mg_per_dl / 100.0
}

pub fn tubular_reabsorption_rate(r: &TubularReabsorption) -> f32 {
    let fl = tubular_filtered_load(r);
    fl.min(r.max_transport_rate_mg_per_min)
}

pub fn tubular_excretion_rate(r: &TubularReabsorption) -> f32 {
    (tubular_filtered_load(r) - tubular_reabsorption_rate(r)).max(0.0)
}

pub fn tubular_threshold_concentration(r: &TubularReabsorption) -> f32 {
    if r.gfr_ml_per_min <= 0.0 {
        return f32::MAX;
    }
    r.max_transport_rate_mg_per_min / r.gfr_ml_per_min * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtered_load_positive() {
        /* filtered load is positive */
        let r = new_tubular_reabsorption(375.0);
        assert!(tubular_filtered_load(&r) > 0.0);
    }

    #[test]
    fn reabsorption_bounded_by_tm() {
        /* reabsorption cannot exceed Tm */
        let r = new_tubular_reabsorption(375.0);
        assert!(tubular_reabsorption_rate(&r) <= r.max_transport_rate_mg_per_min);
    }

    #[test]
    fn excretion_nonneg() {
        /* excretion rate is non-negative */
        let r = new_tubular_reabsorption(375.0);
        assert!(tubular_excretion_rate(&r) >= 0.0);
    }

    #[test]
    fn threshold_concentration_positive() {
        /* threshold concentration is positive */
        let r = new_tubular_reabsorption(375.0);
        assert!(tubular_threshold_concentration(&r) > 0.0);
    }

    #[test]
    fn above_threshold_causes_excretion() {
        /* concentration above threshold leads to excretion */
        let mut r = new_tubular_reabsorption(100.0);
        r.plasma_concentration_mg_per_dl = 300.0; /* well above threshold */
        assert!(tubular_excretion_rate(&r) > 0.0);
    }

    #[test]
    fn below_threshold_no_excretion() {
        /* low concentration: no excretion */
        let mut r = new_tubular_reabsorption(10000.0);
        r.plasma_concentration_mg_per_dl = 10.0;
        assert_eq!(tubular_excretion_rate(&r), 0.0);
    }

    #[test]
    fn filtered_minus_reabsorbed_equals_excreted() {
        /* mass balance: filtered = reabsorbed + excreted */
        let r = new_tubular_reabsorption(375.0);
        let fl = tubular_filtered_load(&r);
        let ra = tubular_reabsorption_rate(&r);
        let ex = tubular_excretion_rate(&r);
        assert!((fl - ra - ex).abs() < 1e-4);
    }
}
