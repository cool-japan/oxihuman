// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct WaistMorph {
    pub narrowing: f32,
    pub height: f32,
    pub front_flatness: f32,
}

pub fn new_waist_morph() -> WaistMorph {
    WaistMorph {
        narrowing: 0.3,
        height: 0.5,
        front_flatness: 0.3,
    }
}

pub fn waist_set_narrowing(m: &mut WaistMorph, v: f32) {
    m.narrowing = v.clamp(0.0, 1.0);
}

pub fn waist_is_hourglass(m: &WaistMorph) -> bool {
    m.narrowing > 0.5
}

pub fn waist_overall_weight(m: &WaistMorph) -> f32 {
    (m.narrowing + m.height * 0.5 + m.front_flatness * 0.5) / 2.0
}

pub fn waist_blend(a: &WaistMorph, b: &WaistMorph, t: f32) -> WaistMorph {
    let t = t.clamp(0.0, 1.0);
    WaistMorph {
        narrowing: a.narrowing + (b.narrowing - a.narrowing) * t,
        height: a.height + (b.height - a.height) * t,
        front_flatness: a.front_flatness + (b.front_flatness - a.front_flatness) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* narrowing > 0 by default */
        let m = new_waist_morph();
        assert!(m.narrowing > 0.0);
    }

    #[test]
    fn test_set_narrowing() {
        /* clamped to range */
        let mut m = new_waist_morph();
        waist_set_narrowing(&mut m, 1.5);
        assert!((m.narrowing - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_hourglass() {
        /* threshold check */
        let mut m = new_waist_morph();
        waist_set_narrowing(&mut m, 0.7);
        assert!(waist_is_hourglass(&m));
    }

    #[test]
    fn test_overall_weight_nonneg() {
        /* non-negative */
        let m = new_waist_morph();
        assert!(waist_overall_weight(&m) >= 0.0);
    }

    #[test]
    fn test_blend() {
        /* midpoint blend */
        let a = WaistMorph {
            narrowing: 0.0,
            height: 0.0,
            front_flatness: 0.0,
        };
        let b = WaistMorph {
            narrowing: 1.0,
            height: 1.0,
            front_flatness: 1.0,
        };
        let c = waist_blend(&a, &b, 0.5);
        assert!((c.narrowing - 0.5).abs() < 1e-5);
    }
}
