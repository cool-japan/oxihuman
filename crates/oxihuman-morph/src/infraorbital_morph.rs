// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct InfraorbitalMorph {
    pub hollow: f32,
    pub puffiness: f32,
    pub tear_trough: f32,
}

pub fn new_infraorbital_morph() -> InfraorbitalMorph {
    InfraorbitalMorph {
        hollow: 0.0,
        puffiness: 0.0,
        tear_trough: 0.0,
    }
}

pub fn io_set_hollow(m: &mut InfraorbitalMorph, v: f32) {
    m.hollow = v.clamp(0.0, 1.0);
}

pub fn io_set_puffiness(m: &mut InfraorbitalMorph, v: f32) {
    m.puffiness = v.clamp(0.0, 1.0);
}

pub fn io_overall_weight(m: &InfraorbitalMorph) -> f32 {
    (m.hollow + m.puffiness + m.tear_trough) / 3.0
}

pub fn io_blend(a: &InfraorbitalMorph, b: &InfraorbitalMorph, t: f32) -> InfraorbitalMorph {
    let t = t.clamp(0.0, 1.0);
    InfraorbitalMorph {
        hollow: a.hollow + (b.hollow - a.hollow) * t,
        puffiness: a.puffiness + (b.puffiness - a.puffiness) * t,
        tear_trough: a.tear_trough + (b.tear_trough - a.tear_trough) * t,
    }
}

pub fn io_is_hollow(m: &InfraorbitalMorph) -> bool {
    m.hollow > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* all zero */
        let m = new_infraorbital_morph();
        assert_eq!(m.hollow, 0.0);
    }

    #[test]
    fn test_set_hollow() {
        /* stores value */
        let mut m = new_infraorbital_morph();
        io_set_hollow(&mut m, 0.6);
        assert!((m.hollow - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_set_hollow_clamp() {
        /* clamp high */
        let mut m = new_infraorbital_morph();
        io_set_hollow(&mut m, 3.0);
        assert_eq!(m.hollow, 1.0);
    }

    #[test]
    fn test_set_puffiness_clamp() {
        /* clamp low */
        let mut m = new_infraorbital_morph();
        io_set_puffiness(&mut m, -1.0);
        assert_eq!(m.puffiness, 0.0);
    }

    #[test]
    fn test_overall_weight() {
        /* average */
        let m = InfraorbitalMorph {
            hollow: 0.3,
            puffiness: 0.6,
            tear_trough: 0.9,
        };
        assert!((io_overall_weight(&m) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_is_hollow_false() {
        /* default not hollow */
        let m = new_infraorbital_morph();
        assert!(!io_is_hollow(&m));
    }

    #[test]
    fn test_is_hollow_true() {
        /* above threshold */
        let m = InfraorbitalMorph {
            hollow: 0.8,
            puffiness: 0.0,
            tear_trough: 0.0,
        };
        assert!(io_is_hollow(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = InfraorbitalMorph {
            hollow: 0.0,
            puffiness: 0.0,
            tear_trough: 0.0,
        };
        let b = InfraorbitalMorph {
            hollow: 1.0,
            puffiness: 1.0,
            tear_trough: 1.0,
        };
        let c = io_blend(&a, &b, 0.5);
        assert!((c.hollow - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = InfraorbitalMorph {
            hollow: 0.3,
            puffiness: 0.4,
            tear_trough: 0.5,
        };
        let m2 = m.clone();
        assert_eq!(m.hollow, m2.hollow);
    }
}
