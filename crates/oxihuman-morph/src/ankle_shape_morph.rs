// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct AnkleMorph {
    pub width: f32,
    pub malleolus_prominence: f32,
    pub achilles_definition: f32,
}

pub fn new_ankle_morph() -> AnkleMorph {
    AnkleMorph {
        width: 0.4,
        malleolus_prominence: 0.3,
        achilles_definition: 0.2,
    }
}

pub fn ankle_set_width(m: &mut AnkleMorph, v: f32) {
    m.width = v.clamp(0.0, 1.0);
}

pub fn ankle_is_slender(m: &AnkleMorph) -> bool {
    m.width < 0.3
}

pub fn ankle_overall_weight(m: &AnkleMorph) -> f32 {
    (m.width + m.malleolus_prominence + m.achilles_definition) / 3.0
}

pub fn ankle_blend(a: &AnkleMorph, b: &AnkleMorph, t: f32) -> AnkleMorph {
    let t = t.clamp(0.0, 1.0);
    AnkleMorph {
        width: a.width + (b.width - a.width) * t,
        malleolus_prominence: a.malleolus_prominence
            + (b.malleolus_prominence - a.malleolus_prominence) * t,
        achilles_definition: a.achilles_definition
            + (b.achilles_definition - a.achilles_definition) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* default width set */
        let m = new_ankle_morph();
        assert!(m.width > 0.0);
    }

    #[test]
    fn test_set_width() {
        /* clamped */
        let mut m = new_ankle_morph();
        ankle_set_width(&mut m, 0.2);
        assert!((m.width - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_is_slender_when_narrow() {
        /* slender if width < 0.3 */
        let mut m = new_ankle_morph();
        ankle_set_width(&mut m, 0.2);
        assert!(ankle_is_slender(&m));
    }

    #[test]
    fn test_not_slender_by_default() {
        /* default width 0.4 is not slender */
        let m = new_ankle_morph();
        assert!(!ankle_is_slender(&m));
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = AnkleMorph {
            width: 0.0,
            malleolus_prominence: 0.0,
            achilles_definition: 0.0,
        };
        let b = AnkleMorph {
            width: 1.0,
            malleolus_prominence: 1.0,
            achilles_definition: 1.0,
        };
        let c = ankle_blend(&a, &b, 0.5);
        assert!((c.width - 0.5).abs() < 1e-5);
    }
}
