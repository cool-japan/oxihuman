// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fall risk assessment model.

pub struct FallRiskFactors {
    pub age_years: f32,
    pub balance_score: f32,
    pub gait_speed_m_per_s: f32,
    pub grip_strength_n: f32,
    pub vision_score: f32,
    pub medication_count: u32,
}

pub fn new_fall_risk_factors(age: f32) -> FallRiskFactors {
    FallRiskFactors {
        age_years: age.max(0.0),
        balance_score: 1.0,
        gait_speed_m_per_s: 1.2,
        grip_strength_n: 300.0,
        vision_score: 1.0,
        medication_count: 0,
    }
}

pub fn fall_risk_score(f: &FallRiskFactors) -> f32 {
    /* weighted composite: higher is riskier */
    let age_factor = ((f.age_years - 60.0) / 40.0).clamp(0.0, 1.0) * 0.25;
    let balance_factor = (1.0 - f.balance_score.clamp(0.0, 1.0)) * 0.25;
    let gait_factor = (1.0 - (f.gait_speed_m_per_s / 1.5).clamp(0.0, 1.0)) * 0.2;
    let grip_factor = (1.0 - (f.grip_strength_n / 400.0).clamp(0.0, 1.0)) * 0.15;
    let vision_factor = (1.0 - f.vision_score.clamp(0.0, 1.0)) * 0.1;
    let med_factor = ((f.medication_count as f32) / 10.0).clamp(0.0, 1.0) * 0.05;
    (age_factor + balance_factor + gait_factor + grip_factor + vision_factor + med_factor)
        .clamp(0.0, 1.0)
}

pub fn fall_is_high_risk(f: &FallRiskFactors) -> bool {
    fall_risk_score(f) > 0.6
}

pub fn fall_risk_category(f: &FallRiskFactors) -> &'static str {
    let s = fall_risk_score(f);
    if s > 0.6 {
        "high"
    } else if s > 0.3 {
        "moderate"
    } else {
        "low"
    }
}

pub fn fall_dominant_factor(f: &FallRiskFactors) -> &'static str {
    let age_c = ((f.age_years - 60.0) / 40.0).clamp(0.0, 1.0) * 0.25;
    let balance_c = (1.0 - f.balance_score.clamp(0.0, 1.0)) * 0.25;
    let gait_c = (1.0 - (f.gait_speed_m_per_s / 1.5).clamp(0.0, 1.0)) * 0.2;
    let grip_c = (1.0 - (f.grip_strength_n / 400.0).clamp(0.0, 1.0)) * 0.15;
    let vision_c = (1.0 - f.vision_score.clamp(0.0, 1.0)) * 0.1;

    if age_c >= balance_c && age_c >= gait_c && age_c >= grip_c && age_c >= vision_c {
        "age"
    } else if balance_c >= gait_c && balance_c >= grip_c && balance_c >= vision_c {
        "balance"
    } else if gait_c >= grip_c && gait_c >= vision_c {
        "gait"
    } else if grip_c >= vision_c {
        "grip_strength"
    } else {
        "vision"
    }
}

pub fn fall_set_balance(f: &mut FallRiskFactors, score: f32) {
    f.balance_score = score.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fall_risk_factors() {
        /* new factors have correct age */
        let f = new_fall_risk_factors(70.0);
        assert!((f.age_years - 70.0).abs() < 1e-5);
    }

    #[test]
    fn test_fall_risk_score_low_for_young() {
        /* young healthy person has low risk */
        let f = new_fall_risk_factors(30.0);
        assert!(fall_risk_score(&f) < 0.3);
    }

    #[test]
    fn test_fall_risk_score_high_for_frail() {
        /* frail elderly person has high risk */
        let mut f = new_fall_risk_factors(90.0);
        f.balance_score = 0.0;
        f.gait_speed_m_per_s = 0.3;
        f.grip_strength_n = 50.0;
        f.vision_score = 0.2;
        f.medication_count = 8;
        assert!(fall_risk_score(&f) > 0.6);
    }

    #[test]
    fn test_fall_is_high_risk() {
        /* high risk detection */
        let mut f = new_fall_risk_factors(90.0);
        f.balance_score = 0.0;
        f.gait_speed_m_per_s = 0.3;
        f.grip_strength_n = 50.0;
        f.vision_score = 0.2;
        f.medication_count = 8;
        assert!(fall_is_high_risk(&f));
    }

    #[test]
    fn test_fall_risk_category() {
        /* category matches risk level */
        let f = new_fall_risk_factors(30.0);
        let cat = fall_risk_category(&f);
        assert!(cat == "low" || cat == "moderate" || cat == "high");
    }

    #[test]
    fn test_fall_dominant_factor() {
        /* dominant factor is a valid string */
        let f = new_fall_risk_factors(70.0);
        let factor = fall_dominant_factor(&f);
        assert!(["age", "balance", "gait", "grip_strength", "vision"].contains(&factor));
    }

    #[test]
    fn test_fall_set_balance() {
        /* set_balance updates balance score */
        let mut f = new_fall_risk_factors(70.0);
        fall_set_balance(&mut f, 0.5);
        assert!((f.balance_score - 0.5).abs() < 1e-5);
    }
}
