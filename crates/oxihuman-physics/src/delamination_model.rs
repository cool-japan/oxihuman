// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Layer delamination failure model stub.
//!
//! Models interlaminar failure in layered composites using a cohesive zone
//! approach with bilinear traction-separation law.

/// Traction-separation law parameters for an interface.
#[derive(Debug, Clone)]
pub struct CohesiveParams {
    /// Peak traction `[MPa]`.
    pub peak_traction: f64,
    /// Critical separation at peak `[mm]`.
    pub delta_0: f64,
    /// Final separation (complete failure) `[mm]`.
    pub delta_f: f64,
    /// Mode I fracture energy [J/m²].
    pub g_c: f64,
}

impl Default for CohesiveParams {
    fn default() -> Self {
        Self {
            peak_traction: 50.0,
            delta_0: 0.01,
            delta_f: 0.1,
            g_c: 300.0,
        }
    }
}

/// State of a cohesive interface element.
#[derive(Debug, Clone)]
pub struct InterfaceElement {
    pub separation: f64,
    pub damage: f64,
    pub failed: bool,
}

impl InterfaceElement {
    pub fn new() -> Self {
        Self {
            separation: 0.0,
            damage: 0.0,
            failed: false,
        }
    }

    pub fn is_intact(&self) -> bool {
        !self.failed && self.damage < 1.0
    }
}

impl Default for InterfaceElement {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute traction from a bilinear cohesive law given current separation.
pub fn bilinear_traction(separation: f64, params: &CohesiveParams) -> f64 {
    if separation <= 0.0 {
        return 0.0;
    }
    if separation <= params.delta_0 {
        params.peak_traction * separation / params.delta_0
    } else if separation < params.delta_f {
        params.peak_traction * (params.delta_f - separation) / (params.delta_f - params.delta_0)
    } else {
        0.0
    }
}

/// Compute the damage variable D ∈ [0, 1] from separation.
pub fn damage_variable(separation: f64, params: &CohesiveParams) -> f64 {
    if separation <= params.delta_0 {
        return 0.0;
    }
    if separation >= params.delta_f {
        return 1.0;
    }
    (separation - params.delta_0) / (params.delta_f - params.delta_0)
}

/// Update an interface element given a new separation.
pub fn update_interface(elem: &mut InterfaceElement, new_sep: f64, params: &CohesiveParams) {
    if elem.failed {
        return;
    }
    let effective_sep = new_sep.max(elem.separation);
    elem.separation = effective_sep;
    elem.damage = damage_variable(effective_sep, params);
    if elem.damage >= 1.0 {
        elem.failed = true;
    }
}

/// Compute the fracture energy dissipated by the interface.
pub fn dissipated_energy(elem: &InterfaceElement, params: &CohesiveParams) -> f64 {
    /* Area under bilinear T-S curve up to current separation */
    let s = elem.separation.min(params.delta_f);
    if s <= params.delta_0 {
        0.5 * params.peak_traction * s * s / params.delta_0
    } else {
        let e0 = 0.5 * params.peak_traction * params.delta_0;
        let e1 = 0.5
            * params.peak_traction
            * (s - params.delta_0)
            * (1.0 + (params.delta_f - s) / (params.delta_f - params.delta_0));
        e0 + e1
    }
}

/// Check if an interface array represents a complete delamination front.
pub fn is_delaminated(elements: &[InterfaceElement]) -> bool {
    elements.iter().all(|e| e.failed)
}

/// Count the number of failed interface elements.
pub fn failed_element_count(elements: &[InterfaceElement]) -> usize {
    elements.iter().filter(|e| e.failed).count()
}

/// Simulate delamination growth step for an array of interface elements.
pub fn grow_delamination(
    elements: &mut [InterfaceElement],
    applied_sep: &[f64],
    params: &CohesiveParams,
) {
    for (elem, &sep) in elements.iter_mut().zip(applied_sep.iter()) {
        update_interface(elem, sep, params);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> CohesiveParams {
        CohesiveParams::default()
    }

    #[test]
    fn test_traction_zero_at_zero_sep() {
        assert_eq!(bilinear_traction(0.0, &default_params()), 0.0);
    }

    #[test]
    fn test_traction_peak_at_delta0() {
        let p = default_params();
        let t = bilinear_traction(p.delta_0, &p);
        assert!((t - p.peak_traction).abs() < 1e-9);
    }

    #[test]
    fn test_traction_zero_at_delta_f() {
        let p = default_params();
        assert_eq!(bilinear_traction(p.delta_f, &p), 0.0);
    }

    #[test]
    fn test_damage_zero_before_delta0() {
        let p = default_params();
        assert_eq!(damage_variable(0.0, &p), 0.0);
    }

    #[test]
    fn test_damage_one_at_delta_f() {
        let p = default_params();
        assert!((damage_variable(p.delta_f, &p) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_update_interface_sets_failed() {
        let p = default_params();
        let mut elem = InterfaceElement::new();
        update_interface(&mut elem, p.delta_f + 0.001, &p);
        assert!(elem.failed);
    }

    #[test]
    fn test_dissipated_energy_positive() {
        let p = default_params();
        let mut elem = InterfaceElement::new();
        update_interface(&mut elem, p.delta_0, &p);
        assert!(dissipated_energy(&elem, &p) >= 0.0);
    }

    #[test]
    fn test_is_delaminated_false() {
        let elems = vec![InterfaceElement::new(), InterfaceElement::new()];
        assert!(!is_delaminated(&elems));
    }

    #[test]
    fn test_failed_element_count() {
        let mut elems = vec![InterfaceElement::new(), InterfaceElement::new()];
        elems[0].failed = true;
        assert_eq!(failed_element_count(&elems), 1);
    }
}
