//! Corrective blend shapes — delta morphs applied at extreme joint poses to fix skin deformation.

#[allow(dead_code)]
pub struct CorrectiveConfig {
    pub influence_falloff: f32,
    pub max_triggers: usize,
    pub global_scale: f32,
}

#[allow(dead_code)]
pub struct CorrectiveTrigger {
    pub joint: String,
    pub angle_rad: f32,
    pub weight: f32,
}

#[allow(dead_code)]
pub struct CorrectiveBlendShape {
    pub name: String,
    pub delta_count: usize,
    pub triggers: Vec<CorrectiveTrigger>,
    pub global_scale: f32,
    pub config: CorrectiveConfig,
}

#[allow(dead_code)]
pub fn default_corrective_config() -> CorrectiveConfig {
    CorrectiveConfig {
        influence_falloff: 0.1,
        max_triggers: 16,
        global_scale: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_corrective_blend_shape(
    name: &str,
    delta_count: usize,
    cfg: &CorrectiveConfig,
) -> CorrectiveBlendShape {
    CorrectiveBlendShape {
        name: name.to_string(),
        delta_count,
        triggers: Vec::new(),
        global_scale: cfg.global_scale,
        config: CorrectiveConfig {
            influence_falloff: cfg.influence_falloff,
            max_triggers: cfg.max_triggers,
            global_scale: cfg.global_scale,
        },
    }
}

#[allow(dead_code)]
pub fn corrective_add_trigger(
    shape: &mut CorrectiveBlendShape,
    joint: &str,
    angle_rad: f32,
    weight: f32,
) {
    if shape.triggers.len() < shape.config.max_triggers {
        shape.triggers.push(CorrectiveTrigger {
            joint: joint.to_string(),
            angle_rad,
            weight: weight.clamp(0.0, 1.0),
        });
    }
}

/// Evaluate the blend shape activation for the given joint angles.
///
/// Each trigger contributes based on how close the current joint angle is to
/// its target `angle_rad`, using a Gaussian-like falloff.
#[allow(dead_code)]
pub fn corrective_evaluate(
    shape: &CorrectiveBlendShape,
    joint_angles: &[(&str, f32)],
) -> f32 {
    if shape.triggers.is_empty() {
        return 0.0;
    }
    let falloff = shape.config.influence_falloff.max(1e-6);
    let mut total = 0.0f32;
    for trigger in &shape.triggers {
        // Find the matching joint angle
        let current = joint_angles
            .iter()
            .find(|(j, _)| *j == trigger.joint)
            .map(|(_, a)| *a)
            .unwrap_or(0.0);
        let diff = (current - trigger.angle_rad).abs();
        // Gaussian-like: contribution falls off as diff grows
        let contribution = trigger.weight * (-diff / falloff).exp();
        total += contribution;
    }
    (total * shape.global_scale).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn corrective_shape_name(shape: &CorrectiveBlendShape) -> &str {
    &shape.name
}

#[allow(dead_code)]
pub fn corrective_trigger_count(shape: &CorrectiveBlendShape) -> usize {
    shape.triggers.len()
}

#[allow(dead_code)]
pub fn corrective_delta_count(shape: &CorrectiveBlendShape) -> usize {
    shape.delta_count
}

#[allow(dead_code)]
pub fn corrective_reset_triggers(shape: &mut CorrectiveBlendShape) {
    shape.triggers.clear();
}

#[allow(dead_code)]
pub fn corrective_set_global_scale(shape: &mut CorrectiveBlendShape, scale: f32) {
    shape.global_scale = scale.clamp(0.0, 10.0);
}

#[allow(dead_code)]
pub fn corrective_is_active(
    shape: &CorrectiveBlendShape,
    joint_angles: &[(&str, f32)],
) -> bool {
    corrective_evaluate(shape, joint_angles) > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_corrective_config();
        assert!(cfg.influence_falloff > 0.0);
        assert_eq!(cfg.max_triggers, 16);
        assert!((cfg.global_scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_corrective_blend_shape() {
        let cfg = default_corrective_config();
        let shape = new_corrective_blend_shape("elbow_fix", 128, &cfg);
        assert_eq!(corrective_shape_name(&shape), "elbow_fix");
        assert_eq!(corrective_delta_count(&shape), 128);
        assert_eq!(corrective_trigger_count(&shape), 0);
    }

    #[test]
    fn test_add_trigger() {
        let cfg = default_corrective_config();
        let mut shape = new_corrective_blend_shape("fix", 64, &cfg);
        corrective_add_trigger(&mut shape, "elbow_l", 1.57, 1.0);
        assert_eq!(corrective_trigger_count(&shape), 1);
    }

    #[test]
    fn test_evaluate_at_exact_angle() {
        let cfg = default_corrective_config();
        let mut shape = new_corrective_blend_shape("fix", 64, &cfg);
        corrective_add_trigger(&mut shape, "elbow_l", 1.0, 1.0);
        // Exact match: e^0 = 1.0 -> weight = 1.0
        let result = corrective_evaluate(&shape, &[("elbow_l", 1.0)]);
        assert!((result - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_no_triggers_returns_zero() {
        let cfg = default_corrective_config();
        let shape = new_corrective_blend_shape("empty", 32, &cfg);
        let result = corrective_evaluate(&shape, &[("any_joint", 0.5)]);
        assert!((result).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_distant_angle_low_contribution() {
        let cfg = default_corrective_config();
        let mut shape = new_corrective_blend_shape("fix", 64, &cfg);
        corrective_add_trigger(&mut shape, "knee_l", 0.0, 1.0);
        // Very different angle → low activation
        let result = corrective_evaluate(&shape, &[("knee_l", 100.0)]);
        assert!(result < 0.01);
    }

    #[test]
    fn test_reset_triggers() {
        let cfg = default_corrective_config();
        let mut shape = new_corrective_blend_shape("fix", 64, &cfg);
        corrective_add_trigger(&mut shape, "hip_l", 0.5, 0.8);
        corrective_add_trigger(&mut shape, "hip_r", 0.3, 0.6);
        corrective_reset_triggers(&mut shape);
        assert_eq!(corrective_trigger_count(&shape), 0);
    }

    #[test]
    fn test_set_global_scale_zero() {
        let cfg = default_corrective_config();
        let mut shape = new_corrective_blend_shape("fix", 64, &cfg);
        corrective_add_trigger(&mut shape, "elbow_l", 1.0, 1.0);
        corrective_set_global_scale(&mut shape, 0.0);
        let result = corrective_evaluate(&shape, &[("elbow_l", 1.0)]);
        assert!((result).abs() < 1e-6);
    }

    #[test]
    fn test_is_active_true() {
        let cfg = default_corrective_config();
        let mut shape = new_corrective_blend_shape("fix", 64, &cfg);
        corrective_add_trigger(&mut shape, "elbow_l", 1.0, 1.0);
        assert!(corrective_is_active(&shape, &[("elbow_l", 1.0)]));
    }

    #[test]
    fn test_is_active_false_no_triggers() {
        let cfg = default_corrective_config();
        let shape = new_corrective_blend_shape("empty", 64, &cfg);
        assert!(!corrective_is_active(&shape, &[("elbow_l", 1.0)]));
    }

    #[test]
    fn test_max_triggers_cap() {
        let mut cfg = default_corrective_config();
        cfg.max_triggers = 2;
        let mut shape = new_corrective_blend_shape("small", 64, &cfg);
        corrective_add_trigger(&mut shape, "j1", 0.1, 1.0);
        corrective_add_trigger(&mut shape, "j2", 0.2, 1.0);
        corrective_add_trigger(&mut shape, "j3", 0.3, 1.0); // over limit
        assert_eq!(corrective_trigger_count(&shape), 2);
    }
}
