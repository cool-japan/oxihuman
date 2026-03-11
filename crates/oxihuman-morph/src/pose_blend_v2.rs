#![allow(dead_code)]

//! Pose blending with additive and override layers.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum PoseLayerMode {
    Override,
    Additive,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseLayer {
    pub name: String,
    pub weight: f32,
    pub mode: PoseLayerMode,
    pub pose: Vec<[f32; 4]>,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseBlendV2 {
    pub layers: Vec<PoseLayer>,
    pub joint_count: usize,
}

#[allow(dead_code)]
pub fn new_pose_blend_v2(joint_count: usize) -> PoseBlendV2 {
    PoseBlendV2 {
        layers: Vec::new(),
        joint_count,
    }
}

#[allow(dead_code)]
pub fn pbv2_add_layer(
    blend: &mut PoseBlendV2,
    name: &str,
    weight: f32,
    mode: PoseLayerMode,
    pose: Vec<[f32; 4]>,
) {
    blend.layers.push(PoseLayer {
        name: name.to_string(),
        weight: weight.clamp(0.0, 1.0),
        mode,
        pose,
        enabled: true,
    });
}

#[allow(dead_code)]
pub fn pbv2_evaluate(blend: &PoseBlendV2) -> Vec<[f32; 4]> {
    let mut result = vec![[0.0_f32; 4]; blend.joint_count];
    for layer in &blend.layers {
        if !layer.enabled || layer.weight.abs() < 1e-9 {
            continue;
        }
        match layer.mode {
            PoseLayerMode::Override => {
                for (i, r) in result.iter_mut().enumerate() {
                    if let Some(p) = layer.pose.get(i) {
                        *r = [
                            r[0] * (1.0 - layer.weight) + p[0] * layer.weight,
                            r[1] * (1.0 - layer.weight) + p[1] * layer.weight,
                            r[2] * (1.0 - layer.weight) + p[2] * layer.weight,
                            r[3] * (1.0 - layer.weight) + p[3] * layer.weight,
                        ];
                        let _ = i;
                    }
                }
            }
            PoseLayerMode::Additive => {
                for (i, r) in result.iter_mut().enumerate() {
                    if let Some(p) = layer.pose.get(i) {
                        for k in 0..4 {
                            r[k] += p[k] * layer.weight;
                        }
                    }
                }
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn pbv2_set_layer_weight(blend: &mut PoseBlendV2, name: &str, weight: f32) {
    if let Some(layer) = blend.layers.iter_mut().find(|l| l.name == name) {
        layer.weight = weight.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn pbv2_enable_layer(blend: &mut PoseBlendV2, name: &str, enabled: bool) {
    if let Some(layer) = blend.layers.iter_mut().find(|l| l.name == name) {
        layer.enabled = enabled;
    }
}

#[allow(dead_code)]
pub fn pbv2_layer_count(blend: &PoseBlendV2) -> usize {
    blend.layers.len()
}

#[allow(dead_code)]
pub fn pbv2_clear(blend: &mut PoseBlendV2) {
    blend.layers.clear();
}

#[allow(dead_code)]
pub fn pbv2_to_json(blend: &PoseBlendV2) -> String {
    format!(
        "{{\"layer_count\":{},\"joint_count\":{}}}",
        blend.layers.len(),
        blend.joint_count
    )
}

#[allow(dead_code)]
pub fn pbv2_active_layer_count(blend: &PoseBlendV2) -> usize {
    blend.layers.iter().filter(|l| l.enabled).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pose_blend() {
        let b = new_pose_blend_v2(16);
        assert_eq!(pbv2_layer_count(&b), 0);
        assert_eq!(b.joint_count, 16);
    }

    #[test]
    fn test_add_layer() {
        let mut b = new_pose_blend_v2(2);
        pbv2_add_layer(&mut b, "base", 1.0, PoseLayerMode::Override, vec![[0.0; 4]; 2]);
        assert_eq!(pbv2_layer_count(&b), 1);
    }

    #[test]
    fn test_evaluate_additive() {
        let mut b = new_pose_blend_v2(1);
        pbv2_add_layer(&mut b, "add", 1.0, PoseLayerMode::Additive, vec![[1.0, 0.0, 0.0, 0.0]]);
        let result = pbv2_evaluate(&b);
        assert!((result[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_override() {
        let mut b = new_pose_blend_v2(1);
        pbv2_add_layer(&mut b, "ov", 1.0, PoseLayerMode::Override, vec![[0.5, 0.0, 0.0, 0.0]]);
        let result = pbv2_evaluate(&b);
        assert!((result[0][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_layer_weight() {
        let mut b = new_pose_blend_v2(1);
        pbv2_add_layer(&mut b, "l", 1.0, PoseLayerMode::Additive, vec![[1.0; 4]]);
        pbv2_set_layer_weight(&mut b, "l", 0.5);
        assert!((b.layers[0].weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_enable_disable_layer() {
        let mut b = new_pose_blend_v2(1);
        pbv2_add_layer(&mut b, "l", 1.0, PoseLayerMode::Additive, vec![[1.0; 4]]);
        pbv2_enable_layer(&mut b, "l", false);
        assert_eq!(pbv2_active_layer_count(&b), 0);
    }

    #[test]
    fn test_clear() {
        let mut b = new_pose_blend_v2(4);
        pbv2_add_layer(&mut b, "x", 1.0, PoseLayerMode::Override, vec![]);
        pbv2_clear(&mut b);
        assert_eq!(pbv2_layer_count(&b), 0);
    }

    #[test]
    fn test_to_json() {
        let b = new_pose_blend_v2(4);
        let json = pbv2_to_json(&b);
        assert!(json.contains("joint_count"));
    }

    #[test]
    fn test_active_layer_count() {
        let mut b = new_pose_blend_v2(2);
        pbv2_add_layer(&mut b, "a", 1.0, PoseLayerMode::Additive, vec![]);
        pbv2_add_layer(&mut b, "b", 1.0, PoseLayerMode::Additive, vec![]);
        pbv2_enable_layer(&mut b, "b", false);
        assert_eq!(pbv2_active_layer_count(&b), 1);
    }

    #[test]
    fn test_evaluate_disabled_layer_ignored() {
        let mut b = new_pose_blend_v2(1);
        pbv2_add_layer(&mut b, "l", 1.0, PoseLayerMode::Additive, vec![[5.0; 4]]);
        pbv2_enable_layer(&mut b, "l", false);
        let result = pbv2_evaluate(&b);
        assert!((result[0][0] - 0.0).abs() < 1e-6);
    }
}
