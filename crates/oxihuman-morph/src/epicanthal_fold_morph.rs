// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct EpicanthalFoldMorph {
    pub coverage: f32,
    pub prominence: f32,
}

pub fn new_epicanthal_fold_morph() -> EpicanthalFoldMorph {
    EpicanthalFoldMorph {
        coverage: 0.0,
        prominence: 0.0,
    }
}

pub fn epicanthal_set_coverage(m: &mut EpicanthalFoldMorph, v: f32) {
    m.coverage = v.clamp(0.0, 1.0);
}

pub fn epicanthal_is_present(m: &EpicanthalFoldMorph) -> bool {
    m.coverage > 0.2
}

pub fn epicanthal_overall_weight(m: &EpicanthalFoldMorph) -> f32 {
    (m.coverage.abs() + m.prominence.abs()) * 0.5
}

pub fn epicanthal_blend(
    a: &EpicanthalFoldMorph,
    b: &EpicanthalFoldMorph,
    t: f32,
) -> EpicanthalFoldMorph {
    let t = t.clamp(0.0, 1.0);
    EpicanthalFoldMorph {
        coverage: a.coverage + (b.coverage - a.coverage) * t,
        prominence: a.prominence + (b.prominence - a.prominence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_epicanthal_fold_morph() {
        /* coverage defaults to 0 */
        let m = new_epicanthal_fold_morph();
        assert_eq!(m.coverage, 0.0);
    }

    #[test]
    fn test_epicanthal_set_coverage() {
        /* coverage is set */
        let mut m = new_epicanthal_fold_morph();
        epicanthal_set_coverage(&mut m, 0.5);
        assert!((m.coverage - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_epicanthal_is_present_false() {
        /* not present at 0 */
        let m = new_epicanthal_fold_morph();
        assert!(!epicanthal_is_present(&m));
    }

    #[test]
    fn test_epicanthal_is_present_true() {
        /* present when coverage > 0.2 */
        let mut m = new_epicanthal_fold_morph();
        m.coverage = 0.3;
        assert!(epicanthal_is_present(&m));
    }

    #[test]
    fn test_epicanthal_blend() {
        /* blend at 0.5 gives midpoint */
        let a = new_epicanthal_fold_morph();
        let b = EpicanthalFoldMorph {
            coverage: 1.0,
            prominence: 1.0,
        };
        let r = epicanthal_blend(&a, &b, 0.5);
        assert!((r.coverage - 0.5).abs() < 1e-6);
    }
}
