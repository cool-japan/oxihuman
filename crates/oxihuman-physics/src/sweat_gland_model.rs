// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Sweat gland thermoregulation model.
pub struct SweatGland {
    pub core_temp_c: f32,
    pub threshold_temp_c: f32,
    pub max_sweat_rate_ml_per_min: f32,
    pub sweat_latent_heat: f32, // J/mL
}

impl SweatGland {
    pub fn new() -> Self {
        SweatGland {
            core_temp_c: 37.0,
            threshold_temp_c: 37.0,
            max_sweat_rate_ml_per_min: 1.5,
            sweat_latent_heat: 2430.0, // J/g ≈ J/mL
        }
    }
}

impl Default for SweatGland {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_sweat_gland() -> SweatGland {
    SweatGland::new()
}

/// Sweat rate proportional to temperature above threshold (mL/min).
pub fn sweat_rate(g: &SweatGland) -> f32 {
    let delta = g.core_temp_c - g.threshold_temp_c;
    if delta <= 0.0 {
        return 0.0;
    }
    // Assume 1°C above threshold → max rate; clamp linearly
    (g.max_sweat_rate_ml_per_min * delta).min(g.max_sweat_rate_ml_per_min)
}

/// Heat loss = sweat_rate (mL/min) * latent_heat (J/mL) / 60 = W
pub fn sweat_heat_loss(g: &SweatGland) -> f32 {
    sweat_rate(g) * g.sweat_latent_heat / 60.0
}

pub fn sweat_is_active(g: &SweatGland) -> bool {
    sweat_rate(g) > 0.0
}

pub fn sweat_set_core_temp(g: &mut SweatGland, temp: f32) {
    g.core_temp_c = temp;
}

/// Cooling power in watts.
pub fn sweat_cooling_power_w(g: &SweatGland) -> f32 {
    sweat_heat_loss(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* new gland has threshold 37°C */
        let g = new_sweat_gland();
        assert!((g.threshold_temp_c - 37.0).abs() < 1e-4);
    }

    #[test]
    fn test_no_sweat_at_threshold() {
        /* no sweat when core temp = threshold */
        let g = new_sweat_gland();
        assert!((sweat_rate(&g) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_sweat_above_threshold() {
        /* sweat rate > 0 above threshold */
        let mut g = new_sweat_gland();
        sweat_set_core_temp(&mut g, 38.0);
        assert!(sweat_rate(&g) > 0.0);
    }

    #[test]
    fn test_sweat_capped() {
        /* sweat rate does not exceed max */
        let mut g = new_sweat_gland();
        sweat_set_core_temp(&mut g, 42.0);
        assert!(sweat_rate(&g) <= g.max_sweat_rate_ml_per_min + 1e-6);
    }

    #[test]
    fn test_heat_loss_positive() {
        /* heat loss positive when sweating */
        let mut g = new_sweat_gland();
        sweat_set_core_temp(&mut g, 38.0);
        assert!(sweat_heat_loss(&g) > 0.0);
    }

    #[test]
    fn test_is_active() {
        /* gland is active above threshold */
        let mut g = new_sweat_gland();
        sweat_set_core_temp(&mut g, 37.5);
        assert!(sweat_is_active(&g));
    }

    #[test]
    fn test_cooling_power_matches_heat_loss() {
        /* cooling_power_w == sweat_heat_loss */
        let mut g = new_sweat_gland();
        sweat_set_core_temp(&mut g, 38.0);
        assert!((sweat_cooling_power_w(&g) - sweat_heat_loss(&g)).abs() < 1e-9);
    }
}
