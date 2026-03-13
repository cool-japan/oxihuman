// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Bladder {
    pub volume_ml: f32,
    pub capacity_ml: f32,
    pub compliance: f32,
    pub fill_rate_ml_per_h: f32,
    pub is_voiding: bool,
}

pub fn new_bladder() -> Bladder {
    Bladder {
        volume_ml: 0.0,
        capacity_ml: 500.0,
        compliance: 10.0,
        fill_rate_ml_per_h: 60.0,
        is_voiding: false,
    }
}

pub fn bladder_step(b: &mut Bladder, dt_hours: f32) {
    if b.is_voiding {
        b.volume_ml -= b.volume_ml * 2.0 * dt_hours;
        b.volume_ml = b.volume_ml.max(0.0);
        if b.volume_ml < 1.0 {
            b.is_voiding = false;
        }
    } else {
        b.volume_ml += b.fill_rate_ml_per_h * dt_hours;
        b.volume_ml = b.volume_ml.min(b.capacity_ml);
    }
}

pub fn bladder_fullness(b: &Bladder) -> f32 {
    if b.capacity_ml <= 0.0 {
        return 0.0;
    }
    b.volume_ml / b.capacity_ml
}

pub fn bladder_urge_threshold(b: &Bladder) -> bool {
    bladder_fullness(b) > 0.6
}

pub fn bladder_void(b: &mut Bladder) {
    b.volume_ml = 0.0;
    b.is_voiding = false;
}

pub fn bladder_pressure_cmh2o(b: &Bladder) -> f32 {
    b.volume_ml / b.compliance.max(1e-9)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_volume_zero() {
        /* starts empty */
        let b = new_bladder();
        assert_eq!(b.volume_ml, 0.0);
    }

    #[test]
    fn fill_over_time() {
        /* volume increases over time */
        let mut b = new_bladder();
        bladder_step(&mut b, 1.0);
        assert!(b.volume_ml > 0.0);
    }

    #[test]
    fn fullness_between_0_and_1() {
        /* fullness is in [0,1] */
        let b = new_bladder();
        assert!(bladder_fullness(&b) >= 0.0 && bladder_fullness(&b) <= 1.0);
    }

    #[test]
    fn no_urge_when_empty() {
        /* no urge when empty */
        let b = new_bladder();
        assert!(!bladder_urge_threshold(&b));
    }

    #[test]
    fn urge_when_full() {
        /* urge when > 60% full */
        let mut b = new_bladder();
        b.volume_ml = 350.0;
        assert!(bladder_urge_threshold(&b));
    }

    #[test]
    fn void_empties_bladder() {
        /* voiding empties bladder */
        let mut b = new_bladder();
        b.volume_ml = 300.0;
        bladder_void(&mut b);
        assert_eq!(b.volume_ml, 0.0);
    }

    #[test]
    fn pressure_increases_with_volume() {
        /* pressure increases as bladder fills */
        let mut b = new_bladder();
        b.volume_ml = 100.0;
        let p1 = bladder_pressure_cmh2o(&b);
        b.volume_ml = 200.0;
        let p2 = bladder_pressure_cmh2o(&b);
        assert!(p2 > p1);
    }
}
