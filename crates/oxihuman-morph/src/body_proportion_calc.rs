// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Body proportion calculator holding key measurements.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BodyProportionCalc {
    pub head_height: f32,
    pub body_height: f32,
    pub shoulder_width: f32,
    pub hip_width: f32,
    pub leg_length: f32,
    pub arm_span: f32,
    pub torso_length: f32,
}

/// Compute head-to-body ratio (number of head-heights in total height).
#[allow(dead_code)]
pub fn head_to_body_ratio(calc: &BodyProportionCalc) -> f32 {
    if calc.head_height.abs() < 1e-9 {
        return 0.0;
    }
    calc.body_height / calc.head_height
}

/// Compute shoulder-to-hip width ratio.
#[allow(dead_code)]
pub fn shoulder_to_hip_ratio(calc: &BodyProportionCalc) -> f32 {
    if calc.hip_width.abs() < 1e-9 {
        return 0.0;
    }
    calc.shoulder_width / calc.hip_width
}

/// Compute leg-to-body height ratio.
#[allow(dead_code)]
pub fn leg_to_body_ratio(calc: &BodyProportionCalc) -> f32 {
    if calc.body_height.abs() < 1e-9 {
        return 0.0;
    }
    calc.leg_length / calc.body_height
}

/// Compute arm span to body height ratio.
#[allow(dead_code)]
pub fn arm_span_ratio(calc: &BodyProportionCalc) -> f32 {
    if calc.body_height.abs() < 1e-9 {
        return 0.0;
    }
    calc.arm_span / calc.body_height
}

/// Compute torso-to-body height ratio.
#[allow(dead_code)]
pub fn torso_ratio(calc: &BodyProportionCalc) -> f32 {
    if calc.body_height.abs() < 1e-9 {
        return 0.0;
    }
    calc.torso_length / calc.body_height
}

/// Score how close proportions are to an ideal (0=perfect, higher=worse).
#[allow(dead_code)]
pub fn proportion_score(calc: &BodyProportionCalc) -> f32 {
    let ideal = ideal_proportions();
    let dh = head_to_body_ratio(calc) - head_to_body_ratio(&ideal);
    let ds = shoulder_to_hip_ratio(calc) - shoulder_to_hip_ratio(&ideal);
    let dl = leg_to_body_ratio(calc) - leg_to_body_ratio(&ideal);
    (dh * dh + ds * ds + dl * dl).sqrt()
}

/// Serialize proportions to a JSON string.
#[allow(dead_code)]
pub fn proportion_to_json(calc: &BodyProportionCalc) -> String {
    format!(
        "{{\"head_to_body\":{:.4},\"shoulder_to_hip\":{:.4},\"leg_to_body\":{:.4},\"arm_span\":{:.4},\"torso\":{:.4}}}",
        head_to_body_ratio(calc),
        shoulder_to_hip_ratio(calc),
        leg_to_body_ratio(calc),
        arm_span_ratio(calc),
        torso_ratio(calc),
    )
}

/// Return ideal adult proportions (7.5 heads tall, etc.).
#[allow(dead_code)]
pub fn ideal_proportions() -> BodyProportionCalc {
    BodyProportionCalc {
        head_height: 0.24,
        body_height: 1.80,
        shoulder_width: 0.46,
        hip_width: 0.36,
        leg_length: 0.86,
        arm_span: 1.80,
        torso_length: 0.52,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideal_head_ratio() {
        let p = ideal_proportions();
        let r = head_to_body_ratio(&p);
        assert!(r > 7.0 && r < 8.0);
    }

    #[test]
    fn ideal_shoulder_hip() {
        let p = ideal_proportions();
        let r = shoulder_to_hip_ratio(&p);
        assert!(r > 1.0);
    }

    #[test]
    fn ideal_leg_ratio() {
        let p = ideal_proportions();
        let r = leg_to_body_ratio(&p);
        assert!((0.3..=0.7).contains(&r));
    }

    #[test]
    fn ideal_arm_span() {
        let p = ideal_proportions();
        let r = arm_span_ratio(&p);
        assert!((r - 1.0).abs() < 0.1);
    }

    #[test]
    fn ideal_torso() {
        let p = ideal_proportions();
        let r = torso_ratio(&p);
        assert!(r > 0.1 && r < 0.5);
    }

    #[test]
    fn ideal_score_zero() {
        let p = ideal_proportions();
        assert!(proportion_score(&p) < 1e-6);
    }

    #[test]
    fn score_deviates() {
        let mut p = ideal_proportions();
        p.head_height = 0.30;
        assert!(proportion_score(&p) > 0.0);
    }

    #[test]
    fn to_json() {
        let p = ideal_proportions();
        let j = proportion_to_json(&p);
        assert!(j.contains("head_to_body"));
    }

    #[test]
    fn zero_head_height() {
        let mut p = ideal_proportions();
        p.head_height = 0.0;
        assert!(head_to_body_ratio(&p).abs() < 1e-6);
    }

    #[test]
    fn zero_body_height() {
        let mut p = ideal_proportions();
        p.body_height = 0.0;
        assert!(leg_to_body_ratio(&p).abs() < 1e-6);
    }
}
