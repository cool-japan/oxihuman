#![allow(dead_code)]
//! Body posture representation and blending.

/// Type of posture.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostureType {
    Standing,
    Sitting,
    Crouching,
    Leaning,
    Supine,
}

/// Body posture state with named parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyPosture {
    /// Human-readable label.
    pub name: String,
    /// Posture classification.
    pub posture_type: PostureType,
    /// Spine curvature factor [-1, 1] (negative = slouch, positive = upright).
    pub spine_curve: f32,
    /// Shoulder roll in [-1, 1].
    pub shoulder_roll: f32,
    /// Hip tilt in [-1, 1].
    pub hip_tilt: f32,
    /// Head tilt in [-1, 1].
    pub head_tilt: f32,
}

/// Create a new [`BodyPosture`] with default neutral standing.
#[allow(dead_code)]
pub fn new_body_posture(name: &str, posture_type: PostureType) -> BodyPosture {
    BodyPosture {
        name: name.to_string(),
        posture_type,
        spine_curve: 0.0,
        shoulder_roll: 0.0,
        hip_tilt: 0.0,
        head_tilt: 0.0,
    }
}

/// Convert posture to a parameter vector [spine, shoulder, hip, head].
#[allow(dead_code)]
pub fn posture_to_params(p: &BodyPosture) -> [f32; 4] {
    [p.spine_curve, p.shoulder_roll, p.hip_tilt, p.head_tilt]
}

/// Apply posture parameters to a mutable posture.
#[allow(dead_code)]
pub fn apply_posture(p: &mut BodyPosture, params: &[f32; 4]) {
    p.spine_curve = params[0].clamp(-1.0, 1.0);
    p.shoulder_roll = params[1].clamp(-1.0, 1.0);
    p.hip_tilt = params[2].clamp(-1.0, 1.0);
    p.head_tilt = params[3].clamp(-1.0, 1.0);
}

/// Linearly blend two postures by factor t in [0, 1].
#[allow(dead_code)]
pub fn posture_blend(a: &BodyPosture, b: &BodyPosture, t: f32) -> BodyPosture {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    BodyPosture {
        name: a.name.clone(),
        posture_type: if t < 0.5 { a.posture_type } else { b.posture_type },
        spine_curve: a.spine_curve * inv + b.spine_curve * t,
        shoulder_roll: a.shoulder_roll * inv + b.shoulder_roll * t,
        hip_tilt: a.hip_tilt * inv + b.hip_tilt * t,
        head_tilt: a.head_tilt * inv + b.head_tilt * t,
    }
}

/// Return the name of the posture.
#[allow(dead_code)]
pub fn posture_name(p: &BodyPosture) -> &str {
    &p.name
}

/// Return the posture type.
#[allow(dead_code)]
pub fn posture_type(p: &BodyPosture) -> PostureType {
    p.posture_type
}

/// Return a default standing posture with neutral parameters.
#[allow(dead_code)]
pub fn default_standing_posture() -> BodyPosture {
    new_body_posture("default_standing", PostureType::Standing)
}

/// Compute deviation from neutral (L2 norm of params).
#[allow(dead_code)]
pub fn posture_deviation(p: &BodyPosture) -> f32 {
    let params = posture_to_params(p);
    params.iter().map(|v| v * v).sum::<f32>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_body_posture() {
        let p = new_body_posture("stand", PostureType::Standing);
        assert_eq!(p.name, "stand");
        assert_eq!(p.posture_type, PostureType::Standing);
    }

    #[test]
    fn test_posture_to_params_default() {
        let p = default_standing_posture();
        let params = posture_to_params(&p);
        assert!(params.iter().all(|v| v.abs() < f32::EPSILON));
    }

    #[test]
    fn test_apply_posture() {
        let mut p = default_standing_posture();
        apply_posture(&mut p, &[0.5, -0.3, 0.1, 0.2]);
        assert!((p.spine_curve - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_posture_clamps() {
        let mut p = default_standing_posture();
        apply_posture(&mut p, &[2.0, -2.0, 0.0, 0.0]);
        assert!((p.spine_curve - 1.0).abs() < f32::EPSILON);
        assert!((p.shoulder_roll - (-1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_posture_blend_zero() {
        let a = new_body_posture("a", PostureType::Standing);
        let mut b = new_body_posture("b", PostureType::Sitting);
        b.spine_curve = 1.0;
        let result = posture_blend(&a, &b, 0.0);
        assert!((result.spine_curve - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_posture_blend_one() {
        let a = new_body_posture("a", PostureType::Standing);
        let mut b = new_body_posture("b", PostureType::Sitting);
        b.spine_curve = 1.0;
        let result = posture_blend(&a, &b, 1.0);
        assert!((result.spine_curve - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_posture_name() {
        let p = new_body_posture("relaxed", PostureType::Leaning);
        assert_eq!(posture_name(&p), "relaxed");
    }

    #[test]
    fn test_posture_type_fn() {
        let p = new_body_posture("sit", PostureType::Sitting);
        assert_eq!(posture_type(&p), PostureType::Sitting);
    }

    #[test]
    fn test_default_standing_posture() {
        let p = default_standing_posture();
        assert_eq!(p.posture_type, PostureType::Standing);
    }

    #[test]
    fn test_posture_deviation() {
        let p = default_standing_posture();
        assert!(posture_deviation(&p).abs() < f32::EPSILON);
        let mut p2 = default_standing_posture();
        p2.spine_curve = 1.0;
        assert!(posture_deviation(&p2) > 0.0);
    }
}
