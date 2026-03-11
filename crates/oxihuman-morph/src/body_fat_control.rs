//! Body fat distribution controls across anatomical regions.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyFatConfig {
    pub total_fat: f32,
    pub android_ratio: f32,
    pub gynoid_ratio: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyFatState {
    pub abdomen: f32,
    pub chest: f32,
    pub hips: f32,
    pub thighs: f32,
    pub arms: f32,
    pub face: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyFatWeights {
    pub belly: f32,
    pub love_handles: f32,
    pub breast: f32,
    pub hip_wide: f32,
    pub thigh_thick: f32,
    pub arm_fat: f32,
}

#[allow(dead_code)]
pub fn default_body_fat_config() -> BodyFatConfig {
    BodyFatConfig {
        total_fat: 0.2,
        android_ratio: 0.5,
        gynoid_ratio: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_body_fat_state() -> BodyFatState {
    BodyFatState {
        abdomen: 0.2,
        chest: 0.2,
        hips: 0.2,
        thighs: 0.2,
        arms: 0.2,
        face: 0.2,
    }
}

#[allow(dead_code)]
pub fn set_total_fat(state: &mut BodyFatState, cfg: &BodyFatConfig, total: f32) {
    let t = total.clamp(0.0, 1.0);
    let android = t * cfg.android_ratio;
    let gynoid = t * cfg.gynoid_ratio;
    state.abdomen = android.clamp(0.0, 1.0);
    state.chest = android.clamp(0.0, 1.0);
    state.hips = gynoid.clamp(0.0, 1.0);
    state.thighs = gynoid.clamp(0.0, 1.0);
    state.arms = (t * 0.5).clamp(0.0, 1.0);
    state.face = (t * 0.3).clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_region_fat(state: &mut BodyFatState, abdomen: f32, chest: f32, hips: f32) {
    state.abdomen = abdomen.clamp(0.0, 1.0);
    state.chest = chest.clamp(0.0, 1.0);
    state.hips = hips.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_fat_weights(state: &BodyFatState, cfg: &BodyFatConfig) -> BodyFatWeights {
    let scale = cfg.total_fat.clamp(0.0, 1.0);
    BodyFatWeights {
        belly: (state.abdomen * scale).clamp(0.0, 1.0),
        love_handles: (state.abdomen * scale * 0.7).clamp(0.0, 1.0),
        breast: (state.chest * scale).clamp(0.0, 1.0),
        hip_wide: (state.hips * scale).clamp(0.0, 1.0),
        thigh_thick: (state.thighs * scale).clamp(0.0, 1.0),
        arm_fat: (state.arms * scale).clamp(0.0, 1.0),
    }
}

#[allow(dead_code)]
pub fn blend_body_fat(a: &BodyFatState, b: &BodyFatState, t: f32) -> BodyFatState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    BodyFatState {
        abdomen: a.abdomen * inv + b.abdomen * t,
        chest: a.chest * inv + b.chest * t,
        hips: a.hips * inv + b.hips * t,
        thighs: a.thighs * inv + b.thighs * t,
        arms: a.arms * inv + b.arms * t,
        face: a.face * inv + b.face * t,
    }
}

#[allow(dead_code)]
pub fn reset_body_fat(state: &mut BodyFatState) {
    *state = new_body_fat_state();
}

#[allow(dead_code)]
pub fn body_fat_to_json(state: &BodyFatState) -> String {
    format!(
        r#"{{"abdomen":{:.4},"chest":{:.4},"hips":{:.4},"thighs":{:.4},"arms":{:.4},"face":{:.4}}}"#,
        state.abdomen, state.chest, state.hips, state.thighs, state.arms, state.face
    )
}

#[allow(dead_code)]
pub fn total_fat_percentage(state: &BodyFatState) -> f32 {
    (state.abdomen + state.chest + state.hips + state.thighs + state.arms + state.face) / 6.0
}

#[allow(dead_code)]
pub fn bmi_estimate_fat(state: &BodyFatState) -> f32 {
    let fat = total_fat_percentage(state);
    // Linear approximation: 18.5 at 0% fat, 35.0 at 100%
    18.5 + fat * 16.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_body_fat_config() {
        let cfg = default_body_fat_config();
        assert!((cfg.total_fat - 0.2).abs() < 1e-6);
        assert!((cfg.android_ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_body_fat_state() {
        let s = new_body_fat_state();
        assert!((s.abdomen - 0.2).abs() < 1e-6);
        assert!((s.face - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_set_total_fat_clamps() {
        let mut s = new_body_fat_state();
        let cfg = default_body_fat_config();
        set_total_fat(&mut s, &cfg, 1.5);
        assert!(s.abdomen <= 1.0);
        assert!(s.hips <= 1.0);
    }

    #[test]
    fn test_set_region_fat() {
        let mut s = new_body_fat_state();
        set_region_fat(&mut s, 0.9, 0.1, 0.6);
        assert!((s.abdomen - 0.9).abs() < 1e-6);
        assert!((s.chest - 0.1).abs() < 1e-6);
        assert!((s.hips - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_blend_body_fat_midpoint() {
        let a = new_body_fat_state();
        let mut b = new_body_fat_state();
        b.abdomen = 0.8;
        let r = blend_body_fat(&a, &b, 0.5);
        assert!((r.abdomen - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_total_fat_percentage_uniform() {
        let mut s = new_body_fat_state();
        s.abdomen = 0.6;
        s.chest = 0.6;
        s.hips = 0.6;
        s.thighs = 0.6;
        s.arms = 0.6;
        s.face = 0.6;
        let pct = total_fat_percentage(&s);
        assert!((pct - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_bmi_estimate_fat_range() {
        let s = new_body_fat_state();
        let bmi = bmi_estimate_fat(&s);
        assert!((18.5..=35.0).contains(&bmi));
    }

    #[test]
    fn test_body_fat_to_json_contains_fields() {
        let s = new_body_fat_state();
        let j = body_fat_to_json(&s);
        assert!(j.contains("abdomen"));
        assert!(j.contains("thighs"));
        assert!(j.contains("face"));
    }

    #[test]
    fn test_reset_body_fat() {
        let mut s = new_body_fat_state();
        s.abdomen = 0.99;
        reset_body_fat(&mut s);
        assert!((s.abdomen - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_compute_fat_weights_range() {
        let s = new_body_fat_state();
        let cfg = default_body_fat_config();
        let w = compute_fat_weights(&s, &cfg);
        assert!(w.belly >= 0.0 && w.belly <= 1.0);
        assert!(w.thigh_thick >= 0.0 && w.thigh_thick <= 1.0);
    }
}
