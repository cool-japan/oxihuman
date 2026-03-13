// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ScmMorph {
    pub definition: f32,
    pub length: f32,
    pub prominence: f32,
}

pub fn new_scm_morph() -> ScmMorph {
    ScmMorph {
        definition: 0.0,
        length: 0.5,
        prominence: 0.0,
    }
}

pub fn scm_set_definition(m: &mut ScmMorph, v: f32) {
    m.definition = v.clamp(0.0, 1.0);
}

pub fn scm_is_defined(m: &ScmMorph) -> bool {
    m.definition > 0.4
}

pub fn scm_overall_weight(m: &ScmMorph) -> f32 {
    (m.definition + m.prominence) * 0.5
}

pub fn scm_blend(a: &ScmMorph, b: &ScmMorph, t: f32) -> ScmMorph {
    let t = t.clamp(0.0, 1.0);
    ScmMorph {
        definition: a.definition + (b.definition - a.definition) * t,
        length: a.length + (b.length - a.length) * t,
        prominence: a.prominence + (b.prominence - a.prominence) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scm_morph() {
        /* definition starts at 0 */
        let m = new_scm_morph();
        assert!((m.definition - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_scm_set_definition() {
        /* set definition */
        let mut m = new_scm_morph();
        scm_set_definition(&mut m, 0.5);
        assert!((m.definition - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_scm_is_defined_true() {
        /* > 0.4 is defined */
        let mut m = new_scm_morph();
        scm_set_definition(&mut m, 0.6);
        assert!(scm_is_defined(&m));
    }

    #[test]
    fn test_scm_is_defined_false() {
        /* 0 not defined */
        let m = new_scm_morph();
        assert!(!scm_is_defined(&m));
    }

    #[test]
    fn test_scm_blend() {
        /* blend midpoint */
        let a = new_scm_morph();
        let mut b = new_scm_morph();
        b.definition = 1.0;
        let mid = scm_blend(&a, &b, 0.5);
        assert!((mid.definition - 0.5).abs() < 1e-6);
    }
}
