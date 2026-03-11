#![allow(dead_code)]
//! Body aging simulation affecting skin, posture, and body composition.

/// Aging parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AgingParams {
    /// Age in years.
    pub age: f32,
    /// Skin elasticity factor in [0, 1] (1 = young).
    pub skin_elasticity: f32,
    /// Muscle loss factor in [0, 1].
    pub muscle_loss: f32,
    /// Fat gain factor in [0, 1].
    pub fat_gain: f32,
    /// Height change factor (negative = shrinkage).
    pub height_change: f32,
}

/// Body aging state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyAging {
    /// Current aging parameters.
    pub params: AgingParams,
    /// Human-readable label.
    pub label: String,
}

/// Create a new [`BodyAging`] at a given age.
#[allow(dead_code)]
pub fn new_body_aging(label: &str, age: f32) -> BodyAging {
    let age = age.clamp(0.0, 120.0);
    BodyAging {
        params: AgingParams {
            age,
            skin_elasticity: (1.0 - age / 120.0).clamp(0.0, 1.0),
            muscle_loss: 0.0,
            fat_gain: 0.0,
            height_change: 0.0,
        },
        label: label.to_string(),
    }
}

/// Compute skin aging effects: returns elasticity reduction factor.
#[allow(dead_code)]
pub fn age_skin(aging: &BodyAging) -> f32 {
    // Elasticity decreases with age
    let age_factor = (aging.params.age / 100.0).clamp(0.0, 1.0);
    1.0 - age_factor * 0.6
}

/// Compute posture degradation from aging.
#[allow(dead_code)]
pub fn age_posture(aging: &BodyAging) -> f32 {
    // Spine curvature increases after 50
    let over_50 = (aging.params.age - 50.0).max(0.0) / 70.0;
    over_50.clamp(0.0, 1.0) * 0.3
}

/// Compute muscle loss factor from aging.
#[allow(dead_code)]
pub fn age_muscle_loss(aging: &BodyAging) -> f32 {
    // Sarcopenia starts around 30, accelerates after 60
    let after_30 = (aging.params.age - 30.0).max(0.0) / 90.0;
    (after_30 * 0.4).clamp(0.0, 1.0)
}

/// Compute fat gain tendency from aging.
#[allow(dead_code)]
pub fn age_fat_gain(aging: &BodyAging) -> f32 {
    // Metabolism slows with age
    let factor = (aging.params.age / 80.0).clamp(0.0, 1.0);
    factor * 0.25
}

/// Compute height change from aging (negative for shrinkage).
#[allow(dead_code)]
pub fn age_height_change(aging: &BodyAging) -> f32 {
    // Height loss after 40
    let after_40 = (aging.params.age - 40.0).max(0.0) / 80.0;
    -(after_40.clamp(0.0, 1.0) * 0.05)
}

/// Convert aging to a parameter vector.
#[allow(dead_code)]
pub fn aging_to_params(aging: &BodyAging) -> [f32; 5] {
    [
        age_skin(aging),
        age_posture(aging),
        age_muscle_loss(aging),
        age_fat_gain(aging),
        age_height_change(aging),
    ]
}

/// Preview aging at a different age without modifying the state.
#[allow(dead_code)]
pub fn aging_preview(aging: &BodyAging, target_age: f32) -> [f32; 5] {
    let preview = new_body_aging(&aging.label, target_age);
    aging_to_params(&preview)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_body_aging() {
        let a = new_body_aging("test", 30.0);
        assert_eq!(a.label, "test");
        assert!((a.params.age - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_new_body_aging_clamps() {
        let a = new_body_aging("test", 200.0);
        assert!((a.params.age - 120.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_age_skin_young() {
        let a = new_body_aging("young", 20.0);
        let skin = age_skin(&a);
        assert!(skin > 0.8);
    }

    #[test]
    fn test_age_skin_old() {
        let a = new_body_aging("old", 100.0);
        let skin = age_skin(&a);
        assert!(skin < 0.5);
    }

    #[test]
    fn test_age_posture_young() {
        let a = new_body_aging("young", 25.0);
        assert!((age_posture(&a) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_age_muscle_loss_young() {
        let a = new_body_aging("young", 20.0);
        assert!((age_muscle_loss(&a) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_age_fat_gain() {
        let a = new_body_aging("mid", 60.0);
        let fat = age_fat_gain(&a);
        assert!(fat > 0.0);
    }

    #[test]
    fn test_age_height_change_young() {
        let a = new_body_aging("young", 30.0);
        assert!((age_height_change(&a) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_aging_to_params() {
        let a = new_body_aging("test", 50.0);
        let params = aging_to_params(&a);
        assert_eq!(params.len(), 5);
    }

    #[test]
    fn test_aging_preview() {
        let a = new_body_aging("test", 30.0);
        let preview = aging_preview(&a, 80.0);
        // At 80, should have noticeable aging
        assert!(preview[2] > 0.0); // muscle loss
    }
}
