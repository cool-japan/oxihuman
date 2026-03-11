//! Coefficient-of-restitution model for collision response (elastic, inelastic, mixed).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RestitutionConfig {
    pub default_restitution: f32,
    pub threshold_velocity: f32,
    pub combine_mode: CombineMode,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CombineMode {
    Average,
    Minimum,
    Maximum,
    Multiply,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialEntry {
    pub name: String,
    pub restitution: f32,
}

#[allow(dead_code)]
pub struct RestitutionModel {
    config: RestitutionConfig,
    materials: Vec<MaterialEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionPair {
    pub velocity_a: [f32; 3],
    pub velocity_b: [f32; 3],
    pub mass_a: f32,
    pub mass_b: f32,
    pub normal: [f32; 3],
    pub material_a: usize,
    pub material_b: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImpulseResult {
    pub impulse: f32,
    pub delta_va: [f32; 3],
    pub delta_vb: [f32; 3],
    pub restitution_used: f32,
}

#[allow(dead_code)]
pub fn default_restitution_config() -> RestitutionConfig {
    RestitutionConfig {
        default_restitution: 0.5,
        threshold_velocity: 0.01,
        combine_mode: CombineMode::Average,
    }
}

#[allow(dead_code)]
pub fn new_restitution_model(config: RestitutionConfig) -> RestitutionModel {
    RestitutionModel { config, materials: Vec::new() }
}

#[allow(dead_code)]
pub fn restitution_set_material(model: &mut RestitutionModel, name: &str, restitution: f32) {
    let r = restitution.clamp(0.0, 1.0);
    if let Some(m) = model.materials.iter_mut().find(|m| m.name == name) {
        m.restitution = r;
    } else {
        model.materials.push(MaterialEntry { name: name.to_string(), restitution: r });
    }
}

#[allow(dead_code)]
pub fn restitution_combined(model: &RestitutionModel, idx_a: usize, idx_b: usize) -> f32 {
    let ra = model.materials.get(idx_a).map(|m| m.restitution).unwrap_or(model.config.default_restitution);
    let rb = model.materials.get(idx_b).map(|m| m.restitution).unwrap_or(model.config.default_restitution);
    match model.config.combine_mode {
        CombineMode::Average  => (ra + rb) * 0.5,
        CombineMode::Minimum  => ra.min(rb),
        CombineMode::Maximum  => ra.max(rb),
        CombineMode::Multiply => ra * rb,
    }
}

fn dot3(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
}

#[allow(dead_code)]
pub fn restitution_compute_impulse(model: &RestitutionModel, pair: &CollisionPair) -> ImpulseResult {
    let e = restitution_combined(model, pair.material_a, pair.material_b);
    let rel_v = [
        pair.velocity_a[0] - pair.velocity_b[0],
        pair.velocity_a[1] - pair.velocity_b[1],
        pair.velocity_a[2] - pair.velocity_b[2],
    ];
    let vn = dot3(&rel_v, &pair.normal);

    // Only resolve if approaching
    if vn > -model.config.threshold_velocity {
        return ImpulseResult {
            impulse: 0.0,
            delta_va: [0.0; 3],
            delta_vb: [0.0; 3],
            restitution_used: e,
        };
    }

    let inv_ma = if pair.mass_a > 0.0 { 1.0 / pair.mass_a } else { 0.0 };
    let inv_mb = if pair.mass_b > 0.0 { 1.0 / pair.mass_b } else { 0.0 };
    let j = -(1.0 + e) * vn / (inv_ma + inv_mb);

    let n = pair.normal;
    let delta_va = [n[0]*j*inv_ma, n[1]*j*inv_ma, n[2]*j*inv_ma];
    let delta_vb = [-n[0]*j*inv_mb, -n[1]*j*inv_mb, -n[2]*j*inv_mb];

    ImpulseResult { impulse: j, delta_va, delta_vb, restitution_used: e }
}

#[allow(dead_code)]
pub fn restitution_coefficient(model: &RestitutionModel, idx: usize) -> f32 {
    model.materials.get(idx).map(|m| m.restitution).unwrap_or(model.config.default_restitution)
}

#[allow(dead_code)]
pub fn restitution_material_count(model: &RestitutionModel) -> usize {
    model.materials.len()
}

#[allow(dead_code)]
pub fn restitution_is_elastic(model: &RestitutionModel, idx_a: usize, idx_b: usize) -> bool {
    (restitution_combined(model, idx_a, idx_b) - 1.0).abs() < 1e-4
}

#[allow(dead_code)]
pub fn restitution_to_json(model: &RestitutionModel) -> String {
    let mats: Vec<String> = model.materials.iter().map(|m| format!("\"{}\":{:.4}", m.name, m.restitution)).collect();
    format!(
        "{{\"material_count\":{},\"default_restitution\":{:.4},\"materials\":{{{}}}}}",
        model.materials.len(), model.config.default_restitution, mats.join(",")
    )
}

#[allow(dead_code)]
pub fn restitution_reset(model: &mut RestitutionModel) {
    model.materials.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_model() -> RestitutionModel {
        let mut m = new_restitution_model(default_restitution_config());
        restitution_set_material(&mut m, "rubber", 0.8);
        restitution_set_material(&mut m, "steel", 0.3);
        m
    }

    #[test]
    fn test_default_config() {
        let cfg = default_restitution_config();
        assert!((cfg.default_restitution - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_material_count() {
        let m = make_model();
        assert_eq!(restitution_material_count(&m), 2);
    }

    #[test]
    fn test_coefficient_lookup() {
        let m = make_model();
        assert!((restitution_coefficient(&m, 0) - 0.8).abs() < 1e-5);
        assert!((restitution_coefficient(&m, 1) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_combined_average() {
        let m = make_model();
        let c = restitution_combined(&m, 0, 1);
        assert!((c - 0.55).abs() < 1e-5);
    }

    #[test]
    fn test_is_elastic_false() {
        let m = make_model();
        assert!(!restitution_is_elastic(&m, 0, 1));
    }

    #[test]
    fn test_is_elastic_true() {
        let mut m = new_restitution_model(default_restitution_config());
        restitution_set_material(&mut m, "elastic", 1.0);
        restitution_set_material(&mut m, "elastic2", 1.0);
        assert!(restitution_is_elastic(&m, 0, 1));
    }

    #[test]
    fn test_impulse_approaching() {
        let m = make_model();
        let pair = CollisionPair {
            velocity_a: [-1.0, 0.0, 0.0],
            velocity_b: [1.0, 0.0, 0.0],
            mass_a: 1.0, mass_b: 1.0,
            normal: [1.0, 0.0, 0.0],
            material_a: 0, material_b: 1,
        };
        let res = restitution_compute_impulse(&m, &pair);
        assert!(res.impulse > 0.0);
    }

    #[test]
    fn test_impulse_separating() {
        let m = make_model();
        let pair = CollisionPair {
            velocity_a: [1.0, 0.0, 0.0],
            velocity_b: [-1.0, 0.0, 0.0],
            mass_a: 1.0, mass_b: 1.0,
            normal: [1.0, 0.0, 0.0],
            material_a: 0, material_b: 1,
        };
        let res = restitution_compute_impulse(&m, &pair);
        assert!((res.impulse - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_reset() {
        let mut m = make_model();
        restitution_reset(&mut m);
        assert_eq!(restitution_material_count(&m), 0);
    }

    #[test]
    fn test_to_json_fields() {
        let m = make_model();
        let json = restitution_to_json(&m);
        assert!(json.contains("material_count"));
        assert!(json.contains("rubber"));
    }
}
