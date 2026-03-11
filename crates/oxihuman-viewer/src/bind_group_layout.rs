#![allow(dead_code)]

//! Bind group layout descriptor stub.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingType {
    UniformBuffer,
    StorageBuffer,
    Texture2d,
    TextureCube,
    Sampler,
    StorageImage,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
    All,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BindingDescriptor {
    pub binding: u32,
    pub binding_type: BindingType,
    pub shader_stage: ShaderStage,
    pub count: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BindGroupLayout {
    pub set: u32,
    pub bindings: Vec<BindingDescriptor>,
}

#[allow(dead_code)]
pub fn new_bind_group_layout(set: u32) -> BindGroupLayout {
    BindGroupLayout {
        set,
        bindings: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn bgl_add_binding(
    layout: &mut BindGroupLayout,
    binding: u32,
    binding_type: BindingType,
    shader_stage: ShaderStage,
    count: u32,
) {
    layout.bindings.push(BindingDescriptor {
        binding,
        binding_type,
        shader_stage,
        count: count.max(1),
    });
}

#[allow(dead_code)]
pub fn bgl_binding_count(layout: &BindGroupLayout) -> usize {
    layout.bindings.len()
}

#[allow(dead_code)]
pub fn bgl_has_binding(layout: &BindGroupLayout, binding: u32) -> bool {
    layout.bindings.iter().any(|b| b.binding == binding)
}

#[allow(dead_code)]
pub fn bgl_remove_binding(layout: &mut BindGroupLayout, binding: u32) {
    layout.bindings.retain(|b| b.binding != binding);
}

#[allow(dead_code)]
pub fn bgl_clear(layout: &mut BindGroupLayout) {
    layout.bindings.clear();
}

#[allow(dead_code)]
pub fn bgl_texture_binding_count(layout: &BindGroupLayout) -> usize {
    layout
        .bindings
        .iter()
        .filter(|b| matches!(b.binding_type, BindingType::Texture2d | BindingType::TextureCube))
        .count()
}

#[allow(dead_code)]
pub fn bgl_uniform_binding_count(layout: &BindGroupLayout) -> usize {
    layout
        .bindings
        .iter()
        .filter(|b| b.binding_type == BindingType::UniformBuffer)
        .count()
}

#[allow(dead_code)]
pub fn bgl_to_json(layout: &BindGroupLayout) -> String {
    format!(
        "{{\"set\":{},\"binding_count\":{}}}",
        layout.set,
        layout.bindings.len()
    )
}

#[allow(dead_code)]
pub fn bgl_compute_hash(layout: &BindGroupLayout) -> u64 {
    layout
        .bindings
        .iter()
        .fold(layout.set as u64, |acc, b| {
            acc.wrapping_add(b.binding as u64)
                .wrapping_mul(0x9e3779b97f4a7c15)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_layout() {
        let l = new_bind_group_layout(0);
        assert_eq!(bgl_binding_count(&l), 0);
    }

    #[test]
    fn test_add_binding() {
        let mut l = new_bind_group_layout(0);
        bgl_add_binding(&mut l, 0, BindingType::UniformBuffer, ShaderStage::Vertex, 1);
        assert_eq!(bgl_binding_count(&l), 1);
    }

    #[test]
    fn test_has_binding() {
        let mut l = new_bind_group_layout(0);
        bgl_add_binding(&mut l, 1, BindingType::Texture2d, ShaderStage::Fragment, 1);
        assert!(bgl_has_binding(&l, 1));
        assert!(!bgl_has_binding(&l, 99));
    }

    #[test]
    fn test_remove_binding() {
        let mut l = new_bind_group_layout(0);
        bgl_add_binding(&mut l, 0, BindingType::Sampler, ShaderStage::Fragment, 1);
        bgl_remove_binding(&mut l, 0);
        assert!(!bgl_has_binding(&l, 0));
    }

    #[test]
    fn test_clear() {
        let mut l = new_bind_group_layout(0);
        bgl_add_binding(&mut l, 0, BindingType::StorageBuffer, ShaderStage::Compute, 1);
        bgl_clear(&mut l);
        assert_eq!(bgl_binding_count(&l), 0);
    }

    #[test]
    fn test_texture_binding_count() {
        let mut l = new_bind_group_layout(0);
        bgl_add_binding(&mut l, 0, BindingType::Texture2d, ShaderStage::Fragment, 1);
        bgl_add_binding(&mut l, 1, BindingType::TextureCube, ShaderStage::Fragment, 1);
        bgl_add_binding(&mut l, 2, BindingType::UniformBuffer, ShaderStage::Vertex, 1);
        assert_eq!(bgl_texture_binding_count(&l), 2);
    }

    #[test]
    fn test_uniform_binding_count() {
        let mut l = new_bind_group_layout(0);
        bgl_add_binding(&mut l, 0, BindingType::UniformBuffer, ShaderStage::All, 1);
        bgl_add_binding(&mut l, 1, BindingType::UniformBuffer, ShaderStage::All, 1);
        assert_eq!(bgl_uniform_binding_count(&l), 2);
    }

    #[test]
    fn test_hash_deterministic() {
        let mut l = new_bind_group_layout(0);
        bgl_add_binding(&mut l, 0, BindingType::UniformBuffer, ShaderStage::Vertex, 1);
        assert_eq!(bgl_compute_hash(&l), bgl_compute_hash(&l));
    }

    #[test]
    fn test_to_json() {
        let l = new_bind_group_layout(1);
        let json = bgl_to_json(&l);
        assert!(json.contains("\"set\":1"));
    }

    #[test]
    fn test_count_clamped_to_one() {
        let mut l = new_bind_group_layout(0);
        bgl_add_binding(&mut l, 0, BindingType::Texture2d, ShaderStage::Fragment, 0);
        assert_eq!(l.bindings[0].count, 1);
    }
}
