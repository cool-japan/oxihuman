// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct HeartValve {
    pub is_open: bool,
    pub area_cm2: f32,
    pub max_area: f32,
    pub pressure_gradient: f32,
}

pub fn new_heart_valve(max_area: f32) -> HeartValve {
    HeartValve {
        is_open: false,
        area_cm2: 0.0,
        max_area,
        pressure_gradient: 0.0,
    }
}

pub fn valve_update(v: &mut HeartValve, upstream_p: f32, downstream_p: f32) {
    let grad = upstream_p - downstream_p;
    v.pressure_gradient = grad;
    if grad > 0.0 {
        v.is_open = true;
        v.area_cm2 = v.max_area;
    } else {
        v.is_open = false;
        v.area_cm2 = 0.0;
    }
}

pub fn valve_flow_rate_ml_per_s(v: &HeartValve) -> f32 {
    if !v.is_open || v.pressure_gradient <= 0.0 {
        return 0.0;
    }
    v.area_cm2 * v.pressure_gradient.sqrt()
}

pub fn valve_is_stenotic(v: &HeartValve) -> bool {
    v.area_cm2 < 0.5 * v.max_area && v.max_area > 0.0
}

pub fn valve_regurgitation_fraction(_v: &HeartValve, reverse_flow: f32, forward_flow: f32) -> f32 {
    if forward_flow <= 0.0 {
        return 0.0;
    }
    (reverse_flow / forward_flow).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_with_positive_gradient() {
        /* valve opens when upstream > downstream */
        let mut v = new_heart_valve(3.0);
        valve_update(&mut v, 120.0, 80.0);
        assert!(v.is_open);
    }

    #[test]
    fn closes_with_negative_gradient() {
        /* valve closes when upstream < downstream */
        let mut v = new_heart_valve(3.0);
        valve_update(&mut v, 70.0, 80.0);
        assert!(!v.is_open);
    }

    #[test]
    fn flow_rate_positive_when_open() {
        /* positive flow when open */
        let mut v = new_heart_valve(3.0);
        valve_update(&mut v, 120.0, 80.0);
        assert!(valve_flow_rate_ml_per_s(&v) > 0.0);
    }

    #[test]
    fn flow_rate_zero_when_closed() {
        /* zero flow when closed */
        let v = new_heart_valve(3.0);
        assert_eq!(valve_flow_rate_ml_per_s(&v), 0.0);
    }

    #[test]
    fn stenotic_when_area_small() {
        /* stenotic when area < 50% max */
        let mut v = new_heart_valve(3.0);
        v.area_cm2 = 1.0;
        assert!(valve_is_stenotic(&v));
    }

    #[test]
    fn not_stenotic_when_fully_open() {
        /* not stenotic when fully open */
        let mut v = new_heart_valve(3.0);
        valve_update(&mut v, 120.0, 80.0);
        assert!(!valve_is_stenotic(&v));
    }

    #[test]
    fn regurgitation_fraction_clamped() {
        /* regurgitation fraction is between 0 and 1 */
        let v = new_heart_valve(3.0);
        let frac = valve_regurgitation_fraction(&v, 100.0, 50.0);
        assert!((0.0..=1.0).contains(&frac));
    }
}
