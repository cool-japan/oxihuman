// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Iliac crest flare and height morph.
#[derive(Debug, Clone)]
pub struct IliacCrestMorph {
    /// Lateral flare of the crest (0.0 = narrow, 1.0 = wide flare).
    pub flare: f32,
    /// Vertical height of the crest above the acetabulum (0.0 = low, 1.0 = high).
    pub height: f32,
    /// Anterior-superior spine prominence (0.0 = flat, 1.0 = prominent).
    pub asis_prominence: f32,
}

pub fn new_iliac_crest_morph() -> IliacCrestMorph {
    IliacCrestMorph {
        flare: 0.0,
        height: 0.0,
        asis_prominence: 0.0,
    }
}

pub fn iliac_set_flare(m: &mut IliacCrestMorph, v: f32) {
    m.flare = v.clamp(0.0, 1.0);
}

pub fn iliac_set_height(m: &mut IliacCrestMorph, v: f32) {
    m.height = v.clamp(0.0, 1.0);
}

pub fn iliac_set_asis_prominence(m: &mut IliacCrestMorph, v: f32) {
    m.asis_prominence = v.clamp(0.0, 1.0);
}

pub fn iliac_overall_weight(m: &IliacCrestMorph) -> f32 {
    (m.flare + m.height + m.asis_prominence) / 3.0
}

pub fn iliac_blend(a: &IliacCrestMorph, b: &IliacCrestMorph, t: f32) -> IliacCrestMorph {
    let t = t.clamp(0.0, 1.0);
    IliacCrestMorph {
        flare: a.flare + (b.flare - a.flare) * t,
        height: a.height + (b.height - a.height) * t,
        asis_prominence: a.asis_prominence + (b.asis_prominence - a.asis_prominence) * t,
    }
}

pub fn iliac_is_neutral(m: &IliacCrestMorph) -> bool {
    m.flare < 1e-5 && m.height < 1e-5 && m.asis_prominence < 1e-5
}

pub fn iliac_to_json(m: &IliacCrestMorph) -> String {
    format!(
        r#"{{"flare":{:.4},"height":{:.4},"asis_prominence":{:.4}}}"#,
        m.flare, m.height, m.asis_prominence
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* all zero */
        let m = new_iliac_crest_morph();
        assert_eq!(m.flare, 0.0);
    }

    #[test]
    fn test_set_flare() {
        /* valid */
        let mut m = new_iliac_crest_morph();
        iliac_set_flare(&mut m, 0.5);
        assert!((m.flare - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_high() {
        /* clamp above 1 */
        let mut m = new_iliac_crest_morph();
        iliac_set_flare(&mut m, 3.0);
        assert_eq!(m.flare, 1.0);
    }

    #[test]
    fn test_clamp_low() {
        /* clamp below 0 */
        let mut m = new_iliac_crest_morph();
        iliac_set_height(&mut m, -0.5);
        assert_eq!(m.height, 0.0);
    }

    #[test]
    fn test_neutral_true() {
        /* default neutral */
        assert!(iliac_is_neutral(&new_iliac_crest_morph()));
    }

    #[test]
    fn test_neutral_false() {
        /* after setting flare */
        let mut m = new_iliac_crest_morph();
        iliac_set_flare(&mut m, 0.3);
        assert!(!iliac_is_neutral(&m));
    }

    #[test]
    fn test_weight() {
        /* equal average */
        let m = IliacCrestMorph {
            flare: 0.9,
            height: 0.9,
            asis_prominence: 0.9,
        };
        assert!((iliac_overall_weight(&m) - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_blend() {
        /* midpoint */
        let a = new_iliac_crest_morph();
        let b = IliacCrestMorph {
            flare: 1.0,
            height: 0.0,
            asis_prominence: 0.0,
        };
        let c = iliac_blend(&a, &b, 0.5);
        assert!((c.flare - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        /* has flare key */
        assert!(iliac_to_json(&new_iliac_crest_morph()).contains("flare"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let m = IliacCrestMorph {
            flare: 0.3,
            height: 0.4,
            asis_prominence: 0.5,
        };
        let m2 = m.clone();
        assert_eq!(m.height, m2.height);
    }
}
