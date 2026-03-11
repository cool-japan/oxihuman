#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export glTF animation data.

#[allow(dead_code)]
pub struct GltfAnimChannel {
    pub target_node: u32,
    pub path: String,
    pub sampler_idx: u32,
}

#[allow(dead_code)]
pub struct GltfAnimSampler {
    pub input: Vec<f32>,
    pub output: Vec<f32>,
    pub interpolation: String,
}

#[allow(dead_code)]
pub struct GltfAnimation {
    pub name: String,
    pub channels: Vec<GltfAnimChannel>,
    pub samplers: Vec<GltfAnimSampler>,
}

#[allow(dead_code)]
pub fn new_gltf_animation(name: &str) -> GltfAnimation {
    GltfAnimation { name: name.to_string(), channels: Vec::new(), samplers: Vec::new() }
}

#[allow(dead_code)]
pub fn add_channel(anim: &mut GltfAnimation, node: u32, path: &str, sampler: u32) {
    anim.channels.push(GltfAnimChannel { target_node: node, path: path.to_string(), sampler_idx: sampler });
}

#[allow(dead_code)]
pub fn add_sampler(anim: &mut GltfAnimation, times: Vec<f32>, values: Vec<f32>, interp: &str) -> u32 {
    let idx = anim.samplers.len() as u32;
    anim.samplers.push(GltfAnimSampler { input: times, output: values, interpolation: interp.to_string() });
    idx
}

#[allow(dead_code)]
pub fn export_gltf_anim_to_json(anim: &GltfAnimation) -> String {
    let mut s = format!("{{\"name\":\"{}\",\"channels\":[", anim.name);
    for (i, ch) in anim.channels.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&format!("{{\"node\":{},\"path\":\"{}\",\"sampler\":{}}}", ch.target_node, ch.path, ch.sampler_idx));
    }
    s.push_str("],\"samplers\":[");
    for (i, sa) in anim.samplers.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&format!("{{\"interpolation\":\"{}\",\"input_len\":{},\"output_len\":{}}}", sa.interpolation, sa.input.len(), sa.output.len()));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_anim_empty() {
        let a = new_gltf_animation("test");
        assert_eq!(a.name, "test");
        assert!(a.channels.is_empty());
        assert!(a.samplers.is_empty());
    }

    #[test]
    fn add_sampler_returns_index() {
        let mut a = new_gltf_animation("a");
        let idx = add_sampler(&mut a, vec![0.0, 1.0], vec![0.0, 1.0], "LINEAR");
        assert_eq!(idx, 0);
    }

    #[test]
    fn add_second_sampler_returns_one() {
        let mut a = new_gltf_animation("a");
        add_sampler(&mut a, vec![0.0], vec![0.0], "LINEAR");
        let idx = add_sampler(&mut a, vec![1.0], vec![1.0], "STEP");
        assert_eq!(idx, 1);
    }

    #[test]
    fn add_channel_stored() {
        let mut a = new_gltf_animation("a");
        add_channel(&mut a, 0, "translation", 0);
        assert_eq!(a.channels.len(), 1);
    }

    #[test]
    fn channel_path_correct() {
        let mut a = new_gltf_animation("a");
        add_channel(&mut a, 1, "rotation", 0);
        assert_eq!(a.channels[0].path, "rotation");
    }

    #[test]
    fn export_json_contains_name() {
        let a = new_gltf_animation("walk");
        let j = export_gltf_anim_to_json(&a);
        assert!(j.contains("walk"));
    }

    #[test]
    fn export_json_contains_channel_path() {
        let mut a = new_gltf_animation("run");
        let idx = add_sampler(&mut a, vec![0.0], vec![0.0], "LINEAR");
        add_channel(&mut a, 0, "scale", idx);
        let j = export_gltf_anim_to_json(&a);
        assert!(j.contains("scale"));
    }

    #[test]
    fn sampler_interpolation_stored() {
        let mut a = new_gltf_animation("a");
        add_sampler(&mut a, vec![0.0], vec![0.0], "CUBICSPLINE");
        assert_eq!(a.samplers[0].interpolation, "CUBICSPLINE");
    }

    #[test]
    fn sampler_input_stored() {
        let mut a = new_gltf_animation("a");
        add_sampler(&mut a, vec![0.0, 0.5, 1.0], vec![0.0], "LINEAR");
        assert_eq!(a.samplers[0].input.len(), 3);
    }
}
