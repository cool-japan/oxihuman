// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct HeatConduction {
    pub conductivity: f32,
    pub capacity: f32,
    pub density: f32,
}

#[allow(dead_code)]
pub fn new_heat_conduction(conductivity: f32, capacity: f32, density: f32) -> HeatConduction {
    HeatConduction { conductivity, capacity, density }
}

#[allow(dead_code)]
pub fn hc_diffusivity(h: &HeatConduction) -> f32 {
    h.conductivity / (h.capacity * h.density)
}

#[allow(dead_code)]
pub fn hc_fourier_step(h: &HeatConduction, temp: &mut [f32], dt: f32) {
    let n = temp.len();
    if n < 3 {
        return;
    }
    let alpha = hc_diffusivity(h);
    let old = temp.to_vec();
    for i in 1..n - 1 {
        temp[i] = old[i] + alpha * dt * (old[i + 1] - 2.0 * old[i] + old[i - 1]);
    }
}

#[allow(dead_code)]
pub fn hc_steady_state_1d(_h: &HeatConduction, t_left: f32, t_right: f32, n: usize) -> Vec<f32> {
    if n == 0 {
        return Vec::new();
    }
    (0..n).map(|i| {
        let t = i as f32 / (n - 1).max(1) as f32;
        t_left + (t_right - t_left) * t
    }).collect()
}

#[allow(dead_code)]
pub fn hc_heat_flux(h: &HeatConduction, grad_t: f32) -> f32 {
    -h.conductivity * grad_t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diffusivity_positive() {
        let h = new_heat_conduction(1.0, 1.0, 1.0);
        assert!(hc_diffusivity(&h) > 0.0);
    }

    #[test]
    fn test_diffusivity_formula() {
        let h = new_heat_conduction(2.0, 4.0, 0.5);
        assert!((hc_diffusivity(&h) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fourier_step_no_crash() {
        let h = new_heat_conduction(1.0, 1.0, 1.0);
        let mut temp = vec![0.0f32, 10.0, 0.0, 0.0, 0.0];
        hc_fourier_step(&h, &mut temp, 0.01);
        assert!(temp.iter().all(|&t| t.is_finite()));
    }

    #[test]
    fn test_steady_state_endpoints() {
        let h = new_heat_conduction(1.0, 1.0, 1.0);
        let temps = hc_steady_state_1d(&h, 300.0, 400.0, 11);
        assert!((temps[0] - 300.0).abs() < 1e-4);
        assert!((temps[10] - 400.0).abs() < 1e-4);
    }

    #[test]
    fn test_steady_state_linear() {
        let h = new_heat_conduction(1.0, 1.0, 1.0);
        let temps = hc_steady_state_1d(&h, 0.0, 10.0, 11);
        assert!((temps[5] - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_heat_flux_sign() {
        let h = new_heat_conduction(2.0, 1.0, 1.0);
        let flux = hc_heat_flux(&h, 5.0);
        assert!(flux < 0.0);
    }

    #[test]
    fn test_heat_flux_value() {
        let h = new_heat_conduction(3.0, 1.0, 1.0);
        let flux = hc_heat_flux(&h, 2.0);
        assert!((flux + 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_steady_state_empty() {
        let h = new_heat_conduction(1.0, 1.0, 1.0);
        let temps = hc_steady_state_1d(&h, 0.0, 100.0, 0);
        assert!(temps.is_empty());
    }
}
